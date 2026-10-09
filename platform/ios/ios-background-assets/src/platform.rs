use core::ptr::NonNull;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use block2::RcBlock;
use objc2::{AnyThread, rc::autoreleasepool};
use objc2_background_assets::{
    BAAppExtensionInfo, BAAssetPackManifest, BADownload, BADownloadManager,
    BADownloaderPriorityDefault,
};
use objc2_foundation::{NSArray, NSData, NSError, NSString};

use crate::completion::Completion;
use crate::{
    AssetPackManifestCount, AssetPackManifestEntry, AssetPackManifestError,
    DownloadQueueQueryError, ExtensionDownloadAllowanceError, NativeBackgroundAssetsError,
};

pub(crate) fn inspect_restricted_download_size_remaining(
    info: &BAAppExtensionInfo,
) -> Result<Option<u64>, ExtensionDownloadAllowanceError> {
    if !objc2::available!(ios = 16.1, ..) {
        return Err(ExtensionDownloadAllowanceError::ApiUnavailable);
    }

    // SAFETY: `info` is borrowed from Apple's downloader-extension callback. The installed SDK
    // declares this property readonly and omits `nonatomic`, so its getter is atomic. The guard
    // above establishes the property's iOS 16.1 availability floor.
    unsafe { info.restrictedDownloadSizeRemaining() }
        .map(|remaining| {
            // Apple refines the NSNumber property to Swift `Int?`; its integer value is therefore
            // the documented signed native count. Preserve `nil` and reject negative byte counts.
            u64::try_from(remaining.integerValue())
                .map_err(|_| ExtensionDownloadAllowanceError::InvalidRemainingBytes)
        })
        .transpose()
}

pub(crate) fn inspect_restricted_essential_download_size_remaining(
    info: &BAAppExtensionInfo,
) -> Result<Option<u64>, ExtensionDownloadAllowanceError> {
    if !objc2::available!(ios = 16.4, ..) {
        return Err(ExtensionDownloadAllowanceError::ApiUnavailable);
    }

    // SAFETY: `info` is borrowed from Apple's downloader-extension callback. The installed SDK
    // declares this property readonly and omits `nonatomic`, so its getter is atomic. The guard
    // above establishes the property's iOS 16.4 availability floor.
    unsafe { info.restrictedEssentialDownloadSizeRemaining() }
        .map(|remaining| {
            // Apple refines the NSNumber property to Swift `Int?`; its integer value is therefore
            // the documented signed native count. Preserve `nil` and reject negative byte counts.
            u64::try_from(remaining.integerValue())
                .map_err(|_| ExtensionDownloadAllowanceError::InvalidRemainingBytes)
        })
        .transpose()
}

pub(crate) fn inspect_asset_pack_manifest(
    json: &[u8],
    app_group_id: &str,
) -> Result<AssetPackManifestCount, AssetPackManifestError> {
    with_asset_pack_manifest(json, app_group_id, |manifest| {
        // SAFETY: `manifest` is a live instance from Apple's initializer. The header declares
        // `assetPacks` as a readonly copied NSSet; we read its count and retain no native object.
        let packs = unsafe { manifest.assetPacks() };
        u64::try_from(packs.count())
            .map(AssetPackManifestCount::new)
            .map_err(|_| AssetPackManifestError::CountOutOfRange)
    })
}

pub(crate) fn inspect_asset_pack_manifest_entries(
    json: &[u8],
    app_group_id: &str,
) -> Result<Vec<AssetPackManifestEntry>, AssetPackManifestError> {
    with_asset_pack_manifest(json, app_group_id, |manifest| {
        // SAFETY: `manifest` is a live instance from Apple's initializer. The header declares
        // `assetPacks` as a readonly copied NSSet; its contents cannot mutate during this call.
        let packs = unsafe { manifest.assetPacks() };
        let mut entries = Vec::with_capacity(packs.len());
        for pack in packs.iter() {
            // SAFETY: Each retained pack came from the parser's readonly manifest set. Apple's
            // header declares these properties readonly and atomic by default; the manifest is a
            // local JSON snapshot and no manager or server update can invalidate it during this
            // synchronous read. The NSData is copied to Rust-owned bytes before `pack` is dropped.
            let (identifier, native_download_size, version, user_info_json) = unsafe {
                (
                    pack.identifier().to_string(),
                    pack.downloadSize(),
                    pack.version(),
                    pack.userInfo().map(|data| data.to_vec()),
                )
            };
            let download_size_bytes = u64::try_from(native_download_size)
                .map_err(|_| AssetPackManifestError::InvalidDownloadSize)?;
            entries.push(AssetPackManifestEntry {
                identifier,
                download_size_bytes,
                version,
                user_info_json,
            });
        }
        entries.sort_unstable_by(|left, right| {
            left.identifier
                .cmp(&right.identifier)
                .then(left.version.cmp(&right.version))
                .then(left.download_size_bytes.cmp(&right.download_size_bytes))
        });
        Ok(entries)
    })
}

fn with_asset_pack_manifest<T>(
    json: &[u8],
    app_group_id: &str,
    inspect: impl FnOnce(&BAAssetPackManifest) -> Result<T, AssetPackManifestError>,
) -> Result<T, AssetPackManifestError> {
    if !objc2::available!(ios = 26.0, ..) {
        return Err(AssetPackManifestError::ApiUnavailable);
    }

    autoreleasepool(|_| {
        let data = NSData::with_bytes(json);
        let group_id = NSString::from_str(app_group_id);

        // SAFETY: The availability guard establishes the iOS 26.0 class floor. `alloc` creates
        // a valid uninitialized instance, and the generated initializer accepts live NSData and
        // NSString references and reports malformed manifests through NSError.
        let manifest = unsafe {
            BAAssetPackManifest::initFromData_applicationGroupIdentifier_error(
                BAAssetPackManifest::alloc(),
                &data,
                &group_id,
            )
        }
        .map_err(|error| {
            AssetPackManifestError::Native(NativeBackgroundAssetsError::new(
                error.domain().to_string(),
                error.code(),
            ))
        })?;

        inspect(&manifest)
    })
}

pub(crate) fn start(completion: Arc<Completion>) {
    start_queue_query(completion, QueueCountFilter::All);
}

pub(crate) fn start_essential(completion: Arc<Completion>) {
    start_queue_query(completion, QueueCountFilter::Essential);
}

pub(crate) fn start_nondefault_priority(completion: Arc<Completion>) {
    start_queue_query(completion, QueueCountFilter::NonDefaultPriority);
}

#[derive(Clone, Copy)]
enum QueueCountFilter {
    All,
    Essential,
    NonDefaultPriority,
}

fn start_queue_query(completion: Arc<Completion>, filter: QueueCountFilter) {
    if !objc2::available!(ios = 16.1, ..) {
        completion.complete(Err(DownloadQueueQueryError::ApiUnavailable));
        return;
    }
    if matches!(filter, QueueCountFilter::Essential) && !objc2::available!(ios = 16.4, ..) {
        completion.complete(Err(DownloadQueueQueryError::ApiUnavailable));
        return;
    }

    let callback_completion = Arc::clone(&completion);
    let callback_filter = filter;
    let handler = RcBlock::new(
        move |downloads: NonNull<NSArray<BADownload>>, native_error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| {
                    // SAFETY: The nullable NSError is borrowed only for this callback invocation.
                    if let Some(error) = unsafe { native_error.as_ref() } {
                        return Err(DownloadQueueQueryError::Native(
                            NativeBackgroundAssetsError::new(
                                error.domain().to_string(),
                                error.code(),
                            ),
                        ));
                    }

                    // SAFETY: Apple declares the returned downloads array nonnull on success.
                    // Its objects are read only during this callback. The iOS 16.4 guard covers
                    // `isEssential`; both filtered getters are readonly and atomic by default.
                    // Iteration retains each object for the getter call.
                    let queue = unsafe { downloads.as_ref() };
                    let count = match callback_filter {
                        QueueCountFilter::All => queue.count(),
                        QueueCountFilter::Essential => queue
                            .iter()
                            .filter(|download| {
                                // SAFETY: This retained object is from the native queue array;
                                // `isEssential` is a readonly atomic BOOL getter, and the outer
                                // availability guard establishes its iOS 16.4 floor.
                                unsafe { download.isEssential() }
                            })
                            .count(),
                        QueueCountFilter::NonDefaultPriority => queue
                            .iter()
                            .filter(|download| {
                                // SAFETY: This retained object is from the native queue array;
                                // `priority` is a readonly atomic NSInteger getter, and the outer
                                // availability guard establishes the iOS 16.1 floor for the
                                // property and its default-priority constant.
                                unsafe { download.priority() != BADownloaderPriorityDefault }
                            })
                            .count(),
                    };
                    u64::try_from(count).map_err(|_| DownloadQueueQueryError::CountOutOfRange)
                })
            }))
            .unwrap_or(Err(DownloadQueueQueryError::CallbackPanicked));
            callback_completion.complete(result);
        },
    );

    // SAFETY: The availability guard establishes the iOS 16.1 API floor. The retained singleton
    // remains alive through submission. The escaping callback captures only Arc<Completion>, so
    // its Rust state is Send + Sync; the native API copies and invokes the block asynchronously.
    let manager = unsafe { BADownloadManager::sharedManager() };
    // SAFETY: `handler` remains alive through submission and has only the sendable Rust-owned
    // completion capture required by the generated binding. The API returns the queue snapshot
    // asynchronously and the callback reads the array only during its invocation.
    unsafe { manager.fetchCurrentDownloadsWithCompletionHandler(&handler) };
}

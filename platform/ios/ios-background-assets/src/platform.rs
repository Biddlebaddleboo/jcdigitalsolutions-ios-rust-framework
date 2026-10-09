use core::ptr::NonNull;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use block2::RcBlock;
use objc2::{AnyThread, rc::autoreleasepool};
use objc2_background_assets::{BAAssetPackManifest, BADownload, BADownloadManager};
use objc2_foundation::{NSArray, NSData, NSError, NSString};

use crate::completion::Completion;
use crate::{
    AssetPackManifestCount, AssetPackManifestError, DownloadQueueQueryError,
    NativeBackgroundAssetsError,
};

pub(crate) fn inspect_asset_pack_manifest(
    json: &[u8],
    app_group_id: &str,
) -> Result<AssetPackManifestCount, AssetPackManifestError> {
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

        // SAFETY: `manifest` is a live instance from Apple's initializer. The header declares
        // `assetPacks` as a readonly copied NSSet; we read its count and retain no native object.
        let packs = unsafe { manifest.assetPacks() };
        u64::try_from(packs.count())
            .map(AssetPackManifestCount::new)
            .map_err(|_| AssetPackManifestError::CountOutOfRange)
    })
}

pub(crate) fn start(completion: Arc<Completion>) {
    if !objc2::available!(ios = 16.1, ..) {
        completion.complete(Err(DownloadQueueQueryError::ApiUnavailable));
        return;
    }

    let callback_completion = Arc::clone(&completion);
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
                    // The count is copied before the callback returns; no BADownload property is read.
                    let count = unsafe { downloads.as_ref() }.count();
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

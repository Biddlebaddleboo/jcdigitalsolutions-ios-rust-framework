#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Read-only unmanaged Background Assets download-queue snapshots for iOS"]

mod completion;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use objc2_background_assets::{BAAppExtensionInfo, BAContentRequest};

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;

use completion::Completion;

/// A fixed-width count from an unmanaged Background Assets queue snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DownloadQueueCount {
    count: u64,
}

impl DownloadQueueCount {
    /// Creates a queue-count snapshot value.
    pub const fn new(count: u64) -> Self {
        Self { count }
    }

    /// Returns the number of queue entries matched by the query's snapshot.
    pub const fn count(self) -> u64 {
        self.count
    }
}

/// An owned Background Assets `NSError` domain and native `NSInteger` code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBackgroundAssetsError {
    domain: String,
    code: isize,
}

impl NativeBackgroundAssetsError {
    /// Creates an owned native error value.
    pub fn new(domain: String, code: isize) -> Self {
        Self { domain, code }
    }

    /// Returns the native error domain.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the native `NSInteger` error code.
    pub const fn code(&self) -> isize {
        self.code
    }
}

/// A read-only Background Assets queue query error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DownloadQueueQueryError {
    /// The Background Assets query is available only on iOS.
    UnsupportedPlatform,
    /// The queue API is unavailable at the current iOS version.
    ApiUnavailable,
    /// The native query failed; its error domain and code are preserved.
    Native(NativeBackgroundAssetsError),
    /// The native queue length did not fit the fixed-width result.
    CountOutOfRange,
    /// The native callback encountered a Rust panic.
    CallbackPanicked,
}

/// A one-shot, caller-owned future for a queue-count snapshot.
#[must_use = "the future carries the requested Background Assets query result"]
pub struct DownloadQueueCountFuture {
    completion: Arc<Completion>,
    finished: bool,
}

impl DownloadQueueCountFuture {
    fn new(completion: Arc<Completion>) -> Self {
        Self {
            completion,
            finished: false,
        }
    }
}

impl Future for DownloadQueueCountFuture {
    type Output = Result<DownloadQueueCount, DownloadQueueQueryError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result.map(DownloadQueueCount::new))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for DownloadQueueCountFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

/// Starts a read-only snapshot of the calling unmanaged Background Assets download queue.
///
/// The native request starts when this function runs, not when the returned future is first
/// polled. Dropping the future abandons Rust interest but cannot cancel the native request. The
/// count is only the number of scheduled or in-flight queue entries returned at the callback's
/// observation point; it is not an installed-asset count or a Background Assets support query.
/// This API does not inspect download properties, schedule or cancel downloads, or validate host
/// app and downloader-extension configuration.
pub fn request_download_queue_count() -> DownloadQueueCountFuture {
    let completion = Arc::new(Completion::new());

    #[cfg(target_os = "ios")]
    platform::start(Arc::clone(&completion));

    #[cfg(not(target_os = "ios"))]
    completion.complete(Err(DownloadQueueQueryError::UnsupportedPlatform));

    DownloadQueueCountFuture::new(completion)
}

/// Starts a read-only count of essential downloads in the calling unmanaged download queue.
///
/// The native request starts when this function runs, not when the returned future is first
/// polled. The count includes only scheduled or in-flight entries returned by Apple's queue
/// snapshot whose `BADownload.isEssential` property is `true`. It does not count installed assets,
/// all essential assets on the device, or downloads outside the returned queue. Dropping the
/// future abandons Rust interest but cannot cancel the native request. Requires iOS 16.4 or later
/// and does not schedule or cancel downloads or validate host configuration.
pub fn request_essential_download_queue_count() -> DownloadQueueCountFuture {
    let completion = Arc::new(Completion::new());

    #[cfg(target_os = "ios")]
    platform::start_essential(Arc::clone(&completion));

    #[cfg(not(target_os = "ios"))]
    completion.complete(Err(DownloadQueueQueryError::UnsupportedPlatform));

    DownloadQueueCountFuture::new(completion)
}

/// Starts a read-only count of non-default-priority downloads in the unmanaged queue.
///
/// The native request starts when this function runs, not when the returned future is first
/// polled. The count includes only scheduled or in-flight entries returned by Apple's queue
/// snapshot whose `BADownload.priority` differs from `BADownloaderPriorityDefault`. It includes
/// priorities both above and below Apple's default and does not infer download progress or
/// installation state. Dropping the future abandons Rust interest but cannot cancel the native
/// request. Requires iOS 16.1 or later and does not schedule or cancel downloads.
pub fn request_nondefault_priority_download_queue_count() -> DownloadQueueCountFuture {
    let completion = Arc::new(Completion::new());

    #[cfg(target_os = "ios")]
    platform::start_nondefault_priority(Arc::clone(&completion));

    #[cfg(not(target_os = "ios"))]
    completion.complete(Err(DownloadQueueQueryError::UnsupportedPlatform));

    DownloadQueueCountFuture::new(completion)
}

/// A count of asset-pack entries in a caller-supplied unmanaged JSON manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetPackManifestCount {
    count: u64,
}

/// Owned metadata for one asset-pack entry from a caller-supplied manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetPackManifestEntry {
    identifier: String,
    download_size_bytes: u64,
    version: isize,
    user_info_json: Option<Vec<u8>>,
}

impl AssetPackManifestEntry {
    /// Returns the unique identifier from the parsed manifest.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Returns Apple's `NSInteger` download-size value in bytes.
    pub const fn download_size_bytes(&self) -> u64 {
        self.download_size_bytes
    }

    /// Returns Apple's `NSInteger` asset-pack version value.
    pub const fn version(&self) -> isize {
        self.version
    }

    /// Returns the optional raw JSON-encoded custom metadata bytes from the manifest.
    pub fn user_info_json_bytes(&self) -> Option<&[u8]> {
        self.user_info_json.as_deref()
    }
}

impl AssetPackManifestCount {
    /// Creates an asset-pack manifest count value.
    pub const fn new(count: u64) -> Self {
        Self { count }
    }

    /// Returns the number of asset packs declared in the parsed manifest.
    pub const fn count(self) -> u64 {
        self.count
    }
}

/// An asset-pack manifest inspection error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetPackManifestError {
    /// This manifest parser is available only on iOS.
    UnsupportedPlatform,
    /// The manifest parser is unavailable at the current iOS version.
    ApiUnavailable,
    /// The native parser failed; its error domain and code are preserved.
    Native(NativeBackgroundAssetsError),
    /// The native pack count did not fit the fixed-width result.
    CountOutOfRange,
    /// Apple's native download-size field was negative and could not represent a byte count.
    InvalidDownloadSize,
}

/// An error while reading a downloader extension's remaining download allowance.
#[cfg(target_os = "ios")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionDownloadAllowanceError {
    /// The allowance property is unavailable at the current iOS version.
    ApiUnavailable,
    /// Apple's native allowance value was negative and could not represent remaining bytes.
    InvalidRemainingBytes,
}

/// The known kind of downloader-extension content request.
#[cfg(target_os = "ios")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionContentRequestKind {
    /// Content requested because the app was installed.
    Install,
    /// Content requested because the app was updated.
    Update,
    /// Content requested during a system periodic update.
    Periodic,
    /// A native request value not named by the installed SDK.
    Unknown(isize),
}

/// Copies a typed callback reason into a Rust enum while preserving future native values.
///
/// Pass the `BAContentRequest` value supplied to Apple's `downloads(for:manifestURL:extensionInfo:)`
/// callback. The installed iOS 26.5 SDK names install, update, and periodic requests; unrecognized
/// native values are returned unchanged as `Unknown`. This is a value conversion only and does
/// not read the manifest or schedule downloads.
#[cfg(target_os = "ios")]
pub fn classify_extension_content_request(
    request: BAContentRequest,
) -> ExtensionContentRequestKind {
    match request {
        BAContentRequest::Install => ExtensionContentRequestKind::Install,
        BAContentRequest::Update => ExtensionContentRequestKind::Update,
        BAContentRequest::Periodic => ExtensionContentRequestKind::Periodic,
        _ => ExtensionContentRequestKind::Unknown(request.0),
    }
}

/// Copies the bytes remaining under the install-time download allowance.
///
/// Pass the `BAAppExtensionInfo` value supplied by Apple's
/// `downloads(for:manifestURL:extensionInfo:)` callback. The result is `None` when Apple reports
/// that downloads are unrestricted, and otherwise contains the remaining bytes before app launch.
/// This function does not inspect the manifest, schedule downloads, or construct extension info.
/// Requires iOS 16.1 or later.
#[cfg(target_os = "ios")]
pub fn inspect_restricted_download_size_remaining(
    info: &BAAppExtensionInfo,
) -> Result<Option<u64>, ExtensionDownloadAllowanceError> {
    platform::inspect_restricted_download_size_remaining(info)
}

/// Copies the bytes remaining under the essential download allowance.
///
/// Pass the `BAAppExtensionInfo` value supplied by Apple's
/// `downloads(for:manifestURL:extensionInfo:)` callback. The result is `None` when Apple reports
/// that downloads are unrestricted, and otherwise contains the remaining essential bytes before
/// app launch. This function does not inspect the manifest, schedule downloads, or construct
/// extension info. Requires iOS 16.4 or later.
#[cfg(target_os = "ios")]
pub fn inspect_restricted_essential_download_size_remaining(
    info: &BAAppExtensionInfo,
) -> Result<Option<u64>, ExtensionDownloadAllowanceError> {
    platform::inspect_restricted_essential_download_size_remaining(info)
}

/// Parses caller-supplied JSON as an unmanaged asset-pack manifest and returns its pack count.
///
/// The app-group identifier is passed to Apple's in-memory manifest initializer as metadata for
/// later unmanaged downloads. This call does not access `BAAssetPackManager`, query installed
/// assets, validate the app-group entitlement, or schedule a download. The result counts only the
/// manifest entries that Apple parsed; it does not report download or local-install status.
/// Requires iOS 26.0 or later. The input bytes and group identifier are copied for the native call.
pub fn inspect_asset_pack_manifest(
    json: &[u8],
    app_group_id: &str,
) -> Result<AssetPackManifestCount, AssetPackManifestError> {
    #[cfg(target_os = "ios")]
    {
        platform::inspect_asset_pack_manifest(json, app_group_id)
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (json, app_group_id);
        Err(AssetPackManifestError::UnsupportedPlatform)
    }
}

/// Parses caller-supplied JSON and returns owned metadata for each unmanaged asset-pack entry.
///
/// Entries are sorted by identifier, then version and download size. The result describes only
/// the supplied manifest; it does not report installed assets or current server state. Requires
/// iOS 26.0 or later and uses the same app-group identifier contract as
/// [`inspect_asset_pack_manifest`].
pub fn inspect_asset_pack_manifest_entries(
    json: &[u8],
    app_group_id: &str,
) -> Result<Vec<AssetPackManifestEntry>, AssetPackManifestError> {
    #[cfg(target_os = "ios")]
    {
        platform::inspect_asset_pack_manifest_entries(json, app_group_id)
    }

    #[cfg(not(target_os = "ios"))]
    {
        let _ = (json, app_group_id);
        Err(AssetPackManifestError::UnsupportedPlatform)
    }
}

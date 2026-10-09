#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Read-only unmanaged Background Assets download-queue snapshots for iOS"]

mod completion;

#[cfg(target_os = "ios")]
mod platform;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;

use completion::Completion;

/// A fixed-width snapshot of the returned unmanaged Background Assets queue length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DownloadQueueCount {
    count: u64,
}

impl DownloadQueueCount {
    /// Creates a queue-count snapshot value.
    pub const fn new(count: u64) -> Self {
        Self { count }
    }

    /// Returns the number of scheduled or in-flight downloads in the callback's snapshot.
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

/// A count of asset-pack entries in a caller-supplied unmanaged JSON manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssetPackManifestCount {
    count: u64,
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

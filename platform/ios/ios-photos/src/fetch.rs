use crate::operation::CompletionCell;
#[cfg(target_os = "ios")]
use crate::status::authorization_status_from_native;
use core::ffi::c_void;
#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::Pin;
use core::ptr::NonNull;
#[cfg(target_os = "ios")]
use core::task::{Context, Poll};
use framework_photos::PhotoLibraryAuthorizationStatus;
use std::sync::{Mutex, MutexGuard};

/// A bounded original-photo resource fetch failed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhotosFetchError {
    /// The PhotoKit local identifier is empty.
    EmptyLocalIdentifier,
    /// The PhotoKit local identifier contains a NUL byte.
    EmbeddedNulLocalIdentifier,
    /// The caller must supply a nonzero byte limit.
    ZeroMaximumByteCount,
    /// The current read/write status is neither Authorized nor Limited.
    ReadWriteNotAuthorized(PhotoLibraryAuthorizationStatus),
    /// The local identifier did not resolve to a Photos asset.
    AssetNotFound,
    /// One local identifier unexpectedly resolved to more than one asset.
    MultipleAssetsReturned,
    /// The asset is not an image.
    NotAnImageAsset,
    /// The image asset has no original-photo resource.
    OriginalPhotoResourceNotFound,
    /// Appending this chunk would exceed the caller's byte limit.
    MaximumByteCountExceeded {
        /// The maximum number of bytes allowed in the returned buffer.
        maximum: usize,
    },
    /// The accumulated byte count overflowed `usize`.
    ByteCountOverflow,
    /// A caller-controlled buffer or identifier allocation failed.
    AllocationFailed,
    /// PhotoKit returned a native error.
    PhotoKit {
        /// The native error domain.
        domain: String,
        /// The native error code.
        code: isize,
    },
    /// PhotoKit returned an invalid request identifier without an error.
    PhotoKitRequestRejected,
    /// A native PhotoKit callback panicked; the panic was contained at the block boundary.
    CallbackPanicked,
}

fn copy_local_identifier(identifier: &str) -> Result<String, PhotosFetchError> {
    if identifier.is_empty() {
        return Err(PhotosFetchError::EmptyLocalIdentifier);
    }
    if identifier.contains('\0') {
        return Err(PhotosFetchError::EmbeddedNulLocalIdentifier);
    }
    let mut owned = String::new();
    owned
        .try_reserve_exact(identifier.len())
        .map_err(|_| PhotosFetchError::AllocationFailed)?;
    owned.push_str(identifier);
    Ok(owned)
}

fn validate_fetch_arguments(
    identifier: &str,
    maximum_byte_count: usize,
) -> Result<String, PhotosFetchError> {
    if maximum_byte_count == 0 {
        return Err(PhotosFetchError::ZeroMaximumByteCount);
    }
    copy_local_identifier(identifier)
}

fn require_read_write_authorized(
    status: PhotoLibraryAuthorizationStatus,
) -> Result<(), PhotosFetchError> {
    if matches!(
        status,
        PhotoLibraryAuthorizationStatus::Authorized | PhotoLibraryAuthorizationStatus::Limited
    ) {
        Ok(())
    } else {
        Err(PhotosFetchError::ReadWriteNotAuthorized(status))
    }
}

fn select_original_photo_resource_index(
    resource_types: impl IntoIterator<Item = isize>,
    photo_type: isize,
) -> Option<usize> {
    resource_types
        .into_iter()
        .position(|resource_type| resource_type == photo_type)
}

fn checked_chunk_end(
    current: usize,
    chunk_length: usize,
    maximum: usize,
) -> Result<usize, PhotosFetchError> {
    let end = current
        .checked_add(chunk_length)
        .ok_or(PhotosFetchError::ByteCountOverflow)?;
    if end > maximum {
        return Err(PhotosFetchError::MaximumByteCountExceeded { maximum });
    }
    Ok(end)
}

fn reserve_chunk(buffer: &mut Vec<u8>, additional: usize) -> Result<(), PhotosFetchError> {
    buffer
        .try_reserve_exact(additional)
        .map_err(|_| PhotosFetchError::AllocationFailed)
}

type DataRequestId = i32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestPhase {
    Active,
    Completed,
    Failed,
    Cancelled,
}

struct RequestState {
    phase: RequestPhase,
    bytes: Vec<u8>,
    request_id: Option<DataRequestId>,
}

struct PhotoFetchOperation {
    maximum_byte_count: usize,
    state: Mutex<RequestState>,
    completion: CompletionCell<Result<Vec<u8>, PhotosFetchError>>,
}

enum AppendOutcome {
    Appended,
    Ignored,
    Failed(Option<DataRequestId>),
}

impl PhotoFetchOperation {
    fn new(maximum_byte_count: usize) -> Self {
        Self {
            maximum_byte_count,
            state: Mutex::new(RequestState {
                phase: RequestPhase::Active,
                bytes: Vec::new(),
                request_id: None,
            }),
            completion: CompletionCell::new(),
        }
    }

    #[cfg(test)]
    fn append_bytes(&self, bytes: &[u8]) -> AppendOutcome {
        self.append_with(bytes.len(), |destination, length| {
            if length != 0 {
                // SAFETY: `append_with` reserved `length` writable bytes at `destination`; the
                // borrowed fixture remains valid for this call and does not overlap the Vec.
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        destination.cast::<u8>().as_ptr(),
                        length,
                    )
                };
            }
        })
    }

    fn append_with(
        &self,
        chunk_length: usize,
        write: impl FnOnce(NonNull<c_void>, usize),
    ) -> AppendOutcome {
        let failure = {
            let mut state = self.lock();
            if state.phase != RequestPhase::Active {
                return AppendOutcome::Ignored;
            }
            let checked =
                checked_chunk_end(state.bytes.len(), chunk_length, self.maximum_byte_count);
            let failure = match checked {
                Err(error) => Some(error),
                Ok(end) => match reserve_chunk(&mut state.bytes, chunk_length) {
                    Err(error) => Some(error),
                    Ok(()) => {
                        if chunk_length != 0 {
                            let previous_length = state.bytes.len();
                            // SAFETY: fallible reservation above guarantees writable capacity
                            // through `end`; `previous_length` is in the initialized Vec prefix.
                            let destination = unsafe {
                                NonNull::new_unchecked(
                                    state
                                        .bytes
                                        .as_mut_ptr()
                                        .add(previous_length)
                                        .cast::<c_void>(),
                                )
                            };
                            write(destination, chunk_length);
                            // SAFETY: the writer initialized exactly `chunk_length` bytes.
                            unsafe { state.bytes.set_len(end) };
                        }
                        None
                    }
                },
            };
            if let Some(error) = failure {
                state.phase = RequestPhase::Failed;
                let request_id = state.request_id.take();
                let retired_bytes = core::mem::take(&mut state.bytes);
                Some((error, request_id, retired_bytes))
            } else {
                None
            }
        };

        match failure {
            None => AppendOutcome::Appended,
            Some((error, request_id, retired_bytes)) => {
                drop(retired_bytes);
                self.completion.complete(Err(error));
                AppendOutcome::Failed(request_id)
            }
        }
    }

    #[cfg(target_os = "ios")]
    fn append_nsdata(&self, data: &objc2_foundation::NSData) -> AppendOutcome {
        let length = data.length();
        self.append_with(length, |destination, length| {
            // SAFETY: `append_with` reserved writable space and the NSData remains valid for this
            // callback. PhotoKit documents chunk bytes as valid only during this handler.
            unsafe { data.getBytes_length(destination, length) };
        })
    }

    fn install_request_id(&self, request_id: DataRequestId) -> bool {
        let mut state = self.lock();
        match state.phase {
            RequestPhase::Active => {
                state.request_id = Some(request_id);
                false
            }
            RequestPhase::Failed | RequestPhase::Cancelled => true,
            RequestPhase::Completed => false,
        }
    }

    fn finish(&self, result: Result<(), PhotosFetchError>) -> bool {
        let (result, retired_bytes) = {
            let mut state = self.lock();
            if state.phase != RequestPhase::Active {
                return false;
            }
            state.request_id.take();
            match result {
                Ok(()) => {
                    state.phase = RequestPhase::Completed;
                    (Ok(core::mem::take(&mut state.bytes)), Vec::new())
                }
                Err(error) => {
                    state.phase = RequestPhase::Failed;
                    (Err(error), core::mem::take(&mut state.bytes))
                }
            }
        };
        drop(retired_bytes);
        self.completion.complete(result)
    }

    fn fail(&self, error: PhotosFetchError) -> Option<DataRequestId> {
        let (request_id, retired_bytes) = {
            let mut state = self.lock();
            if state.phase != RequestPhase::Active {
                return None;
            }
            state.phase = RequestPhase::Failed;
            (state.request_id.take(), core::mem::take(&mut state.bytes))
        };
        drop(retired_bytes);
        self.completion.complete(Err(error));
        request_id
    }

    fn detach_and_cancel(&self) -> Option<DataRequestId> {
        self.completion.detach();
        let (request_id, retired_bytes) = {
            let mut state = self.lock();
            if state.phase != RequestPhase::Active {
                return None;
            }
            state.phase = RequestPhase::Cancelled;
            (state.request_id.take(), core::mem::take(&mut state.bytes))
        };
        drop(retired_bytes);
        request_id
    }

    fn lock(&self) -> MutexGuard<'_, RequestState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(target_os = "ios")]
use block2::RcBlock;
#[cfg(target_os = "ios")]
use objc2::rc::Retained;
#[cfg(target_os = "ios")]
use objc2_foundation::{NSArray, NSData, NSError, NSString};
#[cfg(target_os = "ios")]
use objc2_photos::{
    PHAccessLevel, PHAsset, PHAssetMediaType, PHAssetResource, PHAssetResourceDataRequestID,
    PHAssetResourceManager, PHAssetResourceRequestOptions, PHAssetResourceType,
    PHInvalidAssetResourceDataRequestID, PHPhotoLibrary,
};
#[cfg(target_os = "ios")]
use std::panic::{AssertUnwindSafe, catch_unwind};

#[cfg(target_os = "ios")]
use crate::platform::IosPhotosBackend;

#[cfg(target_os = "ios")]
impl IosPhotosBackend {
    /// Fetches the original still-photo bytes for one local identifier.
    ///
    /// This method never requests authorization and never enumerates the library. Current
    /// read/write authorization must be [`PhotoLibraryAuthorizationStatus::Authorized`] or
    /// [`PhotoLibraryAuthorizationStatus::Limited`]. The caller must set a nonzero byte cap and
    /// explicitly choose whether PhotoKit may fetch the resource from the network. The future
    /// owns the returned bytes; dropping it cancels the PhotoKit request when possible. A dropped
    /// future does not wait for an in-flight native callback.
    pub fn fetch_original_photo_bytes(
        &self,
        local_identifier: &str,
        maximum_byte_count: usize,
        allow_network_access: bool,
    ) -> Result<IosPhotosOriginalPhotoFuture, PhotosFetchError> {
        IosPhotosOriginalPhotoFuture::new(
            local_identifier,
            maximum_byte_count,
            allow_network_access,
        )
    }
}

#[cfg(target_os = "ios")]
/// A lazy fetch of one original still-photo resource, capped by the caller.
pub struct IosPhotosOriginalPhotoFuture {
    local_identifier: Option<String>,
    allow_network_access: bool,
    operation: std::sync::Arc<PhotoFetchOperation>,
    manager: Option<Retained<PHAssetResourceManager>>,
    started: bool,
    finished: bool,
}

#[cfg(target_os = "ios")]
impl IosPhotosOriginalPhotoFuture {
    fn new(
        local_identifier: &str,
        maximum_byte_count: usize,
        allow_network_access: bool,
    ) -> Result<Self, PhotosFetchError> {
        let local_identifier = validate_fetch_arguments(local_identifier, maximum_byte_count)?;
        Ok(Self {
            local_identifier: Some(local_identifier),
            allow_network_access,
            operation: std::sync::Arc::new(PhotoFetchOperation::new(maximum_byte_count)),
            manager: None,
            started: false,
            finished: false,
        })
    }

    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;

        // SAFETY: This public PhotoKit query reads the current explicit iOS 14+ ReadWrite state
        // without prompting. Only Authorized and Limited permit this fetch.
        let status =
            unsafe { PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite) };
        let status = authorization_status_from_native(status.0);
        if let Err(error) = require_read_write_authorized(status) {
            self.operation.finish(Err(error));
            return;
        }

        let local_identifier = self
            .local_identifier
            .take()
            .expect("PhotoKit fetch starts once");
        let identifier = NSString::from_str(&local_identifier);
        let identifiers = NSArray::from_retained_slice(core::slice::from_ref(&identifier));
        // SAFETY: The array contains exactly one validated caller-supplied local identifier.
        let assets =
            unsafe { PHAsset::fetchAssetsWithLocalIdentifiers_options(&identifiers, None) };
        // SAFETY: `count` is a read-only query on the returned fetch result.
        let asset_count = unsafe { assets.count() };
        if asset_count == 0 {
            self.operation.finish(Err(PhotosFetchError::AssetNotFound));
            return;
        }
        if asset_count != 1 {
            self.operation
                .finish(Err(PhotosFetchError::MultipleAssetsReturned));
            return;
        }
        // SAFETY: The preceding count check proves index zero is in range.
        let asset = unsafe { assets.objectAtIndex(0) };
        // SAFETY: The fetched asset remains retained while its public media type is inspected.
        if unsafe { asset.mediaType() } != PHAssetMediaType::Image {
            self.operation
                .finish(Err(PhotosFetchError::NotAnImageAsset));
            return;
        }

        // SAFETY: PhotoKit returns the public resource list for this single image asset.
        let resources = unsafe { PHAssetResource::assetResourcesForAsset(&asset) };
        let resource_count = resources.count();
        let resource_index = select_original_photo_resource_index(
            (0..resource_count).map(|index| {
                // SAFETY: `index` is yielded from the checked range below `resource_count`.
                unsafe { resources.objectAtIndex(index).r#type().0 }
            }),
            PHAssetResourceType::Photo.0,
        );
        let Some(resource_index) = resource_index else {
            self.operation
                .finish(Err(PhotosFetchError::OriginalPhotoResourceNotFound));
            return;
        };
        // SAFETY: The selected resource index came from this array's checked range.
        let resource = resources.objectAtIndex(resource_index);

        // SAFETY: This is PhotoKit's public resource data manager and request-options initializer.
        let manager = unsafe { PHAssetResourceManager::defaultManager() };
        let options = unsafe { PHAssetResourceRequestOptions::new() };
        // SAFETY: The request option is configured before being passed to PhotoKit.
        unsafe { options.setNetworkAccessAllowed(self.allow_network_access) };
        self.manager = Some(manager.clone());

        let operation_for_data = std::sync::Arc::clone(&self.operation);
        let operation_for_data_panic = std::sync::Arc::clone(&self.operation);
        let manager_for_data = manager.clone();
        let data_handler = RcBlock::new(move |data: NonNull<NSData>| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: PhotoKit guarantees this non-null NSData only for this callback.
                let data = unsafe { data.as_ref() };
                match operation_for_data.append_nsdata(data) {
                    AppendOutcome::Appended | AppendOutcome::Ignored => {}
                    AppendOutcome::Failed(request_id) => {
                        cancel_data_request(&manager_for_data, request_id)
                    }
                }
            }));
            if result.is_err() {
                let request_id = operation_for_data_panic.fail(PhotosFetchError::CallbackPanicked);
                cancel_data_request(&manager_for_data, request_id);
            }
        });

        let operation_for_completion = std::sync::Arc::clone(&self.operation);
        let operation_for_completion_panic = std::sync::Arc::clone(&self.operation);
        let manager_for_completion = manager.clone();
        let completion_handler = RcBlock::new(move |error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                let result = if error.is_null() {
                    Ok(())
                } else {
                    // SAFETY: PhotoKit guarantees the nullable NSError is valid for this callback.
                    let error = unsafe { &*error };
                    Err(PhotosFetchError::PhotoKit {
                        domain: error.domain().to_string(),
                        code: error.code(),
                    })
                };
                operation_for_completion.finish(result);
            }));
            if result.is_err() {
                let request_id =
                    operation_for_completion_panic.fail(PhotosFetchError::CallbackPanicked);
                cancel_data_request(&manager_for_completion, request_id);
            }
        });

        // SAFETY: PhotoKit copies both blocks and retains request inputs for the asynchronous
        // operation. The data block copies each callback-scoped NSData chunk before returning.
        let request_id = unsafe {
            manager.requestDataForAssetResource_options_dataReceivedHandler_completionHandler(
                &resource,
                Some(&options),
                &data_handler,
                &completion_handler,
            )
        };
        if request_id == PHInvalidAssetResourceDataRequestID {
            let request_id = self
                .operation
                .fail(PhotosFetchError::PhotoKitRequestRejected);
            cancel_data_request(&manager, request_id);
        } else if self.operation.install_request_id(request_id) {
            // A data/completion callback may have failed or future cancellation may have won
            // before the native call returned its ID. Recheck terminal state before proceeding.
            cancel_data_request(&manager, Some(request_id));
        }
    }
}

#[cfg(target_os = "ios")]
impl Future for IosPhotosOriginalPhotoFuture {
    type Output = Result<Vec<u8>, PhotosFetchError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Pending;
        }
        this.start();
        match this.operation.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosPhotosOriginalPhotoFuture {
    fn drop(&mut self) {
        let request_id = self.operation.detach_and_cancel();
        if let Some(manager) = &self.manager {
            cancel_data_request(manager, request_id);
        }
    }
}

#[cfg(target_os = "ios")]
fn cancel_data_request(
    manager: &objc2_photos::PHAssetResourceManager,
    request_id: Option<PHAssetResourceDataRequestID>,
) {
    if let Some(request_id) =
        request_id.filter(|request_id| *request_id != PHInvalidAssetResourceDataRequestID)
    {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: This is a request ID returned by this same public manager.
            unsafe { manager.cancelDataRequest(request_id) };
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppendOutcome, PhotoFetchOperation, PhotosFetchError, RequestPhase, checked_chunk_end,
        require_read_write_authorized, reserve_chunk, select_original_photo_resource_index,
        validate_fetch_arguments,
    };
    use framework_photos::PhotoLibraryAuthorizationStatus;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn with_context<T>(poll: impl FnOnce(&mut Context<'_>) -> T) -> T {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        poll(&mut context)
    }

    #[test]
    fn fetch_allows_only_readwrite_authorized_or_limited() {
        for status in [
            PhotoLibraryAuthorizationStatus::Authorized,
            PhotoLibraryAuthorizationStatus::Limited,
        ] {
            assert_eq!(require_read_write_authorized(status), Ok(()));
        }
        for status in [
            PhotoLibraryAuthorizationStatus::NotDetermined,
            PhotoLibraryAuthorizationStatus::Restricted,
            PhotoLibraryAuthorizationStatus::Denied,
            PhotoLibraryAuthorizationStatus::Unknown,
        ] {
            assert_eq!(
                require_read_write_authorized(status),
                Err(PhotosFetchError::ReadWriteNotAuthorized(status))
            );
        }
    }

    #[test]
    fn local_identifier_and_limit_are_checked_before_native_work() {
        assert_eq!(
            validate_fetch_arguments("asset-id", 0),
            Err(PhotosFetchError::ZeroMaximumByteCount)
        );
        assert_eq!(
            validate_fetch_arguments("", 1),
            Err(PhotosFetchError::EmptyLocalIdentifier)
        );
        assert_eq!(
            validate_fetch_arguments("asset\0id", 1),
            Err(PhotosFetchError::EmbeddedNulLocalIdentifier)
        );
        assert_eq!(validate_fetch_arguments("asset-id", 1).unwrap(), "asset-id");
    }

    #[test]
    fn original_resource_selection_uses_only_the_photo_type() {
        assert_eq!(
            select_original_photo_resource_index([5, 2, 1, 9], 1),
            Some(2)
        );
        assert_eq!(select_original_photo_resource_index([5, 2, 9], 1), None);
    }

    #[test]
    fn chunk_cap_overflow_and_allocation_errors_fail_closed() {
        assert_eq!(
            checked_chunk_end(2, 2, 3),
            Err(PhotosFetchError::MaximumByteCountExceeded { maximum: 3 })
        );
        assert_eq!(
            checked_chunk_end(usize::MAX, 1, usize::MAX),
            Err(PhotosFetchError::ByteCountOverflow)
        );
        assert_eq!(
            reserve_chunk(&mut Vec::new(), usize::MAX),
            Err(PhotosFetchError::AllocationFailed)
        );

        let operation = PhotoFetchOperation::new(3);
        assert!(!operation.install_request_id(7));
        assert!(matches!(
            operation.append_bytes(b"ab"),
            AppendOutcome::Appended
        ));
        assert!(matches!(
            operation.append_bytes(b"cd"),
            AppendOutcome::Failed(Some(7))
        ));
        with_context(|context| {
            assert_eq!(
                operation.completion.poll(context),
                Poll::Ready(Err(PhotosFetchError::MaximumByteCountExceeded {
                    maximum: 3
                }))
            )
        });
        assert_eq!(operation.lock().phase, RequestPhase::Failed);
        assert!(matches!(
            operation.append_bytes(b"late"),
            AppendOutcome::Ignored
        ));
    }

    #[test]
    fn first_terminal_event_wins_and_cancellation_suppresses_late_chunks() {
        let cancelled = PhotoFetchOperation::new(8);
        assert!(!cancelled.install_request_id(11));
        assert_eq!(cancelled.detach_and_cancel(), Some(11));
        assert_eq!(cancelled.lock().phase, RequestPhase::Cancelled);
        assert!(matches!(
            cancelled.append_bytes(b"late"),
            AppendOutcome::Ignored
        ));
        assert!(!cancelled.finish(Ok(())));
        with_context(|context| {
            assert!(matches!(cancelled.completion.poll(context), Poll::Pending))
        });

        let failed_before_request_id = PhotoFetchOperation::new(8);
        assert_eq!(
            failed_before_request_id.fail(PhotosFetchError::CallbackPanicked),
            None
        );
        assert!(failed_before_request_id.install_request_id(12));
        assert_eq!(failed_before_request_id.lock().phase, RequestPhase::Failed);

        let completed = PhotoFetchOperation::new(8);
        assert!(matches!(
            completed.append_bytes(b"ok"),
            AppendOutcome::Appended
        ));
        assert!(completed.finish(Ok(())));
        with_context(|context| {
            assert_eq!(
                completed.completion.poll(context),
                Poll::Ready(Ok(b"ok".to_vec()))
            )
        });
        assert_eq!(completed.detach_and_cancel(), None);
    }
}

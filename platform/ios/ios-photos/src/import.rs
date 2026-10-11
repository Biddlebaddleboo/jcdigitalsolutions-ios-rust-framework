use crate::operation::CompletionCell;
use crate::platform::IosPhotosBackend;
use crate::save::{
    PhotosAssetFileKind, PhotosSaveError, require_add_only_authorized, validated_source_path,
};
use crate::status::add_only_authorization_status_from_native;
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString, NSURL};
use objc2_photos::{
    PHAccessLevel, PHAssetCreationRequest, PHAssetResourceCreationOptions, PHAssetResourceType,
    PHPhotoLibrary,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

impl IosPhotosBackend {
    /// Saves a caller-owned local JPEG as a new Photos asset using add-only authorization.
    ///
    /// This method never prompts. Call
    /// [`framework_photos::PhotoLibrary::request_add_only_authorization`] through the portable
    /// facade after a user action when access must be requested. The source must be
    /// a readable `.jpg` or `.jpeg` regular file and must remain unchanged until the future
    /// completes. PhotoKit validates the file contents; the extension is not content proof. The
    /// source is not moved or deleted. A security-scoped source requires caller-managed access
    /// that remains open through completion.
    pub fn save_jpeg_from_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<IosPhotosSaveFuture, PhotosSaveError> {
        IosPhotosSaveFuture::new(path.as_ref(), PhotosAssetFileKind::Jpeg)
    }

    /// Saves a caller-owned finalized movie as a new Photos asset using add-only authorization.
    ///
    /// This method never prompts. Call
    /// [`framework_photos::PhotoLibrary::request_add_only_authorization`] through the portable
    /// facade after a user action when access must be requested. The source must be
    /// a readable `.mov`, `.mp4`, or `.m4v` regular file and must remain unchanged until the future
    /// completes. PhotoKit validates the file contents; the extension is not content proof. The
    /// source is not moved or deleted. A security-scoped source requires caller-managed access
    /// that remains open through completion.
    pub fn save_movie_from_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<IosPhotosSaveFuture, PhotosSaveError> {
        IosPhotosSaveFuture::new(path.as_ref(), PhotosAssetFileKind::Movie)
    }
}

/// An asynchronous, add-only PhotoKit save for one caller-owned local file.
pub struct IosPhotosSaveFuture {
    file_url: Option<Retained<NSURL>>,
    resource_type: PHAssetResourceType,
    completion: Arc<CompletionCell<Result<(), PhotosSaveError>>>,
    started: bool,
    finished: bool,
}

impl IosPhotosSaveFuture {
    pub(crate) fn new(path: &Path, kind: PhotosAssetFileKind) -> Result<Self, PhotosSaveError> {
        let path = validated_source_path(path, kind).map_err(PhotosSaveError::SourcePath)?;
        let path = NSString::from_str(&path);
        let file_url = NSURL::fileURLWithPath(&path);
        let resource_type = match kind {
            PhotosAssetFileKind::Jpeg => PHAssetResourceType::Photo,
            PhotosAssetFileKind::Movie => PHAssetResourceType::Video,
        };
        Ok(Self {
            file_url: Some(file_url),
            resource_type,
            completion: Arc::new(CompletionCell::new()),
            started: false,
            finished: false,
        })
    }

    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        // SAFETY: This is the public iOS 14+ status query for the explicit add-only access level.
        let status =
            unsafe { PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::AddOnly) };
        let status = add_only_authorization_status_from_native(status.0);
        if let Err(error) = require_add_only_authorized(status) {
            self.completion.complete(Err(error));
            return;
        }

        let file_url = self.file_url.take().expect("save future starts once");
        let resource_type = self.resource_type;
        let change_block_panicked = Arc::new(AtomicBool::new(false));
        let change_block_panicked_in_change = Arc::clone(&change_block_panicked);
        let change_block = RcBlock::new(move || {
            let result = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: PhotoKit invokes this block inside `performChanges`; creation requests
                // and resource changes are valid only within that transaction block.
                let request = unsafe { PHAssetCreationRequest::creationRequestForAsset() };
                // SAFETY: This public options initializer returns a retained Objective-C object.
                let options = unsafe { PHAssetResourceCreationOptions::new() };
                // SAFETY: Keep the caller's file in place; PhotoKit reads it as a new resource.
                unsafe { options.setShouldMoveFile(false) };
                // SAFETY: `file_url` remains retained by this change block through the transaction.
                unsafe {
                    request.addResourceWithType_fileURL_options(
                        resource_type,
                        &file_url,
                        Some(&options),
                    )
                };
            }));
            if result.is_err() {
                change_block_panicked_in_change.store(true, Ordering::Release);
            }
        });

        let completion = Arc::clone(&self.completion);
        let completion_handler = RcBlock::new(move |success: Bool, error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                if change_block_panicked.load(Ordering::Acquire) {
                    return Err(PhotosSaveError::ChangeBlockPanicked);
                }
                if success.as_bool() {
                    return Ok(());
                }
                if error.is_null() {
                    return Err(PhotosSaveError::PhotoKitFailedWithoutError);
                }
                // SAFETY: PhotoKit supplies a valid error object for the duration of this callback.
                let error = unsafe { &*error };
                Err(PhotosSaveError::PhotoKit {
                    domain: error.domain().to_string(),
                    code: error.code(),
                })
            }))
            .unwrap_or(Err(PhotosSaveError::PhotoKitFailedWithoutError));
            completion.complete(result);
        });

        // SAFETY: The change block is valid for the call and PhotoKit copies it for the
        // asynchronous transaction. The completion block captures only owned synchronized state.
        let library = unsafe { PHPhotoLibrary::sharedPhotoLibrary() };
        unsafe {
            library.performChanges_completionHandler(
                RcBlock::as_ptr(&change_block),
                Some(&completion_handler),
            )
        };
    }
}

impl Future for IosPhotosSaveFuture {
    type Output = Result<(), PhotosSaveError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Pending;
        }
        this.start();
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosPhotosSaveFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

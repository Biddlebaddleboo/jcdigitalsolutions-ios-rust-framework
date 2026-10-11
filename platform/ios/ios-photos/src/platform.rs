use crate::operation::CompletionCell;
use crate::status::{add_only_authorization_status_from_native, authorization_status_from_native};
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_photos::{
    PhotoLibraryAddOnlyAuthorizationBackend, PhotoLibraryAddOnlyAuthorizationStatus,
    PhotoLibraryAuthorizationBackend, PhotoLibraryAuthorizationStatus,
};
use objc2_photos::{PHAccessLevel, PHAuthorizationStatus, PHPhotoLibrary};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

/// A stateless PhotoKit backend for explicit read/write authorization queries and requests.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosPhotosBackend;

/// A lazy PhotoKit read/write authorization request.
pub struct IosPhotosAuthorizationFuture {
    completion: Arc<CompletionCell<PhotoLibraryAuthorizationStatus>>,
    started: bool,
    finished: bool,
}

/// A lazy PhotoKit add-only authorization request.
pub struct IosPhotosAddOnlyAuthorizationFuture {
    completion: Arc<CompletionCell<PhotoLibraryAddOnlyAuthorizationStatus>>,
    started: bool,
    finished: bool,
}

impl PhotoLibraryAuthorizationBackend for IosPhotosBackend {
    type RequestAuthorizationFuture<'a>
        = IosPhotosAuthorizationFuture
    where
        Self: 'a;

    fn authorization_status(&self) -> PhotoLibraryAuthorizationStatus {
        // SAFETY: This public PhotoKit query accepts the explicit iOS 14+ read/write access level.
        let status =
            unsafe { PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite) };
        authorization_status_from_native(status.0)
    }

    fn request_authorization(&mut self) -> Self::RequestAuthorizationFuture<'_> {
        IosPhotosAuthorizationFuture {
            completion: Arc::new(CompletionCell::new()),
            started: false,
            finished: false,
        }
    }
}

impl PhotoLibraryAddOnlyAuthorizationBackend for IosPhotosBackend {
    type RequestAddOnlyAuthorizationFuture<'a> = IosPhotosAddOnlyAuthorizationFuture;

    fn add_only_authorization_status(&self) -> PhotoLibraryAddOnlyAuthorizationStatus {
        // SAFETY: This public PhotoKit query accepts the explicit iOS 14+ add-only access level.
        let status =
            unsafe { PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::AddOnly) };
        add_only_authorization_status_from_native(status.0)
    }

    fn request_add_only_authorization(&mut self) -> Self::RequestAddOnlyAuthorizationFuture<'_> {
        IosPhotosAddOnlyAuthorizationFuture {
            completion: Arc::new(CompletionCell::new()),
            started: false,
            finished: false,
        }
    }
}

impl IosPhotosAuthorizationFuture {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let completion = Arc::clone(&self.completion);
        let handler = RcBlock::new(move |status: PHAuthorizationStatus| {
            let mapped = catch_unwind(AssertUnwindSafe(|| {
                authorization_status_from_native(status.0)
            }))
            .unwrap_or(PhotoLibraryAuthorizationStatus::Unknown);
            completion.complete(mapped);
        });
        // SAFETY: The access-level request is a public iOS 14+ PhotoKit API. PhotoKit retains
        // its completion block for the asynchronous callback; the block captures only Arc state.
        unsafe {
            PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(
                PHAccessLevel::ReadWrite,
                &handler,
            )
        };
    }
}

impl Future for IosPhotosAuthorizationFuture {
    type Output = PhotoLibraryAuthorizationStatus;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Pending;
        }
        this.start();
        match this.completion.poll(context) {
            Poll::Ready(status) => {
                this.finished = true;
                Poll::Ready(status)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosPhotosAuthorizationFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

impl IosPhotosAddOnlyAuthorizationFuture {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let completion = Arc::clone(&self.completion);
        let handler = RcBlock::new(move |status: PHAuthorizationStatus| {
            let mapped = catch_unwind(AssertUnwindSafe(|| {
                add_only_authorization_status_from_native(status.0)
            }))
            .unwrap_or(PhotoLibraryAddOnlyAuthorizationStatus::Unknown);
            completion.complete(mapped);
        });
        // SAFETY: The access-level request is a public iOS 14+ PhotoKit API. PhotoKit retains
        // its completion block for the asynchronous callback; the block captures only Arc state.
        unsafe {
            PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(
                PHAccessLevel::AddOnly,
                &handler,
            )
        };
    }
}

impl Future for IosPhotosAddOnlyAuthorizationFuture {
    type Output = PhotoLibraryAddOnlyAuthorizationStatus;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.finished {
            return Poll::Pending;
        }
        this.start();
        match this.completion.poll(context) {
            Poll::Ready(status) => {
                this.finished = true;
                Poll::Ready(status)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosPhotosAddOnlyAuthorizationFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

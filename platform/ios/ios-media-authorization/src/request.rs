use crate::operation::CompletionCell;
use crate::platform::IosMediaAuthorization;
use crate::status::{AvMediaType, av_media_type};
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_media_authorization::{CaptureMedia, MediaAuthorization, MediaAuthorizationStatus};
use objc2_av_foundation::{AVCaptureDevice, AVMediaTypeAudio, AVMediaTypeVideo};
use std::sync::Arc;

/// A lazy AVFoundation camera/microphone authorization request.
pub struct IosMediaAuthorizationFuture {
    completion: Arc<CompletionCell>,
    media: CaptureMedia,
    started: bool,
    finished: bool,
}

impl IosMediaAuthorizationFuture {
    pub(crate) fn new(media: CaptureMedia) -> Self {
        Self {
            completion: Arc::new(CompletionCell::new()),
            media,
            started: false,
            finished: false,
        }
    }

    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;

        let Some(media_type) = av_media_type(self.media) else {
            self.completion.complete(MediaAuthorizationStatus::Unknown);
            return;
        };
        // SAFETY: These immutable AVFoundation constants are exactly the accepted camera and
        // microphone values. No caller-provided media-type string reaches the native method.
        let media_type = unsafe {
            match media_type {
                AvMediaType::Video => AVMediaTypeVideo,
                AvMediaType::Audio => AVMediaTypeAudio,
            }
        };
        let Some(media_type) = media_type else {
            self.completion.complete(MediaAuthorizationStatus::Unknown);
            return;
        };

        let completion = Arc::clone(&self.completion);
        let media = self.media;
        let handler = RcBlock::new(move |_| {
            completion
                .complete_from_callback(|| IosMediaAuthorization::authorization_status(media));
        });
        // SAFETY: This public iOS 7+ AVFoundation API accepts only AVMediaTypeVideo or
        // AVMediaTypeAudio. The native method copies the escaping completion block and invokes it
        // asynchronously on an arbitrary queue; it does not create a capture session or device.
        unsafe {
            AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &handler)
        };
    }
}

impl Future for IosMediaAuthorizationFuture {
    type Output = MediaAuthorizationStatus;

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

impl Drop for IosMediaAuthorizationFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

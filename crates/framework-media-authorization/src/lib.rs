#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable camera and microphone authorization contracts."]
#![doc = ""]
#![doc = "Use [`MediaAuthorization`] for non-prompting status queries and [`MediaAuthorizationRequester`] for explicit authorization requests."]

#[cfg(test)]
extern crate std;

use core::future::Future;

/// A capture-media category whose authorization status can be queried.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CaptureMedia {
    /// Camera capture permission.
    Camera,
    /// Microphone recording permission.
    Microphone,
}

/// A snapshot of the operating system's authorization status for capture media.
///
/// `Authorized` reports permission state only. It does not establish that suitable hardware is
/// present, available, or ready for capture. Query again when current state matters because the
/// user or operating system may change the status outside this process.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MediaAuthorizationStatus {
    /// The user has not yet made an authorization choice.
    NotDetermined,
    /// System policy prevents access and the user cannot change the status in the app.
    Restricted,
    /// Access is not authorized.
    Denied,
    /// The operating system reports that access is authorized.
    Authorized,
    /// The backend returned a status value it does not recognize.
    Unknown,
}

/// A platform implementation of camera/microphone authorization status queries.
///
/// This static query must not request authorization or start a capture or audio session.
pub trait MediaAuthorization {
    /// Reads the current authorization status for the selected media category.
    fn authorization_status(media: CaptureMedia) -> MediaAuthorizationStatus;
}

/// A platform implementation of explicit camera/microphone authorization requests.
///
/// The returned future must defer prompt-capable native work until its first poll. Dropping an
/// unpolled future starts no native work. Dropping a pending future abandons its Rust result but
/// does not promise cancellation or dismissal of authorization UI already requested. Completing
/// a request reports authorization status only; it does not start capture or audio work. No
/// executor or `Send` requirement is imposed.
pub trait MediaAuthorizationRequester {
    /// The future returned by one explicit authorization request.
    type RequestAuthorizationFuture<'a>: Future<Output = MediaAuthorizationStatus> + 'a
    where
        Self: 'a;

    /// Requests authorization for the selected media category.
    ///
    /// Prompt-capable native work begins only when the returned future is first polled.
    fn request_authorization(
        &mut self,
        media: CaptureMedia,
    ) -> Self::RequestAuthorizationFuture<'_>;
}

#[cfg(test)]
mod tests {
    use super::{
        CaptureMedia, MediaAuthorization, MediaAuthorizationRequester, MediaAuthorizationStatus,
    };
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::Arc;

    struct FakeRequester {
        request_count: Rc<Cell<usize>>,
        selected_media: Rc<Cell<Option<CaptureMedia>>>,
    }

    struct FakeRequest {
        request_count: Rc<Cell<usize>>,
        selected_media: Rc<Cell<Option<CaptureMedia>>>,
        media: CaptureMedia,
        started: bool,
    }

    impl Future for FakeRequest {
        type Output = MediaAuthorizationStatus;

        fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            if !self.started {
                self.started = true;
                self.request_count.set(self.request_count.get() + 1);
                self.selected_media.set(Some(self.media));
            }
            Poll::Ready(FakeRequester::status(self.media))
        }
    }

    impl FakeRequester {
        fn status(media: CaptureMedia) -> MediaAuthorizationStatus {
            match media {
                CaptureMedia::Camera => MediaAuthorizationStatus::Authorized,
                CaptureMedia::Microphone => MediaAuthorizationStatus::Denied,
            }
        }

        fn new() -> Self {
            Self {
                request_count: Rc::new(Cell::new(0)),
                selected_media: Rc::new(Cell::new(None)),
            }
        }
    }

    impl MediaAuthorization for FakeRequester {
        fn authorization_status(media: CaptureMedia) -> MediaAuthorizationStatus {
            Self::status(media)
        }
    }

    impl MediaAuthorizationRequester for FakeRequester {
        type RequestAuthorizationFuture<'a>
            = FakeRequest
        where
            Self: 'a;

        fn request_authorization(
            &mut self,
            media: CaptureMedia,
        ) -> Self::RequestAuthorizationFuture<'_> {
            FakeRequest {
                request_count: Rc::clone(&self.request_count),
                selected_media: Rc::clone(&self.selected_media),
                media,
                started: false,
            }
        }
    }

    struct NoopWake;

    impl std::task::Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn camera_and_microphone_are_distinct_categories() {
        assert_ne!(CaptureMedia::Camera, CaptureMedia::Microphone);
    }

    #[test]
    fn every_authorization_status_is_distinct() {
        let statuses = [
            MediaAuthorizationStatus::NotDetermined,
            MediaAuthorizationStatus::Restricted,
            MediaAuthorizationStatus::Denied,
            MediaAuthorizationStatus::Authorized,
            MediaAuthorizationStatus::Unknown,
        ];
        for left in 0..statuses.len() {
            for right in left + 1..statuses.len() {
                assert_ne!(statuses[left], statuses[right]);
            }
        }
    }

    #[test]
    fn request_starts_on_first_poll_and_uses_selected_media() {
        let mut requester = FakeRequester::new();
        let request_count = Rc::clone(&requester.request_count);
        let selected_media = Rc::clone(&requester.selected_media);
        let mut request = requester.request_authorization(CaptureMedia::Microphone);

        assert_eq!(request_count.get(), 0);
        assert_eq!(selected_media.get(), None);

        let waker = std::task::Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        assert_eq!(
            Pin::new(&mut request).poll(&mut context),
            Poll::Ready(MediaAuthorizationStatus::Denied)
        );
        assert_eq!(request_count.get(), 1);
        assert_eq!(selected_media.get(), Some(CaptureMedia::Microphone));
    }

    #[test]
    fn dropping_unpolled_request_does_not_start_it() {
        let mut requester = FakeRequester::new();
        let request_count = Rc::clone(&requester.request_count);

        drop(requester.request_authorization(CaptureMedia::Camera));

        assert_eq!(request_count.get(), 0);
    }

    #[test]
    fn static_status_query_remains_non_requesting_and_media_specific() {
        assert_eq!(
            FakeRequester::authorization_status(CaptureMedia::Camera),
            MediaAuthorizationStatus::Authorized
        );
        assert_eq!(
            FakeRequester::authorization_status(CaptureMedia::Microphone),
            MediaAuthorizationStatus::Denied
        );
    }
}

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Contacts authorization values and a runtime-neutral static backend contract."]

use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// The normalized authorization state for access to contacts.
///
/// `Limited` is intentionally distinct from [`Authorized`](Self::Authorized): it permits access
/// to only the contacts selected by the user and does not mean full contact-store access.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum ContactsAuthorization {
    /// The native state is not recognized or cannot be classified.
    Unknown = 0,
    /// The user has not yet made a choice.
    NotDetermined = 1,
    /// Access is blocked by policy or device restrictions.
    Restricted = 2,
    /// The user denied contact access.
    Denied = 3,
    /// Full access to contacts is authorized.
    Authorized = 4,
    /// Access is authorized only for a user-selected subset of contacts.
    Limited = 5,
}

impl ContactsAuthorization {
    /// Reports whether the state permits access to any contact data.
    pub const fn allows_contact_access(self) -> bool {
        matches!(self, Self::Authorized | Self::Limited)
    }

    /// Reports whether the state represents full, rather than user-limited, access.
    pub const fn is_full_access(self) -> bool {
        matches!(self, Self::Authorized)
    }
}

/// A stable Contacts operation error with an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ContactsError {
    /// The selected backend returned an operation error rather than an authorization result.
    Backend(Error),
}

impl ContactsError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for Contacts authorization status and request operations.
///
/// [`authorization_status`](Self::authorization_status) must not show a permission prompt. A
/// request may prompt only after its returned future is first polled. Dropping an unpolled future
/// must have no prompt side effect. Dropping a started future abandons interest in its result but
/// cannot be assumed to dismiss a system prompt already shown. Callback state must not borrow
/// future-owned storage and must remain valid until the native operation reaches one terminal
/// outcome. The first terminal completion becomes one future result; duplicate native completions
/// must not produce another result. A live future yields its result at most once. No global
/// service, dynamic dispatch, or executor is required by this contract.
pub trait ContactsBackend {
    /// Reports whether the backend's authorization facility is available in this context.
    fn availability(&self) -> Availability;

    /// Queries authorization without prompting or starting an access request.
    fn authorization_status(&self) -> ContactsAuthorization;

    /// The future type for an explicit permission request.
    type RequestAuthorizationFuture<'a>: Future<Output = Result<ContactsAuthorization, ContactsError>>
        + 'a
    where
        Self: 'a;

    /// Explicitly requests contact authorization.
    ///
    /// The backend must start prompt-capable native work only when this future is first polled.
    /// A completed denial or limited grant is an authorization result, not inherently an
    /// operation error. Dropping the future cannot be assumed to cancel a prompt already shown;
    /// callback state must remain valid until native completion without borrowing future-owned
    /// storage. Duplicate native completions must not produce another future result.
    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a>;
}

/// A thin facade over caller-owned, statically selected Contacts backend state.
pub struct Contacts<B> {
    backend: B,
}

impl<B: ContactsBackend> Contacts<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Queries authorization without prompting.
    pub fn authorization_status(&self) -> ContactsAuthorization {
        self.backend.authorization_status()
    }

    /// Starts an explicit authorization request when the returned future is first polled.
    pub fn request_authorization(&mut self) -> B::RequestAuthorizationFuture<'_> {
        self.backend.request_authorization()
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};
    use std::cell::Cell;
    use std::rc::Rc;

    struct FakeBackend {
        status: ContactsAuthorization,
        requests: Rc<Cell<usize>>,
    }

    struct FakeRequestFuture<'a> {
        backend: &'a mut FakeBackend,
        completed: bool,
    }

    impl Future for FakeRequestFuture<'_> {
        type Output = Result<ContactsAuthorization, ContactsError>;

        fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            if self.completed {
                return Poll::Pending;
            }
            let backend = &mut self.backend;
            backend.requests.set(backend.requests.get() + 1);
            let status = backend.status;
            self.completed = true;
            Poll::Ready(Ok(status))
        }
    }

    impl ContactsBackend for FakeBackend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        fn authorization_status(&self) -> ContactsAuthorization {
            self.status
        }

        type RequestAuthorizationFuture<'a> = FakeRequestFuture<'a>;

        fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
            FakeRequestFuture {
                backend: self,
                completed: false,
            }
        }
    }

    #[test]
    fn authorization_values_keep_limited_separate_from_full_access() {
        assert_eq!(core::mem::size_of::<ContactsAuthorization>(), 1);
        assert_eq!(ContactsAuthorization::Unknown as u8, 0);
        assert_eq!(ContactsAuthorization::NotDetermined as u8, 1);
        assert_eq!(ContactsAuthorization::Restricted as u8, 2);
        assert_eq!(ContactsAuthorization::Denied as u8, 3);
        assert_eq!(ContactsAuthorization::Authorized as u8, 4);
        assert_eq!(ContactsAuthorization::Limited as u8, 5);
        assert!(!ContactsAuthorization::Unknown.allows_contact_access());
        assert!(!ContactsAuthorization::NotDetermined.allows_contact_access());
        assert!(ContactsAuthorization::Authorized.allows_contact_access());
        assert!(ContactsAuthorization::Authorized.is_full_access());
        assert!(ContactsAuthorization::Limited.allows_contact_access());
        assert!(!ContactsAuthorization::Limited.is_full_access());
        assert!(!ContactsAuthorization::Denied.allows_contact_access());
        assert!(!ContactsAuthorization::Restricted.allows_contact_access());
    }

    #[test]
    fn query_does_not_prompt_and_request_is_lazy() {
        let backend = FakeBackend {
            status: ContactsAuthorization::Limited,
            requests: Rc::new(Cell::new(0)),
        };
        let requests = backend.requests.clone();
        let mut contacts = Contacts::new(backend);
        assert_eq!(contacts.availability(), Availability::Available);
        assert_eq!(
            contacts.authorization_status(),
            ContactsAuthorization::Limited
        );
        assert_eq!(requests.get(), 0);

        {
            let _future = contacts.request_authorization();
            assert_eq!(requests.get(), 0);
        }
        assert_eq!(requests.get(), 0);

        assert_eq!(requests.get(), 0);
        {
            let mut future = contacts.request_authorization();
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(
                Pin::new(&mut future).poll(&mut context),
                Poll::Ready(Ok(ContactsAuthorization::Limited))
            ));
        }
        assert_eq!(requests.get(), 1);
    }

    #[test]
    fn backend_errors_preserve_category_and_native_code() {
        let error = ContactsError::Backend(
            Error::new(ErrorKind::Platform)
                .with_platform_code(PlatformErrorCode::new(-42).unwrap()),
        );
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);
    }
}

use crate::conversion::{authorization_from_native, request_result_from_native};
use crate::operation::CompletionCell;
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_contacts::{ContactsAuthorization, ContactsBackend, ContactsError};
use framework_core::{Availability, Error, ErrorKind};
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::Bool;
use objc2_contacts::{CNContactStore, CNEntityType};
use objc2_foundation::NSError;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

type Completion<T> = Arc<CompletionCell<T, ContactsError>>;

/// iOS Contacts authorization-request future.
pub type IosRequestAuthorizationFuture = IosContactsFuture;

/// A lazy Contacts permission request whose result is re-read from the system authorization status.
pub struct IosContactsFuture {
    store: Retained<CNContactStore>,
    completion: Completion<ContactsAuthorization>,
    started: bool,
    finished: bool,
}

impl IosContactsFuture {
    fn new(store: &Retained<CNContactStore>) -> Self {
        Self {
            store: store.clone(),
            completion: Arc::new(CompletionCell::new()),
            started: false,
            finished: false,
        }
    }

    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let store = self.store.clone();
        let completion = self.completion.clone();
        if catch_unwind(AssertUnwindSafe(|| {
            let callback_completion = completion.clone();
            let handler = RcBlock::new(move |_granted: Bool, error: *mut NSError| {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    autoreleasepool(|_| {
                        // SAFETY: NSError is borrowed only for this Contacts callback invocation.
                        let error_code = unsafe { error.as_ref() }.map(|error| error.code());
                        let status = native_authorization_status();
                        request_result_from_native(status, error_code)
                    })
                }))
                .unwrap_or_else(|_| Err(internal_error()));
                callback_completion.complete(result);
            });
            // SAFETY: The Contacts framework copies and retains its completion block for this
            // asynchronous request. The block owns its callback state and tolerates an arbitrary
            // callback queue. The request is reached only after the Rust future is first polled.
            unsafe {
                store.requestAccessForEntityType_completionHandler(CNEntityType::Contacts, &handler)
            };
        }))
        .is_err()
        {
            completion.complete(Err(internal_error()));
        }
    }
}

impl Future for IosContactsFuture {
    type Output = Result<ContactsAuthorization, ContactsError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.start();
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosContactsFuture {
    fn drop(&mut self) {
        if !self.finished {
            self.completion.detach();
        }
    }
}

/// Portable Contacts operations backed by the app's Contacts authorization state.
pub struct IosContactsBackend {
    store: Retained<CNContactStore>,
}

impl IosContactsBackend {
    /// Creates a backend handle without querying authorization or prompting the user.
    pub fn new() -> Self {
        let store = autoreleasepool(|_| {
            // SAFETY: `CNContactStore` is a public Contacts class with a generated `new` initializer.
            unsafe { CNContactStore::new() }
        });
        Self { store }
    }
}

impl Default for IosContactsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ContactsBackend for IosContactsBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn authorization_status(&self) -> ContactsAuthorization {
        authorization_from_native(native_authorization_status())
    }

    type RequestAuthorizationFuture<'a> = IosContactsFuture;

    fn request_authorization(&mut self) -> Self::RequestAuthorizationFuture<'_> {
        IosContactsFuture::new(&self.store)
    }
}

fn native_authorization_status() -> isize {
    // SAFETY: Contacts authorizationStatusForEntityType is a public, thread-safe status query.
    unsafe { CNContactStore::authorizationStatusForEntityType(CNEntityType::Contacts) }.0
}

fn internal_error() -> ContactsError {
    ContactsError::Backend(Error::new(ErrorKind::Internal))
}

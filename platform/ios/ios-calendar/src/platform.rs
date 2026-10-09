use crate::conversion::{authorization_from_native, error_from_native};
use crate::operation::CompletionCell;
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_calendar::{
    CalendarAuthorizationBackend, CalendarAuthorizationStatus, CalendarError,
};
use framework_core::{Error, ErrorKind};
use objc2::AnyThread;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::Bool;
use objc2_event_kit::{EKEntityType, EKEventStore};
use objc2_foundation::NSError;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

type Completion = Arc<CompletionCell<(), CalendarError>>;
type AccessHandler = dyn Fn(Bool, *mut NSError);

/// iOS full-Calendar-event-access request future.
pub type IosRequestFullAccessFuture<'a> = IosCalendarFuture<'a>;

/// A lazy iOS EventKit full-access request that retains safe callback state.
pub struct IosCalendarFuture<'a> {
    backend: &'a mut IosCalendarBackend,
    completion: Completion,
    handler: Option<RcBlock<AccessHandler>>,
    started: bool,
    finished: bool,
}

impl IosCalendarFuture<'_> {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let completion = self.completion.clone();
        let handler = RcBlock::new(move |_granted: Bool, error: *mut NSError| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: EventKit borrows NSError for the duration of its completion callback.
                unsafe { error.as_ref() }
                    .map_or(Ok(()), |error| Err(error_from_native(error.code())))
            }))
            .unwrap_or_else(|_| Err(internal_error()));
            completion.complete(result);
        });
        let handler_ptr = RcBlock::as_ptr(&handler);
        self.handler = Some(handler);
        // SAFETY: `handler_ptr` points to the live Rust block retained in `self.handler` for
        // this call. EventKit copies the completion block for its asynchronous callback.
        unsafe {
            self.backend
                .event_store
                .requestFullAccessToEventsWithCompletion(handler_ptr)
        };
    }
}

impl Future for IosCalendarFuture<'_> {
    type Output = Result<CalendarAuthorizationStatus, CalendarError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.start();
        }
        match this.completion.poll(context) {
            Poll::Ready(callback_result) => {
                // The callback Boolean is not permission state. Re-query EventKit only after
                // its completion has arrived, on the future's owning thread.
                let status = catch_unwind(AssertUnwindSafe(native_authorization_status))
                    .map_err(|_| internal_error());
                this.finished = true;
                Poll::Ready(callback_result.and(status))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosCalendarFuture<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.completion.detach();
        }
    }
}

/// Full Calendar event-authorization status and request through public EventKit APIs.
///
/// Create this backend on iOS 17 or later. Construction creates an `EKEventStore` but does not
/// prompt or query permission. Status reads use
/// `+[EKEventStore authorizationStatusForEntityType:]` with `EKEntityType::Event`. The explicit
/// request uses `requestFullAccessToEventsWithCompletion:` only when its future is first polled.
pub struct IosCalendarBackend {
    event_store: Retained<EKEventStore>,
}

impl IosCalendarBackend {
    /// Creates an EventKit handle without querying status or requesting authorization.
    pub fn new() -> Self {
        let event_store = autoreleasepool(|_| {
            // SAFETY: EKEventStore is documented as constructible with `init` from any thread.
            unsafe { EKEventStore::init(EKEventStore::alloc()) }
        });
        Self { event_store }
    }

    /// Borrows the native event store for iOS-only integrations.
    pub fn native_event_store(&self) -> &EKEventStore {
        &self.event_store
    }
}

impl Default for IosCalendarBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CalendarAuthorizationBackend for IosCalendarBackend {
    fn authorization_status(&self) -> CalendarAuthorizationStatus {
        native_authorization_status()
    }

    type RequestFullAccessFuture<'a>
        = IosRequestFullAccessFuture<'a>
    where
        Self: 'a;

    fn request_full_access<'a>(&'a mut self) -> Self::RequestFullAccessFuture<'a> {
        IosCalendarFuture {
            backend: self,
            completion: Arc::new(CompletionCell::new()),
            handler: None,
            started: false,
            finished: false,
        }
    }
}

fn native_authorization_status() -> CalendarAuthorizationStatus {
    // SAFETY: This static EventKit status query accepts the Event entity and has no prompt side
    // effect. The backend's documented API floor is iOS 17.0.
    let status = unsafe { EKEventStore::authorizationStatusForEntityType(EKEntityType::Event) };
    authorization_from_native(status.0)
}

fn internal_error() -> CalendarError {
    CalendarError::Backend(Error::new(ErrorKind::Internal))
}

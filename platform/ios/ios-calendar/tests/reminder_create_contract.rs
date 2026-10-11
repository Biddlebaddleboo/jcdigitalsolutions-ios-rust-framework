use core::cell::{Cell, RefCell};
use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use framework_calendar::{
    Calendar, CalendarError, CalendarReminderAuthorizationBackend, CalendarReminderCreateBackend,
    CalendarReminderCreateError, CalendarReminderDraft, ReminderAuthorizationStatus,
};
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use std::rc::Rc;
use std::string::String;
use std::sync::Arc;
use std::task::Wake;

struct FakeBackend {
    status: Cell<ReminderAuthorizationStatus>,
    request_calls: Rc<Cell<u8>>,
    request_polls: Rc<Cell<u8>>,
    create_calls: Cell<u8>,
    default_list_available: Cell<bool>,
    create_error: RefCell<Option<Error>>,
    saved_title: RefCell<Option<String>>,
}

impl FakeBackend {
    fn new(status: ReminderAuthorizationStatus) -> Self {
        Self {
            status: Cell::new(status),
            request_calls: Rc::new(Cell::new(0)),
            request_polls: Rc::new(Cell::new(0)),
            create_calls: Cell::new(0),
            default_list_available: Cell::new(true),
            create_error: RefCell::new(None),
            saved_title: RefCell::new(None),
        }
    }
}

struct FakeRequest {
    status: ReminderAuthorizationStatus,
    polls: Rc<Cell<u8>>,
}

impl Future for FakeRequest {
    type Output = Result<ReminderAuthorizationStatus, CalendarError>;

    fn poll(self: core::pin::Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        self.polls.set(self.polls.get() + 1);
        Poll::Ready(Ok(self.status))
    }
}

impl CalendarReminderAuthorizationBackend for FakeBackend {
    fn reminder_authorization_status(&self) -> ReminderAuthorizationStatus {
        self.status.get()
    }

    type RequestFullReminderAccessFuture<'a>
        = FakeRequest
    where
        Self: 'a;

    fn request_full_reminder_access<'a>(&'a mut self) -> Self::RequestFullReminderAccessFuture<'a> {
        self.request_calls.set(self.request_calls.get() + 1);
        FakeRequest {
            status: self.status.get(),
            polls: self.request_polls.clone(),
        }
    }
}

impl CalendarReminderCreateBackend for FakeBackend {
    fn create_reminder(
        &self,
        draft: &CalendarReminderDraft,
    ) -> Result<(), CalendarReminderCreateError> {
        let status = self.status.get();
        if status != ReminderAuthorizationStatus::FullAccess {
            return Err(CalendarReminderCreateError::NotFullAccess(status));
        }
        self.create_calls.set(self.create_calls.get() + 1);
        if !self.default_list_available.get() {
            return Err(CalendarReminderCreateError::DefaultListUnavailable);
        }
        if let Some(error) = *self.create_error.borrow() {
            return Err(CalendarReminderCreateError::Backend(error));
        }
        *self.saved_title.borrow_mut() = Some(String::from(draft.title()));
        Ok(())
    }
}

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

#[test]
fn reminder_status_is_nonprompting_and_explicit_request_is_lazy() {
    let mut calendar = Calendar::new(FakeBackend::new(ReminderAuthorizationStatus::NotDetermined));
    let request_calls = calendar.backend().request_calls.clone();
    let request_polls = calendar.backend().request_polls.clone();
    assert_eq!(
        calendar.reminder_authorization_status(),
        ReminderAuthorizationStatus::NotDetermined
    );
    assert_eq!(request_calls.get(), 0);

    let unpolled = calendar.request_full_reminder_access();
    assert_eq!(request_calls.get(), 0);
    drop(unpolled);
    assert_eq!(request_calls.get(), 0);
    assert_eq!(request_polls.get(), 0);

    let mut request = pin!(calendar.request_full_reminder_access());
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    assert!(matches!(
        request.as_mut().poll(&mut context),
        Poll::Ready(Ok(ReminderAuthorizationStatus::NotDetermined))
    ));
    assert_eq!(request_calls.get(), 1);
    assert_eq!(request_polls.get(), 1);
}

#[test]
fn invalid_or_non_full_access_never_reaches_create_backend() {
    assert_eq!(
        CalendarReminderDraft::new(String::new()),
        Err(CalendarReminderCreateError::EmptyTitle)
    );
    for status in [
        ReminderAuthorizationStatus::Unknown,
        ReminderAuthorizationStatus::NotDetermined,
        ReminderAuthorizationStatus::Restricted,
        ReminderAuthorizationStatus::Denied,
    ] {
        let calendar = Calendar::new(FakeBackend::new(status));
        let draft = CalendarReminderDraft::new(String::from("Call Alex")).unwrap();
        assert_eq!(
            calendar.create_reminder(&draft),
            Err(CalendarReminderCreateError::NotFullAccess(status))
        );
        assert_eq!(calendar.backend().create_calls.get(), 0);
        assert_eq!(calendar.backend().request_calls.get(), 0);
    }
}

#[test]
fn one_full_access_create_preserves_title_and_does_not_request_access() {
    let calendar = Calendar::new(FakeBackend::new(ReminderAuthorizationStatus::FullAccess));
    let draft = CalendarReminderDraft::new(String::from("Pay rent 🏠")).unwrap();

    assert_eq!(calendar.create_reminder(&draft), Ok(()));
    assert_eq!(calendar.backend().create_calls.get(), 1);
    assert_eq!(
        calendar.backend().saved_title.borrow().as_deref(),
        Some("Pay rent 🏠")
    );
    assert_eq!(calendar.backend().request_calls.get(), 0);
}

#[test]
fn missing_default_list_is_a_typed_error_without_prompt() {
    let backend = FakeBackend::new(ReminderAuthorizationStatus::FullAccess);
    backend.default_list_available.set(false);
    let calendar = Calendar::new(backend);
    let draft = CalendarReminderDraft::new(String::from("Pay rent")).unwrap();

    assert_eq!(
        calendar.create_reminder(&draft),
        Err(CalendarReminderCreateError::DefaultListUnavailable)
    );
    assert_eq!(calendar.backend().create_calls.get(), 1);
    assert_eq!(calendar.backend().request_calls.get(), 0);
    assert_eq!(
        CalendarReminderCreateError::DefaultListUnavailable.kind(),
        ErrorKind::Unavailable
    );
}

#[test]
fn native_create_error_keeps_kind_and_platform_code() {
    let backend = FakeBackend::new(ReminderAuthorizationStatus::FullAccess);
    let code = PlatformErrorCode::new(-42).unwrap();
    *backend.create_error.borrow_mut() =
        Some(Error::new(ErrorKind::Platform).with_platform_code(code));
    let calendar = Calendar::new(backend);
    let draft = CalendarReminderDraft::new(String::from("Pay rent")).unwrap();

    assert_eq!(
        calendar.create_reminder(&draft),
        Err(CalendarReminderCreateError::Backend(
            Error::new(ErrorKind::Platform).with_platform_code(code)
        ))
    );
    assert_eq!(calendar.backend().create_calls.get(), 1);
}

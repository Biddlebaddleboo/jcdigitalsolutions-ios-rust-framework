use core::cell::{Cell, RefCell};
use framework_calendar::{
    Calendar, CalendarAuthorizationStatus, CalendarEvent, CalendarEventDeleteBackend,
    CalendarEventDeleteError, CalendarEventTime,
};
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use std::string::String;

struct FakeBackend {
    status: Cell<CalendarAuthorizationStatus>,
    resolved_identifier: String,
    lookup_calls: Cell<u8>,
    remove_calls: Cell<u8>,
    removed_identifiers: RefCell<Vec<String>>,
    remove_error: Cell<Option<Error>>,
}

impl FakeBackend {
    fn new(status: CalendarAuthorizationStatus, resolved_identifier: &str) -> Self {
        Self {
            status: Cell::new(status),
            resolved_identifier: String::from(resolved_identifier),
            lookup_calls: Cell::new(0),
            remove_calls: Cell::new(0),
            removed_identifiers: RefCell::new(Vec::new()),
            remove_error: Cell::new(None),
        }
    }
}

impl CalendarEventDeleteBackend for FakeBackend {
    fn delete_event(&self, event: &CalendarEvent) -> Result<(), CalendarEventDeleteError> {
        if self.status.get() != CalendarAuthorizationStatus::FullAccess {
            return Err(CalendarEventDeleteError::NotFullAccess(self.status.get()));
        }
        let identifier = event
            .event_identifier()
            .filter(|identifier| !identifier.is_empty())
            .ok_or(CalendarEventDeleteError::InvalidEventIdentifier)?;
        self.lookup_calls.set(self.lookup_calls.get() + 1);
        if identifier != self.resolved_identifier {
            return Err(CalendarEventDeleteError::EventNotFound);
        }
        self.remove_calls.set(self.remove_calls.get() + 1);
        self.removed_identifiers
            .borrow_mut()
            .push(String::from(identifier));
        if let Some(error) = self.remove_error.get() {
            return Err(CalendarEventDeleteError::Backend(error));
        }
        Ok(())
    }
}

fn event(identifier: Option<&str>) -> CalendarEvent {
    CalendarEvent::new(
        identifier.map(String::from),
        String::from("Selected event"),
        CalendarEventTime::from_unix_seconds(1_700_000_000.125).unwrap(),
        CalendarEventTime::from_unix_seconds(1_700_000_300.875).unwrap(),
        false,
        None,
        None,
    )
    .unwrap()
}

#[test]
fn absent_and_empty_identifiers_fail_before_backend_calls() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::FullAccess,
        "selected-event",
    ));

    assert_eq!(
        calendar.delete_event(&event(None)),
        Err(CalendarEventDeleteError::InvalidEventIdentifier)
    );
    assert_eq!(
        calendar.delete_event(&event(Some(""))),
        Err(CalendarEventDeleteError::InvalidEventIdentifier)
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 0);
    assert_eq!(calendar.backend().remove_calls.get(), 0);
}

#[test]
fn revoked_full_access_fails_without_event_lookup_or_removal() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::FullAccess,
        "selected-event",
    ));
    calendar
        .backend()
        .status
        .set(CalendarAuthorizationStatus::Denied);

    assert_eq!(
        calendar.delete_event(&event(Some("selected-event"))),
        Err(CalendarEventDeleteError::NotFullAccess(
            CalendarAuthorizationStatus::Denied
        ))
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 0);
    assert_eq!(calendar.backend().remove_calls.get(), 0);
}

#[test]
fn one_delete_uses_exactly_the_selected_identifier_once() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::FullAccess,
        "selected-event",
    ));

    assert_eq!(
        calendar.delete_event(&event(Some("selected-event"))),
        Ok(())
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().remove_calls.get(), 1);
    assert_eq!(
        calendar.backend().removed_identifiers.borrow().as_slice(),
        &[String::from("selected-event")]
    );
}

#[test]
fn stale_identifier_fails_without_removal() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::FullAccess,
        "different-event",
    ));

    assert_eq!(
        calendar.delete_event(&event(Some("selected-event"))),
        Err(CalendarEventDeleteError::EventNotFound)
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().remove_calls.get(), 0);
}

#[test]
fn native_delete_failure_preserves_its_platform_code() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess, "selected-event");
    let code = PlatformErrorCode::new(-42).unwrap();
    backend.remove_error.set(Some(
        Error::new(ErrorKind::Platform).with_platform_code(code),
    ));
    let calendar = Calendar::new(backend);

    let error = calendar
        .delete_event(&event(Some("selected-event")))
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Platform);
    assert_eq!(error.platform_code(), Some(code));
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().remove_calls.get(), 1);
}

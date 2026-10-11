use core::cell::{Cell, RefCell};
use framework_calendar::{
    Calendar, CalendarAuthorizationStatus, CalendarEvent, CalendarEventDraft, CalendarEventTime,
    CalendarEventUpdateBackend, CalendarEventUpdateError,
};
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use std::string::String;

struct FakeBackend {
    status: Cell<CalendarAuthorizationStatus>,
    resolved_identifier: String,
    resolved_start: Cell<f64>,
    resolved_end: Cell<f64>,
    supports_events: Cell<bool>,
    allows_changes: Cell<bool>,
    calendar_identifier: String,
    dispatch_calls: Cell<u8>,
    lookup_calls: Cell<u8>,
    save_calls: Cell<u8>,
    save_error: Cell<Option<Error>>,
    saved: RefCell<Option<(String, CalendarEventDraft)>>,
}

impl FakeBackend {
    fn new(status: CalendarAuthorizationStatus, resolved_identifier: &str) -> Self {
        Self {
            status: Cell::new(status),
            resolved_identifier: String::from(resolved_identifier),
            resolved_start: Cell::new(1_700_000_000.125),
            resolved_end: Cell::new(1_700_000_300.875),
            supports_events: Cell::new(true),
            allows_changes: Cell::new(true),
            calendar_identifier: String::from("current-calendar"),
            dispatch_calls: Cell::new(0),
            lookup_calls: Cell::new(0),
            save_calls: Cell::new(0),
            save_error: Cell::new(None),
            saved: RefCell::new(None),
        }
    }
}

impl CalendarEventUpdateBackend for FakeBackend {
    fn update_event(
        &self,
        event: &CalendarEvent,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventUpdateError> {
        self.dispatch_calls.set(self.dispatch_calls.get() + 1);
        let status = self.status.get();
        if status != CalendarAuthorizationStatus::FullAccess {
            return Err(CalendarEventUpdateError::NotFullAccess(status));
        }
        let identifier = event
            .event_identifier()
            .filter(|identifier| !identifier.is_empty())
            .ok_or(CalendarEventUpdateError::InvalidEventIdentifier)?;
        self.lookup_calls.set(self.lookup_calls.get() + 1);
        if identifier != self.resolved_identifier
            || event.start().unix_seconds() != self.resolved_start.get()
            || event.end().unix_seconds() != self.resolved_end.get()
        {
            return Err(CalendarEventUpdateError::EventNotFound);
        }
        if !self.supports_events.get() {
            return Err(CalendarEventUpdateError::CalendarDoesNotSupportEvents);
        }
        if !self.allows_changes.get() {
            return Err(CalendarEventUpdateError::CalendarNotWritable);
        }
        self.save_calls.set(self.save_calls.get() + 1);
        if let Some(error) = self.save_error.get() {
            return Err(CalendarEventUpdateError::Backend(error));
        }
        *self.saved.borrow_mut() = Some((self.calendar_identifier.clone(), draft.clone()));
        Ok(())
    }
}

fn event(identifier: Option<&str>, start: f64, end: f64) -> CalendarEvent {
    CalendarEvent::new(
        identifier.map(String::from),
        String::from("Selected event"),
        CalendarEventTime::from_unix_seconds(start).unwrap(),
        CalendarEventTime::from_unix_seconds(end).unwrap(),
        false,
        Some(String::from("Old location")),
        Some(String::from("Old notes")),
    )
    .unwrap()
}

fn draft() -> CalendarEventDraft {
    CalendarEventDraft::new(
        String::from("Updated 🗓"),
        CalendarEventTime::from_unix_seconds(1_700_001_000.25).unwrap(),
        CalendarEventTime::from_unix_seconds(1_700_001_360.75).unwrap(),
        true,
        None,
        Some(String::from("Replacement notes")),
    )
    .unwrap()
}

fn selected_event() -> CalendarEvent {
    event(Some("selected-event"), 1_700_000_000.125, 1_700_000_300.875)
}

#[test]
fn missing_and_empty_identifiers_fail_before_backend_dispatch() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::FullAccess,
        "selected-event",
    ));

    assert_eq!(
        calendar.update_event(&event(None, 1_700_000_000.125, 1_700_000_300.875), &draft()),
        Err(CalendarEventUpdateError::InvalidEventIdentifier)
    );
    assert_eq!(
        calendar.update_event(
            &event(Some(""), 1_700_000_000.125, 1_700_000_300.875),
            &draft()
        ),
        Err(CalendarEventUpdateError::InvalidEventIdentifier)
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 0);
    assert_eq!(calendar.backend().save_calls.get(), 0);
    assert_eq!(calendar.backend().dispatch_calls.get(), 0);
}

#[test]
fn revoked_full_access_fails_without_event_lookup_or_save() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::Denied,
        "selected-event",
    ));

    assert_eq!(
        calendar.update_event(&selected_event(), &draft()),
        Err(CalendarEventUpdateError::NotFullAccess(
            CalendarAuthorizationStatus::Denied
        ))
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 0);
    assert_eq!(calendar.backend().save_calls.get(), 0);
}

#[test]
fn recurring_first_occurrence_mismatch_fails_without_mutation() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess, "recurring-event");
    backend.resolved_start.set(1_700_000_000.0);
    backend.resolved_end.set(1_700_000_300.0);
    let calendar = Calendar::new(backend);

    assert_eq!(
        calendar.update_event(
            &event(Some("recurring-event"), 1_700_086_400.0, 1_700_086_800.0,),
            &draft()
        ),
        Err(CalendarEventUpdateError::EventNotFound)
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().save_calls.get(), 0);
    assert_eq!(calendar.backend().saved.borrow().as_ref(), None);
}

#[test]
fn non_event_calendar_fails_before_save() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess, "selected-event");
    backend.supports_events.set(false);
    let calendar = Calendar::new(backend);

    assert_eq!(
        calendar.update_event(&selected_event(), &draft()),
        Err(CalendarEventUpdateError::CalendarDoesNotSupportEvents)
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().save_calls.get(), 0);
}

#[test]
fn unwritable_calendar_fails_before_save() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess, "selected-event");
    backend.allows_changes.set(false);
    let calendar = Calendar::new(backend);

    assert_eq!(
        calendar.update_event(&selected_event(), &draft()),
        Err(CalendarEventUpdateError::CalendarNotWritable)
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().save_calls.get(), 0);
}

#[test]
fn one_update_replaces_exact_draft_fields_and_preserves_current_calendar() {
    let calendar = Calendar::new(FakeBackend::new(
        CalendarAuthorizationStatus::FullAccess,
        "selected-event",
    ));
    let replacement = draft();

    assert_eq!(
        calendar.update_event(&selected_event(), &replacement),
        Ok(())
    );
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().save_calls.get(), 1);
    assert_eq!(
        calendar.backend().saved.borrow().as_ref(),
        Some(&(String::from("current-calendar"), replacement.clone()))
    );
    let saved = calendar.backend().saved.borrow();
    let (_, saved_draft) = saved.as_ref().unwrap();
    assert_eq!(saved_draft.title(), "Updated 🗓");
    assert_eq!(saved_draft.start().unix_seconds(), 1_700_001_000.25);
    assert_eq!(saved_draft.end().unix_seconds(), 1_700_001_360.75);
    assert!(saved_draft.is_all_day());
    assert_eq!(saved_draft.location(), None);
    assert_eq!(saved_draft.notes(), Some("Replacement notes"));
}

#[test]
fn native_update_failure_preserves_platform_error_code() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess, "selected-event");
    let code = PlatformErrorCode::new(-42).unwrap();
    backend.save_error.set(Some(
        Error::new(ErrorKind::Platform).with_platform_code(code),
    ));
    let calendar = Calendar::new(backend);

    let error = calendar
        .update_event(&selected_event(), &draft())
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Platform);
    assert_eq!(error.platform_code(), Some(code));
    assert_eq!(calendar.backend().lookup_calls.get(), 1);
    assert_eq!(calendar.backend().save_calls.get(), 1);
}

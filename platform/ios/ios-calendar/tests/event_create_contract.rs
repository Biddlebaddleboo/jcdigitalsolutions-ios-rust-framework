use core::cell::{Cell, RefCell};
use framework_calendar::{
    Calendar, CalendarAuthorizationStatus, CalendarEventCreateBackend, CalendarEventCreateError,
    CalendarEventDraft, CalendarEventError, CalendarEventTime, WritableEventCalendar,
};
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use std::string::String;

struct FakeBackend {
    status: Cell<CalendarAuthorizationStatus>,
    list_calls: Cell<u8>,
    create_calls: Cell<u8>,
    create_error: Cell<Option<Error>>,
    saved: RefCell<Option<(String, CalendarEventDraft)>>,
}

impl FakeBackend {
    fn new(status: CalendarAuthorizationStatus) -> Self {
        Self {
            status: Cell::new(status),
            list_calls: Cell::new(0),
            create_calls: Cell::new(0),
            create_error: Cell::new(None),
            saved: RefCell::new(None),
        }
    }

    fn require_full_access(&self) -> Result<(), CalendarEventCreateError> {
        if self.status.get() == CalendarAuthorizationStatus::FullAccess {
            Ok(())
        } else {
            Err(CalendarEventCreateError::NotFullAccess(self.status.get()))
        }
    }
}

impl CalendarEventCreateBackend for FakeBackend {
    fn writable_event_calendars(
        &self,
    ) -> Result<Vec<WritableEventCalendar>, CalendarEventCreateError> {
        self.require_full_access()?;
        self.list_calls.set(self.list_calls.get() + 1);
        Ok(vec![WritableEventCalendar::new(
            String::from("calendar-1"),
            String::from("Personal"),
        )?])
    }

    fn create_event(
        &self,
        calendar: &WritableEventCalendar,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventCreateError> {
        self.require_full_access()?;
        self.create_calls.set(self.create_calls.get() + 1);
        if let Some(error) = self.create_error.get() {
            return Err(CalendarEventCreateError::Backend(error));
        }
        *self.saved.borrow_mut() = Some((String::from(calendar.identifier()), draft.clone()));
        Ok(())
    }
}

fn sample_draft() -> CalendarEventDraft {
    CalendarEventDraft::new(
        String::from("Planning 🗓"),
        CalendarEventTime::from_unix_seconds(1_700_000_000.125).unwrap(),
        CalendarEventTime::from_unix_seconds(1_700_000_300.875).unwrap(),
        false,
        Some(String::from("Room 1")),
        Some(String::from("Keep this note exactly")),
    )
    .unwrap()
}

#[test]
fn invalid_draft_and_empty_calendar_identifier_fail_before_backend_calls() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess);
    let calendar = Calendar::new(backend);
    let time = CalendarEventTime::from_unix_seconds(1.0).unwrap();

    assert_eq!(
        CalendarEventDraft::new(String::new(), time, time, false, None, None),
        Err(CalendarEventCreateError::InvalidEventInterval)
    );
    assert_eq!(
        CalendarEventDraft::new(
            String::new(),
            CalendarEventTime::from_unix_seconds(2.0).unwrap(),
            time,
            false,
            None,
            None
        ),
        Err(CalendarEventCreateError::InvalidEventInterval)
    );
    assert_eq!(
        CalendarEventTime::from_unix_seconds(f64::NAN),
        Err(CalendarEventError::InvalidTime)
    );
    assert_eq!(
        WritableEventCalendar::new(String::new(), String::from("Personal")),
        Err(CalendarEventCreateError::InvalidCalendarIdentifier)
    );
    assert_eq!(calendar.backend().list_calls.get(), 0);
    assert_eq!(calendar.backend().create_calls.get(), 0);
}

#[test]
fn calendar_listing_and_create_recheck_full_access_without_prompt() {
    let calendar = Calendar::new(FakeBackend::new(CalendarAuthorizationStatus::Denied));
    assert_eq!(
        calendar.writable_event_calendars(),
        Err(CalendarEventCreateError::NotFullAccess(
            CalendarAuthorizationStatus::Denied
        ))
    );
    assert_eq!(calendar.backend().list_calls.get(), 0);

    calendar
        .backend()
        .status
        .set(CalendarAuthorizationStatus::FullAccess);
    let selected = calendar.writable_event_calendars().unwrap().remove(0);
    calendar
        .backend()
        .status
        .set(CalendarAuthorizationStatus::WriteOnly);
    assert_eq!(
        calendar.create_event(&selected, &sample_draft()),
        Err(CalendarEventCreateError::NotFullAccess(
            CalendarAuthorizationStatus::WriteOnly
        ))
    );
    assert_eq!(calendar.backend().create_calls.get(), 0);
}

#[test]
fn native_backend_failure_keeps_its_error_kind_and_platform_code() {
    let backend = FakeBackend::new(CalendarAuthorizationStatus::FullAccess);
    let code = PlatformErrorCode::new(-42).unwrap();
    backend.create_error.set(Some(
        Error::new(ErrorKind::Platform).with_platform_code(code),
    ));
    let calendar = Calendar::new(backend);
    let selected = calendar.writable_event_calendars().unwrap().remove(0);

    assert_eq!(
        calendar.create_event(&selected, &sample_draft()),
        Err(CalendarEventCreateError::Backend(
            Error::new(ErrorKind::Platform).with_platform_code(code)
        ))
    );
    assert_eq!(calendar.backend().create_calls.get(), 1);
}

#[test]
fn one_successful_create_preserves_selected_calendar_and_exact_draft_fields() {
    let calendar = Calendar::new(FakeBackend::new(CalendarAuthorizationStatus::FullAccess));
    let selected = calendar.writable_event_calendars().unwrap().remove(0);
    let draft = sample_draft();

    assert_eq!(calendar.create_event(&selected, &draft), Ok(()));
    assert_eq!(calendar.backend().list_calls.get(), 1);
    assert_eq!(calendar.backend().create_calls.get(), 1);
    assert_eq!(selected.identifier(), "calendar-1");
    assert_eq!(selected.title(), "Personal");
    assert_eq!(draft.title(), "Planning 🗓");
    assert_eq!(draft.start().unix_seconds(), 1_700_000_000.125);
    assert_eq!(draft.end().unix_seconds(), 1_700_000_300.875);
    assert!(!draft.is_all_day());
    assert_eq!(draft.location(), Some("Room 1"));
    assert_eq!(draft.notes(), Some("Keep this note exactly"));
    assert_eq!(
        calendar.backend().saved.borrow().as_ref(),
        Some(&(String::from("calendar-1"), draft))
    );
}

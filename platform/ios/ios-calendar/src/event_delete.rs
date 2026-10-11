use crate::conversion::{error_from_native, validate_event_delete_snapshot};
use crate::platform::{IosCalendarBackend, native_authorization_status};
use framework_calendar::{
    CalendarAuthorizationStatus, CalendarError, CalendarEvent, CalendarEventDeleteBackend,
    CalendarEventDeleteError,
};
use framework_core::{Error, ErrorKind};
use objc2::rc::autoreleasepool;
use objc2_event_kit::{EKEntityMask, EKSpan};
use objc2_foundation::NSString;
use std::string::ToString;

impl CalendarEventDeleteBackend for IosCalendarBackend {
    fn delete_event(&self, event: &CalendarEvent) -> Result<(), CalendarEventDeleteError> {
        let identifier = event
            .event_identifier()
            .filter(|identifier| !identifier.is_empty())
            .ok_or(CalendarEventDeleteError::InvalidEventIdentifier)?;
        require_full_access_for_delete(native_authorization_status())?;

        autoreleasepool(|_| {
            let native_identifier = NSString::from_str(identifier);
            // SAFETY: The non-empty caller-selected identifier is resolved only through the
            // retained EventKit store owned by this backend. No query or fallback is performed.
            let native_event = unsafe {
                self.native_event_store()
                    .eventWithIdentifier(&native_identifier)
            }
            .ok_or(CalendarEventDeleteError::EventNotFound)?;

            // SAFETY: This is a live EKEvent returned by this backend's own event store.
            let resolved_identifier = unsafe { native_event.eventIdentifier() }
                .ok_or(CalendarEventDeleteError::EventNotFound)?
                .to_string();
            // `eventWithIdentifier:` returns the first occurrence for a recurring identifier.
            // Require exact snapshot dates so a later selected recurrence cannot delete another
            // occurrence when EventKit resolves only the first one.
            // SAFETY: The fetched event is retained and its date getters are valid for this scope.
            let (start, end) = unsafe {
                (
                    native_event.startDate().timeIntervalSince1970(),
                    native_event.endDate().timeIntervalSince1970(),
                )
            };
            validate_event_delete_snapshot(event, &resolved_identifier, start, end)?;

            // SAFETY: This event was resolved from this store and remains retained above.
            let native_calendar = unsafe { native_event.calendar() }
                .ok_or(CalendarEventDeleteError::EventNotFound)?;
            // SAFETY: Both getters read current properties from the retained event calendar.
            let supports_events = unsafe {
                native_calendar
                    .allowedEntityTypes()
                    .contains(EKEntityMask::Event)
            };
            // SAFETY: This reads the calendar's current modification permission.
            let allows_changes = unsafe { native_calendar.allowsContentModifications() };
            if !supports_events {
                return Err(CalendarEventDeleteError::CalendarDoesNotSupportEvents);
            }
            if !allows_changes {
                return Err(CalendarEventDeleteError::CalendarNotWritable);
            }

            // SAFETY: The exact selected EKEvent belongs to this retained store. This single
            // call commits only this occurrence; native errors are preserved below.
            unsafe {
                self.native_event_store().removeEvent_span_commit_error(
                    &native_event,
                    EKSpan::ThisEvent,
                    true,
                )
            }
            .map_err(|error| delete_error_from_native(error.code()))
        })
    }
}

fn require_full_access_for_delete(
    status: CalendarAuthorizationStatus,
) -> Result<(), CalendarEventDeleteError> {
    if status == CalendarAuthorizationStatus::FullAccess {
        Ok(())
    } else {
        Err(CalendarEventDeleteError::NotFullAccess(status))
    }
}

fn delete_error_from_native(code: isize) -> CalendarEventDeleteError {
    match error_from_native(code) {
        CalendarError::Backend(error) => CalendarEventDeleteError::Backend(error),
        _ => CalendarEventDeleteError::Backend(Error::new(ErrorKind::Platform)),
    }
}

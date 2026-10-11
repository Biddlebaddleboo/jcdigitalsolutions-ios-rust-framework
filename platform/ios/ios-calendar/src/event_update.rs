use crate::conversion::{update_error_from_native, validate_event_update_snapshot};
use crate::platform::{IosCalendarBackend, native_authorization_status};
use framework_calendar::{
    CalendarAuthorizationStatus, CalendarEvent, CalendarEventDraft, CalendarEventUpdateBackend,
    CalendarEventUpdateError,
};
use objc2::rc::autoreleasepool;
use objc2_event_kit::{EKEntityMask, EKSpan};
use objc2_foundation::{NSDate, NSString};
use std::string::ToString;

impl CalendarEventUpdateBackend for IosCalendarBackend {
    fn update_event(
        &self,
        event: &CalendarEvent,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventUpdateError> {
        let identifier = event
            .event_identifier()
            .filter(|identifier| !identifier.is_empty())
            .ok_or(CalendarEventUpdateError::InvalidEventIdentifier)?;
        if draft.start() >= draft.end() {
            return Err(CalendarEventUpdateError::InvalidEventInterval);
        }
        require_full_access_for_update(native_authorization_status())?;

        autoreleasepool(|_| {
            let native_identifier = NSString::from_str(identifier);
            // SAFETY: The validated caller-selected identifier is resolved only through this
            // backend's retained EventKit store. No enumeration, search, or fallback is performed.
            let native_event = unsafe {
                self.native_event_store()
                    .eventWithIdentifier(&native_identifier)
            }
            .ok_or(CalendarEventUpdateError::EventNotFound)?;

            // SAFETY: This is a live EKEvent returned by this backend's own event store.
            let resolved_identifier = unsafe { native_event.eventIdentifier() }
                .ok_or(CalendarEventUpdateError::EventNotFound)?
                .to_string();
            // `eventWithIdentifier:` resolves the first occurrence for a recurring identifier.
            // Require the exact selected dates so an update cannot mutate a different occurrence.
            // SAFETY: The fetched event is retained and its date getters are valid for this scope.
            let (start, end) = unsafe {
                (
                    native_event.startDate().timeIntervalSince1970(),
                    native_event.endDate().timeIntervalSince1970(),
                )
            };
            validate_event_update_snapshot(event, &resolved_identifier, start, end)?;

            // SAFETY: This calendar belongs to the retained event resolved above.
            let native_calendar = unsafe { native_event.calendar() }
                .ok_or(CalendarEventUpdateError::EventNotFound)?;
            // SAFETY: Both getters read current properties from the retained event calendar.
            let supports_events = unsafe {
                native_calendar
                    .allowedEntityTypes()
                    .contains(EKEntityMask::Event)
            };
            // SAFETY: This reads the calendar's current modification permission.
            let allows_changes = unsafe { native_calendar.allowsContentModifications() };
            if !supports_events {
                return Err(CalendarEventUpdateError::CalendarDoesNotSupportEvents);
            }
            if !allows_changes {
                return Err(CalendarEventUpdateError::CalendarNotWritable);
            }

            let title = NSString::from_str(draft.title());
            let location = draft.location().map(NSString::from_str);
            let notes = draft.notes().map(NSString::from_str);
            let start = NSDate::dateWithTimeIntervalSince1970(draft.start().unix_seconds());
            let end = NSDate::dateWithTimeIntervalSince1970(draft.end().unix_seconds());

            // SAFETY: Set only the caller's replacement fields on the exact retained event. Its
            // current calendar is left untouched, and all supplied objects live through the save.
            unsafe {
                native_event.setTitle(Some(&title));
                native_event.setStartDate(Some(&start));
                native_event.setEndDate(Some(&end));
                native_event.setAllDay(draft.is_all_day());
                native_event.setLocation(location.as_deref());
                native_event.setNotes(notes.as_deref());
            }

            // SAFETY: Commit exactly this selected occurrence once. EventKit's NSError code is
            // preserved when representable; no retry or series edit is attempted.
            unsafe {
                self.native_event_store().saveEvent_span_commit_error(
                    &native_event,
                    EKSpan::ThisEvent,
                    true,
                )
            }
            .map_err(|error| update_error_from_native(error.code()))
        })
    }
}

fn require_full_access_for_update(
    status: CalendarAuthorizationStatus,
) -> Result<(), CalendarEventUpdateError> {
    if status == CalendarAuthorizationStatus::FullAccess {
        Ok(())
    } else {
        Err(CalendarEventUpdateError::NotFullAccess(status))
    }
}

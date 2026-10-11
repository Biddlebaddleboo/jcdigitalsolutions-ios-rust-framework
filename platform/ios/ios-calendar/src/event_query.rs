use crate::conversion::{event_from_native_fields, require_full_access, validate_eventkit_range};
use crate::platform::{IosCalendarBackend, native_authorization_status};
use framework_calendar::{
    CalendarEvent, CalendarEventBackend, CalendarEventError, CalendarEventRange,
};
use objc2::rc::autoreleasepool;
use objc2_event_kit::EKEvent;
use objc2_foundation::NSDate;
use std::string::ToString;
use std::vec::Vec;

impl CalendarEventBackend for IosCalendarBackend {
    fn events_in_range(
        &self,
        range: CalendarEventRange,
    ) -> Result<Vec<CalendarEvent>, CalendarEventError> {
        validate_eventkit_range(range)?;
        require_full_access(native_authorization_status())?;

        autoreleasepool(|_| {
            let start = NSDate::dateWithTimeIntervalSince1970(range.start().unix_seconds());
            let end = NSDate::dateWithTimeIntervalSince1970(range.end().unix_seconds());
            // SAFETY: EventKit requires a predicate built by this event store; the supplied
            // finite dates were validated and `None` intentionally selects all calendars.
            let predicate = unsafe {
                self.native_event_store()
                    .predicateForEventsWithStartDate_endDate_calendars(&start, &end, None)
            };
            // SAFETY: `predicate` was created by this same event store with its public range API.
            let events = unsafe {
                self.native_event_store()
                    .eventsMatchingPredicate(&predicate)
            };
            events.iter().map(|event| copy_event(&event)).collect()
        })
    }
}

fn copy_event(event: &EKEvent) -> Result<CalendarEvent, CalendarEventError> {
    // SAFETY: The query returned this committed event from the same live event store; each
    // getter is valid for the event's lifetime and all native values are copied before return.
    let identifier = unsafe { event.eventIdentifier() }.map(|value| value.to_string());
    // SAFETY: `event` is a live EventKit event borrowed from the retained result array.
    let title = unsafe { event.title() }.to_string();
    // SAFETY: EventKit supplies the event's start date for every saved event.
    let start = unsafe { event.startDate() }.timeIntervalSince1970();
    // SAFETY: EventKit supplies the event's end date for every saved event.
    let end = unsafe { event.endDate() }.timeIntervalSince1970();
    // SAFETY: `isAllDay` reads a Boolean property from the live event.
    let all_day = unsafe { event.isAllDay() };
    // SAFETY: Optional strings are copied while the event remains retained by the result array.
    let location = unsafe { event.location() }.map(|value| value.to_string());
    // SAFETY: Optional strings are copied while the event remains retained by the result array.
    let notes = unsafe { event.notes() }.map(|value| value.to_string());

    event_from_native_fields(identifier, title, start, end, all_day, location, notes)
}

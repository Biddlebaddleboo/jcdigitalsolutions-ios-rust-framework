use crate::conversion::error_from_native;
use crate::platform::{IosCalendarBackend, native_authorization_status};
use framework_calendar::{
    CalendarAuthorizationStatus, CalendarError, CalendarEventCreateBackend,
    CalendarEventCreateError, CalendarEventDraft, WritableEventCalendar,
};
use framework_core::{Error, ErrorKind};
use objc2::rc::autoreleasepool;
use objc2_event_kit::{EKCalendar, EKEntityMask, EKEntityType, EKEvent, EKSpan};
use objc2_foundation::{NSDate, NSString};
use std::string::ToString;
use std::vec::Vec;

impl CalendarEventCreateBackend for IosCalendarBackend {
    fn writable_event_calendars(
        &self,
    ) -> Result<Vec<WritableEventCalendar>, CalendarEventCreateError> {
        require_full_access_for_create(native_authorization_status())?;

        autoreleasepool(|_| {
            // SAFETY: This public synchronous EventKit query is scoped to Event entities and uses
            // the backend's retained store. Current FullAccess was checked immediately above.
            let calendars = unsafe {
                self.native_event_store()
                    .calendarsForEntityType(EKEntityType::Event)
            };
            let mut writable = Vec::new();
            for calendar in calendars.iter() {
                let supports_events = supports_event_calendar(&calendar);
                // SAFETY: The getter reads the current modifiable state of this live calendar.
                let allows_changes = unsafe { calendar.allowsContentModifications() };
                if !supports_events || !allows_changes {
                    continue;
                }
                writable.push(writable_calendar_from_native(&calendar)?);
            }
            Ok(writable)
        })
    }

    fn create_event(
        &self,
        calendar: &WritableEventCalendar,
        draft: &CalendarEventDraft,
    ) -> Result<(), CalendarEventCreateError> {
        if calendar.identifier().is_empty() {
            return Err(CalendarEventCreateError::InvalidCalendarIdentifier);
        }
        if draft.start() >= draft.end() {
            return Err(CalendarEventCreateError::InvalidEventInterval);
        }
        require_full_access_for_create(native_authorization_status())?;

        autoreleasepool(|_| {
            let identifier = NSString::from_str(calendar.identifier());
            // SAFETY: The identifier is a validated caller selection resolved afresh through the
            // exact EKEventStore that owns this backend. No calendar enumeration or fallback occurs.
            let native_calendar = unsafe {
                self.native_event_store()
                    .calendarWithIdentifier(&identifier)
            }
            .ok_or(CalendarEventCreateError::CalendarNotFound)?;

            let supports_events = supports_event_calendar(&native_calendar);
            // SAFETY: This reads the current writability of the same store-owned calendar.
            let allows_changes = unsafe { native_calendar.allowsContentModifications() };
            if !supports_events {
                return Err(CalendarEventCreateError::CalendarDoesNotSupportEvents);
            }
            if !allows_changes {
                return Err(CalendarEventCreateError::CalendarNotWritable);
            }

            let event_store = self.native_event_store();
            // SAFETY: The new event is created in this backend's store and remains retained for
            // the complete field assignment and single save call below.
            let event = unsafe { EKEvent::eventWithEventStore(event_store) };
            let title = NSString::from_str(draft.title());
            let location = draft.location().map(NSString::from_str);
            let notes = draft.notes().map(NSString::from_str);
            let start = NSDate::dateWithTimeIntervalSince1970(draft.start().unix_seconds());
            let end = NSDate::dateWithTimeIntervalSince1970(draft.end().unix_seconds());

            // SAFETY: `native_calendar` and all values belong to the current autorelease pool;
            // setters copy the supplied objects and the event is retained by this scope.
            unsafe {
                event.setCalendar(Some(&native_calendar));
                event.setTitle(Some(&title));
                event.setStartDate(Some(&start));
                event.setEndDate(Some(&end));
                event.setAllDay(draft.is_all_day());
                event.setLocation(location.as_deref());
                event.setNotes(notes.as_deref());
            }

            // SAFETY: This commits exactly the one new event created above. EventKit returns its
            // native NSError on failure; no retry or identifier claim is made.
            let save_result =
                unsafe { event_store.saveEvent_span_commit_error(&event, EKSpan::ThisEvent, true) };
            save_result.map_err(|error| create_error_from_native(error.code()))
        })
    }
}

fn writable_calendar_from_native(
    calendar: &EKCalendar,
) -> Result<WritableEventCalendar, CalendarEventCreateError> {
    // SAFETY: The calendar was returned by the backend's retained event store and remains live
    // through this copy. The portable result owns both strings before the autorelease pool drains.
    let identifier = unsafe { calendar.calendarIdentifier() }.to_string();
    // SAFETY: This copies the visible title of the same live calendar into Rust-owned memory.
    let title = unsafe { calendar.title() }.to_string();
    WritableEventCalendar::new(identifier, title)
}

fn supports_event_calendar(calendar: &EKCalendar) -> bool {
    // SAFETY: The caller passes a live EventKit calendar retained by the backend's store.
    unsafe { calendar.allowedEntityTypes().contains(EKEntityMask::Event) }
}

fn require_full_access_for_create(
    status: CalendarAuthorizationStatus,
) -> Result<(), CalendarEventCreateError> {
    if status == CalendarAuthorizationStatus::FullAccess {
        Ok(())
    } else {
        Err(CalendarEventCreateError::NotFullAccess(status))
    }
}

fn create_error_from_native(code: isize) -> CalendarEventCreateError {
    match error_from_native(code) {
        CalendarError::Backend(error) => CalendarEventCreateError::Backend(error),
        _ => CalendarEventCreateError::Backend(Error::new(ErrorKind::Platform)),
    }
}

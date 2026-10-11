use crate::conversion::{reminder_create_error_from_native, require_full_reminder_access};
use crate::platform::{IosCalendarBackend, native_reminder_authorization_status};
use framework_calendar::{
    CalendarReminderCreateBackend, CalendarReminderCreateError, CalendarReminderDraft,
};
use objc2::rc::autoreleasepool;
use objc2_event_kit::{EKEventStore, EKReminder};
use objc2_foundation::NSString;

impl CalendarReminderCreateBackend for IosCalendarBackend {
    fn create_reminder(
        &self,
        draft: &CalendarReminderDraft,
    ) -> Result<(), CalendarReminderCreateError> {
        if draft.title().is_empty() {
            return Err(CalendarReminderCreateError::EmptyTitle);
        }
        require_full_reminder_access(native_reminder_authorization_status())?;

        autoreleasepool(|_| {
            let event_store: &EKEventStore = self.native_event_store();
            // SAFETY: This public synchronous EventKit accessor selects only the current default
            // Reminders list from the backend's retained store; it does not read reminder items.
            let calendar = unsafe { event_store.defaultCalendarForNewReminders() }
                .ok_or(CalendarReminderCreateError::DefaultListUnavailable)?;

            // SAFETY: The new reminder is created for this backend's live event store and remains
            // retained throughout field assignment and the single save call below.
            let reminder = unsafe { EKReminder::reminderWithEventStore(event_store) };
            let title = NSString::from_str(draft.title());
            // SAFETY: Both the calendar and title remain live through these copy setters.
            unsafe {
                reminder.setCalendar(Some(&calendar));
                reminder.setTitle(Some(&title));
            }

            // SAFETY: This commits exactly one newly created reminder. No lookup, retry, or
            // existing-reminder read is performed.
            unsafe { event_store.saveReminder_commit_error(&reminder, true) }
                .map_err(|error| reminder_create_error_from_native(error.code()))
        })
    }
}

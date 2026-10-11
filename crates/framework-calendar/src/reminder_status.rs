/// The normalized authorization status for Reminders data.
///
/// EventKit exposes full access as the only requestable Reminders permission. Any raw state that
/// does not safely identify the Reminders authorization level maps to `Unknown`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ReminderAuthorizationStatus {
    /// The platform returned an authorization status this contract cannot classify.
    Unknown,
    /// The platform has not yet asked the user for Reminders access.
    NotDetermined,
    /// Access is blocked by platform policy or device restrictions.
    Restricted,
    /// The user denied Reminders access.
    Denied,
    /// The platform grants full read and write access to Reminders data.
    FullAccess,
}

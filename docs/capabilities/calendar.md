# Calendar authorization

**framework-calendar** defines a portable **no_std** contract for Calendar event-authorization status and an explicit full-access request. It contains only framework-owned values and does not probe a device, prompt a user, or create a global service. The separate iOS backend is documented in the [iOS Calendar guide](../ios/calendar.md).

## Rust API

~~~rust
use framework_calendar::{Calendar, CalendarAuthorizationBackend, CalendarAuthorizationStatus};

async fn request<B: CalendarAuthorizationBackend>(
    calendar: &mut Calendar<B>,
) -> Result<CalendarAuthorizationStatus, framework_calendar::CalendarError> {
    calendar.request_full_access().await
}
~~~

`Calendar<B>` owns the explicitly supplied backend. Backend selection is static, and associated futures use `core::future::Future`; there is no boxed trait object, executor, **Send** requirement, registry, or hidden initialization. `authorization_status` is a non-prompting query. Only an explicit `request_full_access` operation may prompt, and a backend must defer native work until that future is first polled.

## Status and result semantics

`CalendarAuthorizationStatus` distinguishes `NotDetermined`, `Restricted`, `Denied`, `WriteOnly`, `FullAccess`, and `Unknown`. `WriteOnly` is not equivalent to `FullAccess`: it reports permission to save new events without full Calendar event-data access. The contract reports a write-only state but does not offer a write-only permission request.

The result of a request is the authorization status queried after native completion, not a callback grant Boolean. Dropping a pending request future abandons its result; it cannot promise to dismiss a native prompt already shown. Native callback state must remain safe until completion and complete at most once. Errors retain their portable `ErrorKind` and optional signed `PlatformErrorCode` through `CalendarError::Backend`.

The portable crate contains no Apple framework types, event values, strings, heap buffers, unsafe code, or platform-specific permission details. It does not fetch or modify events or reminders.

## Scope and checks

This contract covers only status and an explicit full-access request. It does not define event/reminder enumeration, event creation or edits, reminders access, EventKitUI, calendar UI, entitlements, or permission-copy values.

Portable checks:

~~~sh
cargo fmt --all -- --check
cargo test -p framework-calendar
cargo check -p framework-calendar --no-default-features
git diff --check
~~~

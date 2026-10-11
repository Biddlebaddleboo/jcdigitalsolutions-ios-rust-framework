# Calendar authorization

**framework-calendar** defines portable **no_std** authorization and event-range contracts with framework-owned values. It does not probe a device, prompt a user, or create a global service. The separate iOS backend is documented in the [iOS Calendar guide](../ios/calendar.md).

## Rust API

~~~rust
use framework_calendar::{Calendar, CalendarAuthorizationBackend, CalendarAuthorizationStatus};

async fn request<B: CalendarAuthorizationBackend>(
    calendar: &mut Calendar<B>,
) -> Result<CalendarAuthorizationStatus, framework_calendar::CalendarError> {
    calendar.request_full_access().await
}
~~~

`Calendar<B>` owns the explicitly supplied backend. Backend selection is static, and authorization futures use `core::future::Future`; there is no boxed trait object, executor, **Send** requirement, registry, or hidden initialization. `authorization_status` is a non-prompting query. Only an explicit `request_full_access` operation may prompt. Creating a request future, or dropping it before its first poll, starts no backend work; a backend may start prompt-capable work only when that future is first polled.

## Event-range reads

`CalendarEventTime` stores a finite Unix timestamp in `f64` seconds. `CalendarEventRange::new` requires an ordered, non-empty pair of timestamps. This retains the native precision used by iOS `NSDate`/`NSTimeInterval`; it does not round to nanoseconds. `CalendarEvent` owns its title and optional identifier, location, and notes, and records the native start/end timestamps plus the `is_all_day` flag. Absent native fields remain `None`.

`CalendarEventBackend::events_in_range` and `Calendar::events_in_range` are synchronous. A backend must recheck current authorization immediately before the native read, require full event-data access, fail closed for every other state including `WriteOnly`, and never prompt. The call may block; it creates no thread or executor and leaves thread choice to the caller. Native range matching, result order, and date-zone behavior remain backend-defined; the portable contract does not add a second result filter or sort.

On iOS, the backend uses only `EKEventStore`'s event-range predicate and `eventsMatchingPredicate:`. It rejects ranges longer than EventKit's four-year query horizon rather than accept silent truncation. EventKit uses the default time zone for this query. For floating/all-day events, the returned start/end values are EventKit's `NSDate` instants in its default time zone and `is_all_day` remains true; they are not floating local date components. An `eventIdentifier` may be absent and can change after calendar moves or sync, so it is not a durable global identity.

This is read-only access to committed events in the selected time range and all calendars. It does not read reminders, choose calendars, fetch outside the range, request permission, or modify records.

## Status and result semantics

`CalendarAuthorizationStatus` distinguishes `Unknown`, `NotDetermined`, `Restricted`, `Denied`, `WriteOnly`, and `FullAccess`. `Unknown` means the backend could not classify a reported status; it is distinct from `NotDetermined`, which means no user decision has been made. `WriteOnly` is not equivalent to `FullAccess`: it reports permission to save new events without full Calendar event-data access. The contract reports a write-only state but does not offer a write-only permission request.

The result of a request is the authorization status queried after native completion, not a callback grant Boolean. Dropping a pending request future abandons its result; it cannot promise to dismiss a native prompt already shown. Native callback state must remain safe until completion and complete at most once. Errors retain their portable `ErrorKind` and optional signed `PlatformErrorCode` through `CalendarError::Backend`.

The portable crate depends only on `framework-core` and `core` plus `alloc` for owned event text and result vectors. It has no Apple framework types, unsafe code, executor, or platform-specific permission implementation.

## Scope and checks

This contract covers only status and an explicit full-access request. It does not define event/reminder enumeration, event creation or edits, reminders access, EventKitUI, calendar UI, entitlements, or permission-copy values.

Portable checks:

~~~sh
cargo fmt --all -- --check
cargo test -p framework-calendar
cargo check -p framework-calendar --no-default-features
git diff --check
~~~

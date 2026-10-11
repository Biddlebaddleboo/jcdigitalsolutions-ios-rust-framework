# Calendar authorization and events

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

## Selecting a writable calendar and creating one event

`Calendar::writable_event_calendars` returns Rust-owned snapshots containing only the identifier and title of existing event calendars that currently allow modifications. It requires existing `FullAccess` and never requests permission. An empty result means the caller has no writable event calendar to select; the API does not choose a default calendar or show UI. Calendar identifiers are opaque and can become stale after synchronization.

`CalendarEventDraft::new` accepts a caller-owned title, finite ordered start/end timestamps, an all-day flag, and optional location/notes. It rejects equal or reversed times. The text and `f64` timestamps are not trimmed, normalized, rounded, or reconstructed. `Calendar::create_event` accepts one selected snapshot, requires current `FullAccess`, resolves its identifier again, and rejects a missing, non-event, or non-writable calendar before one native save. If the snapshot is stale, callers must list calendars again and select anew. It returns `Result<(), CalendarEventCreateError>`: success means EventKit's save call succeeded, not that an identifier or later sync outcome is durable. Native errors retain `ErrorKind::Platform` and a representable nonzero `PlatformErrorCode`.

Calendar listing and creation are synchronous and may block on the caller's thread. They do not prompt, enumerate events, retry, create worker threads, or return EventKit objects. EventKit save failures, permission changes, and the saved event's runtime contents are not host-test proof.

## Deleting one selected event

`Calendar::delete_event(&CalendarEvent)` accepts a selected summary returned by `events_in_range`. Missing or empty identifiers fail before backend dispatch. The iOS backend rechecks current `FullAccess` without prompting, resolves only that identifier through `eventWithIdentifier:`, and requires the resolved identifier and exact start/end instants to match the selected summary. It then requires the event's attached calendar to support Event entities and currently allow modifications, and calls `removeEvent:span:commit:error:` once with `EKSpan::ThisEvent` and commit enabled. Stale identifiers, snapshots, or read-only calendars fail without removal. Native error kind and a representable platform code are retained.

Apple documents that `eventWithIdentifier:` returns the first occurrence for a recurring identifier. To avoid deleting a different occurrence, the backend fails closed when the resolved date interval differs from the selected summary; it does not enumerate or search for a substitute occurrence. The operation does not remove a series or future occurrences, access reminders, prompt, retry, or show UI. EventKit deletion and runtime state are not proven by host tests.

## Updating one selected event (B485)

`Calendar::update_event(&CalendarEvent, &CalendarEventDraft)` treats the existing `CalendarEventDraft` as a complete replacement for title, start, end, all-day, location, and notes. Optional location/notes set to `None` clear those fields. The operation preserves the event's current calendar; the draft cannot select or move an event to another calendar.

The iOS backend rejects missing/empty identifiers before native work, rechecks current `FullAccess` without prompting, resolves only the selected identifier, and requires the resolved identifier and exact start/end instants to match the selected snapshot before mutation. It then checks the attached calendar's Event support and current modifiability, sets only the six draft fields, and calls `saveEvent:span:commit:error:` exactly once with `EKSpan::ThisEvent` and commit enabled. It does not set the calendar, enumerate or search events, retry, access reminders, edit a series, or show UI. Native error kind and representable code are retained. Because `eventWithIdentifier:` resolves the first occurrence for a recurring identifier, a different selected occurrence fails closed without mutation rather than searching for a substitute.

The operation is synchronous and may block. Host regressions cover invalid identifiers/no dispatch, denied access, stale occurrence/no save, non-event and unwritable calendars, exact replacement fields/current-calendar preservation, one save, and native error-code retention. These checks do not prove live EventKit state or on-device recurrence behavior.

## Status and result semantics

`CalendarAuthorizationStatus` distinguishes `Unknown`, `NotDetermined`, `Restricted`, `Denied`, `WriteOnly`, and `FullAccess`. `Unknown` means the backend could not classify a reported status; it is distinct from `NotDetermined`, which means no user decision has been made. `WriteOnly` is not equivalent to `FullAccess`: it reports permission to save new events without full Calendar event-data access. The contract reports a write-only state but does not offer a write-only permission request.

The result of a request is the authorization status queried after native completion, not a callback grant Boolean. Dropping a pending request future abandons its result; it cannot promise to dismiss a native prompt already shown. Native callback state must remain safe until completion and complete at most once. Errors retain their portable `ErrorKind` and optional signed `PlatformErrorCode` through `CalendarError::Backend`.

The portable crate depends only on `framework-core` and `core` plus `alloc` for owned event text and result vectors. It has no Apple framework types, unsafe code, executor, or platform-specific permission implementation.

## Scope and checks

This contract covers status, an explicit full-access request, bounded event-range reads, a writable event-calendar choice snapshot, creation of one caller-described event, deletion of one selected event, and replacement of six fields on one selected event. It does not define full event enumeration, series edits, reminder access, EventKitUI, calendar UI, entitlements, or permission-copy values.

Portable checks:

~~~sh
cargo fmt --all -- --check
cargo test -p framework-calendar
cargo check -p framework-calendar --no-default-features
git diff --check
~~~

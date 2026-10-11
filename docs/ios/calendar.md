# iOS Calendar authorization

**ios-calendar** implements `framework-calendar` authorization and event-range contracts using public EventKit APIs from Rust. The backend targets iOS 17.0 or later. It exposes non-prompting status, an explicit full-access request, and a synchronous read of events within one caller-selected range.

## Permission declaration and request

The consuming app must provide a meaningful `NSCalendarsFullAccessUsageDescription` string in its `Info.plist` for the full Calendar event-access request. This crate does not edit the host app's property list or assert that a prompt will be presented.

`IosCalendarBackend::new` creates an `EKEventStore` without querying authorization or requesting access. `authorization_status` calls `+[EKEventStore authorizationStatusForEntityType:]` for `EKEntityType::Event` and does not prompt. The explicit `Calendar::request_full_access` path calls `requestFullAccessToEventsWithCompletion:` only when its returned future is first polled. It does not call deprecated `requestAccess(to:completion:)` / `requestAccessToEntityType:completion:`. That legacy API is deprecated starting in iOS 17; on iOS 17 and later it does not prompt and returns an error.

The EventKit request callback can run on an arbitrary queue. The callback copies only its optional native error code into Rust-owned completion state; it does not access the `EKEventStore`. When the future observes callback completion, it re-queries authorization status on the future's poll thread. The callback's grant Boolean is not treated as the current status. Native error codes map to `CalendarError::Backend(ErrorKind::Platform)` when present and representable as a nonzero signed 32-bit value.

## Status mapping and lifecycle

The EventKit mapping is:

| EventKit status | Portable status |
| --- | --- |
| `NotDetermined` | `NotDetermined` |
| `Restricted` | `Restricted` |
| `Denied` | `Denied` |
| `FullAccess` | `FullAccess` |
| `WriteOnly` | `WriteOnly` |
| Unknown raw value | `Unknown` |

`WriteOnly` remains distinct from `FullAccess`, even though this backend does not request write-only permission. The deprecated `Authorized` alias maps to the same native value as `FullAccess` and is not used.

## Event-range read (B465)

`Calendar::events_in_range(range)` creates `NSDate` bounds from the finite caller-provided Unix-second values, creates a predicate with `predicateForEventsWithStartDate:endDate:calendars:` using `nil` to select all calendars, and calls `eventsMatchingPredicate:` on the backend's existing `EKEventStore`. It rechecks `authorizationStatusForEntityType:` immediately before the query and proceeds only for `FullAccess`; `WriteOnly`, denied, restricted, not-determined, and unknown states fail with `CalendarEventError::NotFullAccess`. The read path never requests access. Callers must use the explicit full-access request when they need permission.

`CalendarEventTime` stores the exact finite `NSTimeInterval` value as `f64` Unix seconds. No nanosecond rounding, calendar-zone conversion, or date-component reconstruction occurs. `CalendarEventRange::new` rejects non-finite timestamps and `start >= end`; the iOS backend rejects durations greater than 1461 days because EventKit limits this predicate to four years and otherwise shortens the query. EventKit owns the exact matching boundary semantics and result order; the Rust backend does not sort or apply a second filter. EventKit includes only committed events in this query.

Each result copies `eventIdentifier`, title, start/end, `isAllDay`, location, and notes into Rust-owned values. A missing identifier/location/notes value remains `None`. Native identifiers can be absent or change after calendar moves or sync and are not durable global IDs. All-day events retain EventKit's returned `NSDate` instants and an explicit `is_all_day` flag; EventKit returns floating/all-day dates in its default time zone, so the values are not local floating-date components. The API does not infer a time zone or synthesize midnight boundaries.

EventKit's `eventsMatchingPredicate:` is synchronous and may block. The API starts no worker, executor, or dispatch queue and imposes no main-thread marker; the caller chooses whether to invoke the read off-main. The same `EKEventStore` remains owned by its non-`Send` backend value. This does not claim that a query has run on a device.

The future starts the native request on first poll. Dropping it before first poll starts no request. Dropping it after start detaches Rust result interest; it cannot promise to dismiss an operating-system prompt. EventKit retains the copied completion block for its asynchronous callback, and callback state is detached and completed at most once. No main-thread requirement is imposed by this backend; the native store remains on the non-`Send` backend value, while callback work is synchronized independently.

`IosCalendarBackend::native_event_store` is a narrow iOS-only escape hatch. Direct native operations may observe or change EventKit state independently of this contract.

## API floor, exclusions, and evidence limits

The local iPhoneOS 26.5 SDK marks `requestFullAccessToEventsWithCompletion:` available from iOS 17.0. The package does not add a runtime fallback for older operating systems; apps using this package must target iOS 17.0 or later. The backend does not request reminders or write-only event permission, read reminders, save/edit/delete events, display EventKitUI or calendar UI, add entitlements, or ship Swift source.

Host conversion tests verify status gating, finite ranges, EventKit's maximum query horizon, fractional-second retention, nullable fields, all-day flag retention, malformed native-date failure, and the portable facade. Device/Simulator compilation and Release link/import/deployment probes verify target compilation and static linkage only. They do not exercise a live authorization prompt, a user's choice, event-store contents, returned EventKit data, or runtime behavior on a device.

Apple references: [`EKEventStore` access and request guidance](https://developer.apple.com/documentation/eventkit/accessing-the-event-store?changes=__3%2C__3), [`EKAuthorizationStatus`](https://developer.apple.com/documentation/eventkit/ekauthorizationstatus?changes=_5%2C_5), [retrieving events and reminders](https://developer.apple.com/documentation/eventkit/retrieving-events-and-reminders?changes=_9), [`eventsMatchingPredicate:`](https://developer.apple.com/documentation/eventkit/ekeventstore/events%28matching%3A%29?language=objc), and [Calendar access with EventKit](https://developer.apple.com/documentation/eventkit/accessing-calendar-using-eventkit-and-eventkitui?changes=_7).

Binding: `objc2-event-kit` 0.3.2 with only `EKCalendar`, `EKCalendarItem`, `EKEvent`, `EKEventStore`, `EKObject`, `EKTypes`, and `block2` features. The generated feature graph leaves reminder, calendar-UI, and unrelated binding surfaces disabled.

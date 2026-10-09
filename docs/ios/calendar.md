# iOS Calendar authorization

**ios-calendar** implements the `framework-calendar` authorization contract using public EventKit APIs from Rust. The backend targets iOS 17.0 or later and exposes only Calendar event-authorization status plus an explicit full-access request. It does not expose event or reminder records.

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

The future starts the native request on first poll. Dropping it before first poll starts no request. Dropping it after start detaches Rust result interest; it cannot promise to dismiss an operating-system prompt. EventKit retains the copied completion block for its asynchronous callback, and callback state is detached and completed at most once. No main-thread requirement is imposed by this backend; the native store remains on the non-`Send` backend value, while callback work is synchronized independently.

`IosCalendarBackend::native_event_store` is a narrow iOS-only escape hatch. Direct native operations may observe or change EventKit state independently of this contract.

## API floor, exclusions, and evidence limits

The local iPhoneOS 26.5 SDK marks `requestFullAccessToEventsWithCompletion:` available from iOS 17.0. The package does not add a runtime fallback for older operating systems; apps using the request path must target iOS 17.0 or later. The backend does not request reminders or write-only event permission, enumerate or fetch events, save or edit events, display EventKitUI or calendar UI, add entitlements, or ship Swift source.

Host conversion and callback-state tests verify mapping, exactly-once result delivery, and result detachment. Device/simulator Cargo checks and import inspection verify target compilation/linkage only. They do not exercise a live authorization prompt, a user's choice, event-store contents, or runtime behavior on a device.

Apple references: [`EKEventStore` access and request guidance](https://developer.apple.com/documentation/eventkit/accessing-the-event-store?changes=__3%2C__3), [`EKAuthorizationStatus`](https://developer.apple.com/documentation/eventkit/ekauthorizationstatus?changes=_5%2C_5), and [Calendar access with EventKit](https://developer.apple.com/documentation/eventkit/accessing-calendar-using-eventkit-and-eventkitui?changes=_7).

Binding: `objc2-event-kit` 0.3.2 with only `EKEventStore`, `EKTypes`, and `block2` features. The EventKit generated feature graph keeps unrelated event, reminder, calendar, and UI binding surfaces disabled.

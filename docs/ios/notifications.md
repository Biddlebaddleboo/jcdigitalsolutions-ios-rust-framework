# iOS local notifications

`ios-notifications` implements the portable [`framework-notifications`](../notifications.md)
contract with the app's shared `UNUserNotificationCenter`. It uses public `UserNotifications` and
Foundation APIs from Rust; it adds no Swift source, delegate, process-wide Rust registry, or
executor.

```rust,ignore
use framework_notifications::{
    Notification, NotificationContent, NotificationId, NotificationTrigger, Notifications,
};
use ios_notifications::IosNotificationsBackend;
use alloc::string::String;

async fn schedule_reminder(
    trigger_at_unix_millis: u64,
) -> Result<(), framework_notifications::NotificationError> {
    let mut notifications = Notifications::new(IosNotificationsBackend::new());
    let state = notifications.authorization().await?;
    if state == framework_core::AuthorizationState::NotDetermined {
        notifications.request_authorization().await?;
    }
    let request = Notification::new(
        NotificationId::new(String::from("daily-reminder"))?,
        NotificationContent::new(
            String::from("Reminder"),
            Some(String::from("Review today's notes")),
        ),
        NotificationTrigger::at_unix_millis(trigger_at_unix_millis)?,
    );
    notifications.schedule(request).await
}
```

The app supplies its executor or polls these executor-neutral futures itself. `IosNotificationsBackend::new()` obtains the shared notification-center object only; it does not query permission, show a prompt, or schedule work. `authorization()` queries without prompting. `request_authorization()` asks for alert authorization only, and the prompt-capable call begins when that future is first polled. The result is read back from notification settings after the native authorization callback.

## Availability and authorization

`UserNotifications` center, authorization-state query, request, schedule, pending-request count, and
calendar-trigger APIs in this backend have an iOS 10.0 floor per Xcode 26.6 iPhoneOS SDK 26.5. `UNAuthorizationStatusProvisional`
has an iOS 12.0 floor; `UNAuthorizationStatusEphemeral` has an iOS 14.0 floor

Native `NotDetermined` and `Denied` map to the matching `framework_core::AuthorizationState`.
Native `Authorized`, `Provisional`, and `Ephemeral` map to portable `Authorized`; this intentionally
loses the provisional/temporary distinction and does not imply that banners, lock-screen display,
or sound are enabled. `UserNotifications` has no `Restricted` authorization status, so that portable
state is not synthesized; unknown native raw values map to `Unknown`. The backend does not request
provisional authorization. It asks only for alerts because D3 models title/body text and has no
sound or badge fields.

Local notification authorization does not require a notification-specific `Info.plist` usage
description or push entitlement. The app should request authorization in a user-understandable
context. This backend does not register for remote notifications and has no APNs, PushKit, category,
action, response-delegate, or notification-service-extension behavior.

## Request and trigger semantics

- Caller IDs are passed unchanged through `NSString`; the backend adds no prefix, truncation, or
  normalization. D3's constructor remains responsible for rejecting empty and NUL-containing IDs.
- Title and optional body strings are copied into `UNMutableNotificationContent`. Actions,
  categories, attachments, sounds, and badges are not represented.
- Immediate triggers and timestamps already in the past use a nil native trigger, which requests
  immediate delivery. Future absolute Unix-millisecond timestamps become non-repeating calendar
  triggers with Gregorian calendar and UTC time zone, including the millisecond fraction in the
  nanosecond date component. This avoids dependence on the device's current calendar or time zone.
- The backend supports UTC dates through `9999-12-31 23:59:59.999` (`253402300799999` Unix
  milliseconds). A larger timestamp that is valid under D3's signed-64-bit bound returns
  `NotificationError::Backend` with `ErrorKind::Unsupported`; it is never wrapped, clamped, or
  changed to a relative delay. Calendar conversion fixtures cover epoch, a leap day, fractional
  milliseconds, due/past timestamps, and the backend upper bound.
- `UNNotificationRequest` uses the exact caller ID. Apple documents that adding a request with the
  same ID replaces its previous pending request, matching D3 replacement behavior.
- Cancellation fetches the app's pending requests, compares their identifiers, and removes the ID
  when present. It returns `true` for an ID observed in that query and `false` otherwise; the bool
  is not confirmation of an atomic removal. The public remove API has no completion or
  prior-existence result, so this is a best-effort query-then-remove sequence, not an atomic system
  transaction. Calls made directly through `native_notification_center()` can race that sequence.
  Cancellation does not remove already delivered notifications.

The iOS-only `IosNotificationsBackend::pending_request_count` future gives a `u64` count of this
app's local-request queue at native callback time. Apple's [pending-request API](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter/getpendingnotificationrequests%28completionhandler%3A%29)
docs say an empty array means zero and the callback block may run on a background thread. A native
add or cancel can alter the queue at once; treat this count as a snapshot, not a stable fact. The
count has no IDs or content, does not cover a notice once delivery occurs, and proves no
authorization or readiness. A future drop after first poll ends result interest only; the native
call still runs to its callback

The iOS system owns its pending-request capacity and delivery policy. A successful `schedule` means
the native add API completed without an `NSError`, not that the request remains pending, is delivered
at a precise time, or is shown to the user. The backend does not prompt or gate scheduling on
authorization; callers can query/request permission separately. iOS may suppress presentation based
on authorization and notification settings.

## Async, ownership, and errors

Every backend operation starts on first poll. Dropping an unpolled future makes no native call.
Dropping a started future detaches its waker and result interest but does not cancel a prompt,
settings query, schedule request, or pending-request query. Each escaping native block owns an
`Arc`-backed completion cell; it accepts one terminal result, wakes outside its mutex, and discards
late results after detachment. Rust panics are caught in native block callbacks and map to
`ErrorKind::Internal`, so unwinding does not cross an Objective-C block boundary. UserNotifications
chooses callback queues; callers should not assume callbacks run on the main thread.

Authorization denial is returned as `AuthorizationState::Denied`, not as an operation error.
Native `NSError` values map to `ErrorKind::Platform`; the signed native code is retained when it fits
`PlatformErrorCode(i32)`. The error domain is not retained because the current portable error type
has no domain field. A clock before the Unix epoch maps to `Unavailable`. Unsupported future dates
map to `Unsupported`; a native queue count that cannot fit `u64` maps to `ResourceExhausted`;
unexpected Rust panics map to `Internal`.

## Dependency and linkage

The iOS-only crate enables `objc2-user-notifications` 0.3.2 with default features disabled and only
the center, settings, content, request, trigger, and `block2` features. The matching framework
bindings avoid hand-written Objective-C selector declarations and keep the public OS surface typed.
`objc2`, `objc2-foundation`, and `block2` are already workspace dependencies; Foundation date
components and time-zone features are enabled only for this capability. The UserNotifications
binding's optional Core Location feature and unrelated categories, actions, attachments, and service
extension bindings remain disabled. No UserNotifications type enters the portable D3 contract, and
replacing the binding crate is confined to this backend adapter.

Release linkage should include the public `UserNotifications` and Foundation frameworks plus the
Objective-C runtime support required by `objc2`; it must not import UIKit or a Swift runtime. No
physical-device prompt/delivery or simulator-delivery behavior is claimed by automated checks.

Run `sh platform/ios/ios-notifications/check-link-imports.sh` on macOS for a Release probe on each
target. The gate checks the exact direct imports `Foundation`, `UserNotifications`,
`libSystem.B.dylib`, and `libobjc.A.dylib`, then scans undefined symbols for Swift/Python runtime
and selected unrelated capability imports. It also checks that the compile-only probe retains
`pending_request_count()`. It does not run the probes or request permission, schedule, cancel, count
live requests, or deliver a notification

After root refreshed `Cargo.lock` and added the pending-request count future, the updated
2026-10-09 gate passed for `aarch64-apple-ios` and `aarch64-apple-ios-sim`. Both exact import lists were `Foundation`, `UserNotifications`,
`libSystem.B.dylib`, and `libobjc.A.dylib`; the `nm -u` denylist passed on both targets. The probe
binaries were inspected but not executed, so this check makes no runtime claim
The macOS CI workflow now runs this gate; no passing workflow run is recorded

The first attempt exited 101 before either target build because `--locked` required a shared
`Cargo.lock` refresh; no probe was built by that attempt

References: [Apple's UserNotifications center](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter), [authorization request guidance](https://developer.apple.com/documentation/usernotifications/asking-permission-to-use-notifications), [authorization status](https://developer.apple.com/documentation/usernotifications/unauthorizationstatus), [absolute calendar triggers](https://developer.apple.com/documentation/usernotifications/uncalendarnotificationtrigger), [request identifier replacement](https://developer.apple.com/documentation/usernotifications/unnotificationrequest/identifier), and [objc2 UserNotifications bindings](https://docs.rs/objc2-user-notifications/0.3.2/objc2_user_notifications/).

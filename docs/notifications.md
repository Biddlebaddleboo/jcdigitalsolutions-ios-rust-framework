# Local notifications

`framework-notifications` defines a portable, `no_std` + `alloc` contract for owned local-notification content, optional system-default sound and badge count, an optional category identifier, and immediate, absolute-time, repeating-interval, or daily local-clock scheduling. The D3 crate does not link `UserNotifications` or implement a native backend; B4's iOS adapter is documented in the [iOS local-notifications guide](ios/notifications.md). Neither crate implements remote push/APNs/PushKit.

D9 adds portable [notification interaction response values](notification-responses.md). Those values do not add a delivery callback, delegate, or response backend; B12 owns native response delivery.

## Rust API

```rust
use alloc::string::String;
use framework_notifications::{
    Notification, NotificationContent, NotificationId, NotificationTrigger,
};

let identifier = NotificationId::new(String::from("daily-reminder"))?;
let content = NotificationContent::new(
    String::from("Reminder"),
    Some(String::from("Review today's notes")),
);
let trigger = NotificationTrigger::at_unix_millis(1_700_000_000_000)?;
let request = Notification::new(identifier, content, trigger);
# let _ = request;
# Ok::<(), framework_notifications::NotificationError>(())
```

`NotificationId` and `NotificationCategoryId` own exact, case-sensitive UTF-8 strings. Empty values and values containing NUL are invalid; neither type normalizes or truncates identifiers. `NotificationContent` owns its title and optional body; `with_system_default_sound()` opts into the system default sound, `with_badge_count(u32)` records an optional badge value, and `with_category_identifier` opts into a category ID. `None` leaves the badge and native category unchanged; zero is preserved as an explicit badge-clear request. Action definitions, attachments, custom sounds, and other platform-specific presentation fields are not modeled by the portable contract. The iOS backend can register action categories separately.

`NotificationTrigger` supports immediate eligibility, an absolute Unix timestamp in milliseconds from zero through `MAX_UNIX_TIMESTAMP_MILLIS` (`i64::MAX`), a repeating whole-second interval from `MIN_REPEAT_INTERVAL_SECONDS` (60) through `u32::MAX`, or `daily_at_local_time(hour, minute, second)` with hour 0–23 and minute/second 0–59. The minimum matches iOS's public `UNTimeIntervalNotificationTrigger` requirement. Daily local time does not encode a calendar or time zone; each backend interprets the clock fields using its local calendar rules. On iOS, the adapter uses Foundation's autoupdating current calendar and public `UNCalendarNotificationTrigger`. Calendar/time-zone changes, daylight-saving gaps or overlaps, and exact delivery timing remain system-defined; the portable API makes no next-occurrence promise. Repeat timing is system-scheduled, not exact. A timestamp in the past is eligible as soon as a backend permits, but no exact delivery time is promised. A backend with a smaller date range must report the limitation rather than silently change the timestamp. Cancel the notification ID to stop a repeating or daily request.

## Backend and operation semantics

Implement `NotificationBackend` for a backend selected by Rust's type system and pass it to `Notifications::new`. The facade adds no global service lookup, initialization, executor, scheduler, or dynamic dispatch. It exposes availability, a non-prompting authorization query, an authorization request, schedule, and cancel.

Authorization uses the framework-owned `framework_core::AuthorizationState` values (`Unknown`, `NotDetermined`, `Denied`, `Restricted`, and `Authorized`), not an operating-system status type. Query does not prompt. An authorization request may prompt only when an implemented native backend is called; this portable crate has no prompt side effect.

The future returned by each facade operation starts backend work on first poll. A started operation has one terminal success or error result. Dropping the caller-facing future drops interest in that result; it does not cancel a started backend operation. A backend that uses callbacks must retain callback state safely until the operation's one terminal outcome. `cancel(id)` is a separate operation that requests removal of a pending scheduled request and reports whether the backend observed one at its documented cancellation point. A backend whose native API separates lookup from removal must document the race; the boolean does not promise an atomic compare-and-remove. It does not promise to withdraw a notification already presented or delivered.

Scheduling a pending request with an identifier already in use replaces the existing pending request; it must not create a duplicate. When requests for one ID race through multiple clients, the backend's serialization order decides which successful schedule remains. A successful schedule means the backend accepted the request, not that the operating system or user will deliver or display it. Authorization state and scheduling are separate operations; a backend defines its error behavior when authorization is insufficient.

Backend futures are executor-neutral and do not require `Send`. Wake behavior and any native callback queue are backend details and must be documented by an implementation. The `backend()` and `backend_mut()` accessors preserve a route to backend-specific options or native handles without adding platform types to this portable crate.

## Scope and support

This contract is portable and contains no iOS, Android, desktop, or web types. The portable crate makes no platform availability claim. The iOS adapter uses `UserNotifications` from iOS 10.0; compile and import evidence does not establish live prompt or delivery behavior. Push notifications/APNs/PushKit, attachments, calendar triggers, critical alerts, custom sounds, and other platform-specific presentation remain outside D3 scope. Badge count is content only: it does not request authorization or establish delivery or app-icon state. A category identifier is only a routing value until an iOS category is registered; it does not define actions or promise they will be presented. Response values are portable; D3 does not deliver responses or own a delegate. The separate opt-in [B12 iOS bridge](ios/notification-responses.md) maps valid local responses to D9 values

## Costs and validation

Identifiers, titles, and bodies use owned `String` storage. `Notification` moves those allocations into the backend call; the facade adds no content copy. Absolute timestamps use a fixed-width `u64` API value with a signed-64-bit upper bound, and repeat intervals use a fixed-width `u32` count of seconds; no pointer-width value is part of the contract.

Portable checks for this crate:

```sh
cargo fmt --all -- --check
cargo test -p framework-notifications
cargo check -p framework-notifications --no-default-features
git diff --check
```

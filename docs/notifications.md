# Local notifications

`framework-notifications` defines a portable, `no_std` + `alloc` contract for owned local-notification content and one-shot scheduling. The D3 crate does not link `UserNotifications` or implement a native backend; B4's iOS adapter is documented in the [iOS local-notifications guide](ios/notifications.md). Neither crate implements remote push/APNs/PushKit.

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

`NotificationId` owns an exact, case-sensitive UTF-8 string. An empty ID or one containing NUL is invalid. The crate does not normalize or truncate identifiers. `NotificationContent` owns its title and optional body; it does not model actions, categories, attachments, sounds, badges, or other platform-specific presentation fields.

`NotificationTrigger` is deliberately bounded to `Immediate` or an absolute Unix timestamp in milliseconds from zero through `MAX_UNIX_TIMESTAMP_MILLIS` (`i64::MAX`). Repeating, calendar, relative-delay, and interval triggers are outside this contract. A timestamp in the past is eligible as soon as a backend permits, but no exact delivery time is promised. A backend with a smaller date range must report the limitation rather than silently change the timestamp.

## Backend and operation semantics

Implement `NotificationBackend` for a backend selected by Rust's type system and pass it to `Notifications::new`. The facade adds no global service lookup, initialization, executor, scheduler, or dynamic dispatch. It exposes availability, a non-prompting authorization query, an authorization request, schedule, and cancel.

Authorization uses the framework-owned `framework_core::AuthorizationState` values (`Unknown`, `NotDetermined`, `Denied`, `Restricted`, and `Authorized`), not an operating-system status type. Query does not prompt. An authorization request may prompt only when an implemented native backend is called; this portable crate has no prompt side effect.

The future returned by each facade operation starts backend work on first poll. A started operation has one terminal success or error result. Dropping the caller-facing future drops interest in that result; it does not cancel a started backend operation. A backend that uses callbacks must retain callback state safely until the operation's one terminal outcome. `cancel(id)` is a separate operation that removes a pending scheduled request and returns whether one existed. It does not promise to withdraw a notification already presented or delivered.

Scheduling a pending request with an identifier already in use replaces the existing pending request; it must not create a duplicate. When requests for one ID race through multiple clients, the backend's serialization order decides which successful schedule remains. A successful schedule means the backend accepted the request, not that the operating system or user will deliver or display it. Authorization state and scheduling are separate operations; a backend defines its error behavior when authorization is insufficient.

Backend futures are executor-neutral and do not require `Send`. Wake behavior and any native callback queue are backend details and must be documented by an implementation. The `backend()` and `backend_mut()` accessors preserve a route to backend-specific options or native handles without adding platform types to this portable crate.

## Scope and support

This contract is portable and contains no iOS, Android, desktop, or web types. The portable crate makes no platform availability claim. The iOS adapter uses `UserNotifications` from iOS 10.0; compile and import evidence does not establish live prompt or delivery behavior. Push notifications, notification responses/delegates, categories/actions, attachments, repeating/calendar triggers, critical alerts, and badge APIs remain out of scope.

## Costs and validation

Identifiers, titles, and bodies use owned `String` storage. `Notification` moves those allocations into the backend call; the facade adds no content copy. Absolute timestamps use a fixed-width `u64` API value with a signed-64-bit upper bound; no pointer-width value is part of the contract.

Portable checks for this crate:

```sh
cargo fmt --all -- --check
cargo test -p framework-notifications
cargo check -p framework-notifications --no-default-features
git diff --check
```

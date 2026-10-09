# iOS local notification responses

`ios-notification-responses` is an opt-in bridge from the iOS UserNotifications response API to D9's owned `NotificationResponse` value

```rust,ignore
use ios_notification_responses::IosNotificationResponses;

let responses = IosNotificationResponses::install(|response| {
    // Move or read the owned D9 response value here
})?;
```

Keep `responses` alive while the app needs local notification responses. Its strong `Retained` field keeps the Rust delegate alive while the native center holds only a weak `delegate` reference. Drop of `responses` releases that delegate and does not clear a delegate set by app code after this install

## Install and delegate ownership

Call `IosNotificationResponses::install` before app launch completes, such as from app launch setup; if the shared `UNUserNotificationCenter` already has a delegate, install returns `InstallError::DelegateAlreadySet` and leaves that delegate as is

Apple marks `UNUserNotificationCenter.delegate` weak and non-atomic; the nil check and set are not an atomic compare-and-set; the app must serialize install with all reads and writes of that shared property; unsynchronized external mutation may race the check and set and is outside this API contract

The returned value strongly retains the Rust delegate. The delegate stores the response closure in `Arc` state and requires `Send + Sync + 'static`. The native API page does not name a callback queue, so the closure runs synchronously on the native callback thread with no main-thread promise. Do not use UIKit from this closure unless the app has a separate main-thread handoff

The bridge takes a strong self retain before it reads delegate state and keeps that retain through response conversion, the closure call, and native completion. The closure may drop the public handle or replace the center delegate without freeing this delegate mid-call

## Value map

- Apple's `UNNotificationDefaultActionIdentifier` maps to `NotificationResponseKind::Default`
- Apple's `UNNotificationDismissActionIdentifier` maps to `NotificationResponseKind::Dismiss`
- `UNTextInputNotificationResponse.userText` maps to `NotificationResponseKind::TextInput` with its exact action ID and owned text
- The notification request's exact `identifier` maps to D3's `NotificationId`

The bridge maps Apple's default and dismiss IDs first; all other action IDs map to `CustomAction` unless the native response is `UNTextInputNotificationResponse`

The bridge copies UTF-16 code units into owned Rust UTF-8 without normalization; an unpaired surrogate cannot form a Rust `String`, so that response is dropped; D9 rejects empty notification IDs or action IDs and IDs with NUL, so a response with one of those values is dropped; empty text is kept, as D9 permits it

`UNPushNotificationTrigger` responses are ignored, so this crate does not route remote-push responses into D9 local values. It does not register APNs, call a permission API, schedule notifications, or define categories/actions. A custom or text-input action needs app-owned category setup outside this crate

The crate uses `objc2-user-notifications` 0.3.2 with default features off and only `block2`, `UNNotification`, `UNNotificationRequest`, `UNNotificationResponse`, `UNNotificationTrigger`, and `UNUserNotificationCenter`. `UNNotificationTrigger` permits the `UNPushNotificationTrigger` check

## Completion, queue, and limits

Apple requires the native completion block after response work; the bridge invokes it exactly once for each native method call, even after it drops an invalid or remote value or catches a Rust panic; Rust panic payloads are discarded at this boundary

The closure call is synchronous and runs before native completion. Do not block for long work. This crate has no event queue, async callback, foreground presentation method, action/category setup, app-settings hook, APNs path, delivery promise, or response-order guarantee across app states. It adds no serial callback queue; app code must protect any state shared with the closure. The callback may only occur when iOS reports a notification action to the app

## Availability

The `UNUserNotificationCenter`, `UNUserNotificationCenterDelegate`, and `didReceiveNotificationResponse` APIs have an iOS 10.0 floor in the Xcode 26.6 iPhoneOS 26.5 SDK headers; Apple says to set the delegate before launch completes; the delegate property is weak, and the relevant Apple callback docs do not specify its queue

References: [delegate protocol](https://developer.apple.com/documentation/usernotifications/unusernotificationcenterdelegate), [response method](https://developer.apple.com/documentation/usernotifications/unusernotificationcenterdelegate/usernotificationcenter%28_%3Adidreceive%3Awithcompletionhandler%3A%29), [delegate property](https://developer.apple.com/documentation/usernotifications/unusernotificationcenter/delegate), [response values](https://developer.apple.com/documentation/usernotifications/unnotificationresponse), [text input value](https://developer.apple.com/documentation/usernotifications/untextinputnotificationresponse/usertext), [remote trigger](https://developer.apple.com/documentation/usernotifications/unpushnotificationtrigger), and [objc2 UserNotifications bindings](https://docs.rs/objc2-user-notifications/0.3.2/objc2_user_notifications/)

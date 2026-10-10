# iOS app scene activity and lifecycle

`ios-app-scenes` provides two independent iOS 13.0 surfaces. B81's synchronous snapshot remains a count-only query; B454 adds an event stream for per-scene lifecycle notifications. Neither changes the portable `framework-app` API.

## B81 activity snapshot

The snapshot reads `UIApplication.connectedScenes` once and each returned `UIScene.activationState` once on the main thread:

```rust,ignore
let main_thread = ios_runtime::main_thread::MainThread::current().ok_or_else(|| {
    framework_core::Error::new(framework_core::ErrorKind::Unavailable)
})?;
let snapshot = ios_app_scenes::snapshot(main_thread);
let active_scenes = snapshot.foreground_active_count();
```

It returns only counts, makes no atomic cross-scene or after-return guarantee, and does not expose scene identifiers, names, native handles, or callback state. A connected scene can be foreground or background and on or offscreen. An empty set is valid. B81 does not subscribe to notifications or alter the snapshot contract.

## B454 per-scene lifecycle events

Create the observer on the main thread, poll without waiting, and inspect the overflow counter:

```rust,ignore
let mut events = ios_app_scenes::IosSceneLifecycleEvents::new(main_thread);
while let Some(event) = events.poll_event() {
    let kind = event.kind();
    let session_id = event.session_identifier().to_string_lossless();
}
let dropped = events.dropped_event_count();
```

The event source listens for UIKit's `UISceneWillConnectNotification`, `UISceneDidDisconnectNotification`, `UISceneDidActivateNotification`, `UISceneWillDeactivateNotification`, `UISceneWillEnterForegroundNotification`, and `UISceneDidEnterBackgroundNotification`. UIKit puts the originating `UIScene` in each notification's `object`; the event owns the exact UTF-16 code units of `UISceneSession.persistentIdentifier`. `to_string_lossless()` returns `None` only for malformed UTF-16. `as_utf16_code_units()` remains available for exact round trips, including embedded NULs and unpaired surrogates.

Observer blocks run on `NSOperationQueue::mainQueue()`. The event source owns and removes every observer token on drop. Blocks capture only weak event-queue state and contain panics before returning through Objective-C. The queue holds 32 events; when full, it drops the oldest event. `dropped_event_count()` reports the cumulative saturating count so callers can detect lost transition history. There is no initial event and no replay of notifications posted before construction.

These are the notifications UIKit posts; this API does not install or replace a `UISceneDelegate`, configure scenes, create windows, present UI, control a scene, or perform any app/runtime initialization. It does not expose capture or media data, promise event delivery while the process is suspended, or guarantee the scene's state after an event. The host remains responsible for its own scene configuration. `UIApplication.sharedApplication` is unavailable to app extensions, so this package targets apps only.

The public APIs and notification constants have an iOS 13.0 floor in the installed iPhoneOS 26.5 SDK. The Rust target's effective minimum is iOS 13.0 for device and 14.0 for Simulator in the registered link probe. No permission, entitlement, or usage-description key is required.

The adapter uses generated `objc2-ui-kit 0.3.2` and `objc2-foundation 0.3.2` bindings, plus `block2 0.6.2`; it adds no Swift source and no hand-written Objective-C ABI. UIKit delivery, multi-scene ordering, suspend/resume delivery, Simulator runtime, and physical-device runtime remain unverified by host tests or compile/link checks.

## Evidence

- iOS 26.5 SDK `UIKit.framework/Headers/UIScene.h:106-111` declares all six notification names with `API_AVAILABLE(ios(13.0))`.
- iOS 26.5 SDK `UIKit.framework/Headers/UISceneSession.h:29` declares `persistentIdentifier`.
- iOS 26.5 SDK `Foundation.framework/Headers/NSString.h:131` declares `getCharacters:range:` for exact UTF-16 copying.
- Apple's [`UIScene.willConnectNotification`](https://developer.apple.com/documentation/uikit/uiscene/willconnectnotification), [`UIScene.didActivateNotification`](https://developer.apple.com/documentation/uikit/uiscene/didactivatenotification), and [`UIScene.didDisconnectNotification`](https://developer.apple.com/documentation/uikit/uiscene/diddisconnectnotification) docs specify the scene as the notification object.
- Apple's [`NotificationCenter.addObserver(forName:object:queue:using:)`](https://developer.apple.com/documentation/foundation/notificationcenter/addobserver%28forname%3Aobject%3Aqueue%3Ausing%3A%29) docs specify that the observer block runs on the selected operation queue and must be removed with its token.

Run `sh platform/ios/ios-app-scenes/check.sh` for target, Clippy, rustdoc, and link/import checks; run `cargo +1.94.1 test --locked -p ios-app-scenes` for deterministic host tests. Link/import checks are not runtime proof.

See [PLAN_IOS_SCENE_SNAPSHOT.md](../../PLAN_IOS_SCENE_SNAPSHOT.md) and [PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md](../../PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md) for B81's original snapshot scope and validation contract.

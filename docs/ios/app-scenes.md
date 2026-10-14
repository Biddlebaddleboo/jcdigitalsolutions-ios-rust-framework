# iOS app scene activity and lifecycle

`ios-app-scenes` provides three independent iOS 13.0 surfaces. B81's synchronous snapshot remains a count-only query; B454 adds an event stream for per-scene lifecycle notifications; B526 adds current activation-state records keyed by exact session identity; B552 adds an optional direct Rust handler for lifecycle events. None changes the portable `framework-app` API.

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

## B526 per-scene activation snapshot

Read the current connected window scenes on UIKit's main thread and match each record to later B454 events by session identifier:

```rust,ignore
let scenes = ios_app_scenes::scene_activation_snapshot(main_thread);
for scene in scenes {
    let session_id = scene.session_identifier();
    let activation = scene.state();
}
```

Each record owns the exact UTF-16 code units of `UISceneSession.persistentIdentifier` and one typed `UISceneActivationState` mapping: `Unattached`, `ForegroundActive`, `ForegroundInactive`, `Background`, or `Unknown(raw_value)`. Only connected `UIWindowScene` instances are included; non-window scenes are omitted. The query retains no UIKit object, and order is unspecified. It samples each activation state once in one synchronous main-thread pass; it is not an atomic cross-scene instant and says nothing about state after return. The result supplies initial per-scene state; B454 supplies subsequent notifications. It does not synthesize an initial event or change B454 queue/lifecycle semantics.

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

Observer blocks run on `NSOperationQueue::mainQueue()`. The event source owns and removes every observer token on drop. Blocks capture only weak event-queue and handler state and contain panics before returning through Objective-C. The queue holds 32 events; when full, it drops the oldest event. `dropped_event_count()` reports the cumulative saturating count so callers can detect lost transition history. There is no initial event and no replay of notifications posted before construction.

### B552 direct event handler

Use `set_event_handler` when a main-thread scene transition should update application-owned policy directly instead of requiring a polling loop:

```rust,ignore
let mut events = ios_app_scenes::IosSceneLifecycleEvents::new(main_thread);
events.set_event_handler(|event| match event.kind() {
    ios_app_scenes::SceneLifecycleEventKind::DidEnterBackground => pause_app_work(),
    ios_app_scenes::SceneLifecycleEventKind::DidActivate => resume_app_work(),
    _ => {}
});
```

The handler receives each ordinary live notification after the same event has entered the polling queue. Polling-queue overflow does not suppress direct handler delivery; both paths preserve the event's exact UTF-16 session identifier and typed transition. Calling `set_event_handler` again replaces the callback, and `clear_event_handler` removes it without clearing already-polled events. The handler is optional and no notification is replayed during construction or registration.

The callback runs on `NSOperationQueue::mainQueue()` and must be `Send + 'static` so its synchronized `Arc<Mutex<...>>` capture satisfies `NSNotificationCenter::addObserverForName:object:queue:usingBlock:`'s sendable-block safety contract. It accepts an owned `SceneLifecycleEvent`; the callback may retain it. Panics are contained before returning through Objective-C, and the installed handler remains available for later notifications. If handling one event synchronously causes nested scene notifications, the same `FnMut` is not invoked recursively: nested events wait until the current call returns and then drain in main-queue order. This reentrant backlog is independently bounded at 32 events, drops oldest on overflow, and is measured by the saturating `dropped_handler_event_count()`. This counter is separate from `dropped_event_count()`, which measures the poll queue only.

The handler is app policy only. It does not install or replace a scene delegate, create a framework runtime, add an executor, change scene setup, or run while the process is suspended. Host tests and static compiler/link checks do not prove UIKit callback timing or live device/Simulator behavior.

These are the notifications UIKit posts; this API does not install or replace a `UISceneDelegate`, configure scenes, create windows, present UI, control a scene, or perform any app/runtime initialization. It does not expose capture or media data, promise event delivery while the process is suspended, or guarantee the scene's state after an event. The host remains responsible for its own scene configuration. `UIApplication.sharedApplication` is unavailable to app extensions, so this package targets apps only.

## B506 caller-scene window attachment

`IosSceneWindow::new(main_thread, &scene, root.native_controller())` creates a retained `UIWindow` with public `UIWindow::initWithWindowScene`, attaches the caller's root controller, and calls `makeKeyAndVisible`. The host supplies a live scene and owns delegate/bootstrap and scene-event policy; this handle does not replace `WindowGroup` or install a scene delegate.

The constructor returns `framework_core::ErrorKind::InvalidInput` before creating a window when the root already has a parent controller or its already-loaded view belongs to a window. It checks `isViewLoaded` and `viewIfLoaded`; it does not load the view or reparent a controller. The main-thread-bound, `!Send`/`!Sync` handle borrows its native window with `native_window()`. On drop it hides the window and clears the root controller to release the UIKit hierarchy. Callers remain responsible for dropping it when their scene lifecycle requires cleanup.

The link probe compiles the constructor, `setRootViewController:`, `makeKeyAndVisible`, native escape, and drop cleanup for device iOS 13.0 and Simulator iOS 14.0. These checks do not prove live presentation, key-window status, scene-disconnect cleanup, or on-device lifecycle behavior.

## Rust-owned application and scene host

`IosApplicationHost::run(handler, main_thread)` enters public UIKit `UIApplicationMain` with Rust-defined `UIApplicationDelegate` and `UISceneDelegate` classes. The app delegate creates a `UISceneConfiguration` through `configurationForConnectingSceneSession:options:` and assigns the registered Rust scene delegate with `setDelegateClass:`. The consuming app's `Info.plist` must declare `UIApplicationSceneManifest`, a `UIWindowSceneSessionRoleApplication` configuration named `Rust Scene`, and `JcdIosApplicationSceneDelegate`; the Rust sample in `examples/ios-minimal` shows this setup without Swift or a Swift `@main`.

Each connected window scene owns its own `IosSceneHost`, `UIWindowScene`, `IosSceneWindow`, root `UIViewController`, root `UIView`, exact session identifier, and typed `IosScenePhase`. `scene.root_view_controller()` is a borrow for presenting native controllers; `scene.attach_view(&view)` and `detach_view` manage unparented native child views. Keep all UIKit access on the callback's main thread. The host is `!Send`/`!Sync`, and the lifecycle handler borrows it only for the duration of the callback.

The app delegate owns one application handler and a fixed 32-entry FIFO for callbacks caused by synchronous UIKit reentrancy. On overflow it drops the oldest callback; the affected scene's saturating dropped-event count increases. A delivered or dropped disconnect callback releases that scene's host deterministically. Objective-C delegate callbacks contain Rust panics before returning to UIKit; `panic=abort` still terminates the process. `run` is a one-shot process entry and does not return. No global scene registry, executor, permission, entitlement, private API, or Swift source is added.

`examples/ios-minimal/src/scene_app.rs` creates per-scene alert, share, export, and custom-render sessions from `WillConnect`; it pauses display/touch/text work on inactive/background transitions and cancels or detaches owned operations on disconnect. The current sample launched on the iOS 18.0 iPhone 16 Pro x86_64 Simulator and visibly displayed its buttons, RGBA8 view, text-input view, and a live display-link status update. This proves one scene connection and root presentation only; button actions, alert/share/export results, text entry, lifecycle transitions, multiple scenes, device runtime, and arm64 Simulator runtime were not exercised. Device/Simulator ABI checks establish imports, selectors, and deployment floors only.

The public APIs and notification constants have an iOS 13.0 floor in the installed iPhoneOS 26.5 SDK. The Rust target's effective minimum is iOS 13.0 for device and 14.0 for Simulator in the registered link probe. No permission, entitlement, or usage-description key is required.

The adapter uses generated `objc2-ui-kit 0.3.2` and `objc2-foundation 0.3.2` bindings, plus `block2 0.6.2`; it adds no Swift source and no hand-written Objective-C ABI. The Simulator smoke proves one scene connection and visible root content only; multiple simultaneous scenes, suspend/resume transitions, disconnect/reconnect, and physical-device runtime remain unverified.

## Evidence

- iOS 26.5 SDK `UIKit.framework/Headers/UIScene.h:106-111` declares all six notification names with `API_AVAILABLE(ios(13.0))`.
- iOS 26.5 SDK `UIKit.framework/Headers/UISceneSession.h:29` declares `persistentIdentifier`.
- iOS 26.5 SDK `Foundation.framework/Headers/NSString.h:131` declares `getCharacters:range:` for exact UTF-16 copying.
- Apple's [`UIScene.willConnectNotification`](https://developer.apple.com/documentation/uikit/uiscene/willconnectnotification), [`UIScene.didActivateNotification`](https://developer.apple.com/documentation/uikit/uiscene/didactivatenotification), and [`UIScene.didDisconnectNotification`](https://developer.apple.com/documentation/uikit/uiscene/diddisconnectnotification) docs specify the scene as the notification object.
- Apple's [`NotificationCenter.addObserver(forName:object:queue:using:)`](https://developer.apple.com/documentation/foundation/notificationcenter/addobserver%28forname%3Aobject%3Aqueue%3Ausing%3A%29) docs specify that the observer block runs on the selected operation queue and must be removed with its token.

Run `sh platform/ios/ios-app-scenes/check.sh` for target, Clippy, rustdoc, and link/import checks; run `cargo +1.94.1 test --locked -p ios-app-scenes` for deterministic host tests. Link/import checks are not runtime proof.

See [PLAN_IOS_SCENE_SNAPSHOT.md](../../PLAN_IOS_SCENE_SNAPSHOT.md) and [PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md](../../PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md) for B81's original snapshot scope and the additive B526 contract.

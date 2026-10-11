# iOS application lifecycle

`ios-app-lifecycle` implements the portable `framework_app::ApplicationBackend` contract with
public UIKit application state and lifecycle notifications. It does not add a new portable API or
require framework-wide initialization, a global service, executor, event loop, worker thread, or
Swift source.

## Use

Construct the backend on UIKit's main thread with the existing typed proof, then pass it to the
portable facade:

```rust
use framework_app::Application;
use ios_app_lifecycle::IosApplicationLifecycle;
use ios_runtime::main_thread::MainThread;

fn make_application(main_thread: MainThread) -> Application<IosApplicationLifecycle> {
    Application::new(IosApplicationLifecycle::new(main_thread))
}
```

Call `state()` for the latest lifecycle state known to the backend. Call `poll_event()` to remove
one already-queued event; it never waits. Construction seeds state from
`UIApplication.applicationState` and does not emit a synthetic initial event.

## Open the current application's Settings page

Call the iOS-only `open_application_settings(main_thread)` action when the app needs to direct a
user to its own Settings page:

```rust
use ios_app_lifecycle::open_application_settings;
use ios_runtime::main_thread::MainThread;

fn show_application_settings(main_thread: MainThread) {
    open_application_settings(main_thread);
}
```

The action constructs a URL from UIKit's public `UIApplicationOpenSettingsURLString` constant and
calls `UIApplication.openURL:options:completionHandler:` with empty options and no completion. It
returns immediately and does not report whether UIKit opened Settings or whether the user changed
a setting. The settings URL constant is available from iOS 8.0; this package's effective device
deployment minimum is iOS 10.0. Like the lifecycle backend, this UIApplication action is for iOS
applications and is not app-extension-safe. It does not read notification settings or request
notification permission.

## State and event mapping

UIKit's active, inactive, and background application states map to the portable `Foreground`,
`Inactive`, and `Background` states. App notifications map as follows:

| UIKit notification | State update | Portable event |
| --- | --- | --- |
| `UIApplicationDidBecomeActiveNotification` | `Foreground` | `StateChanged(Foreground)` |
| `UIApplicationWillResignActiveNotification` | `Inactive` | `StateChanged(Inactive)` |
| `UIApplicationWillEnterForegroundNotification` | `Inactive` | `StateChanged(Inactive)` |
| `UIApplicationDidEnterBackgroundNotification` | `Background` | `StateChanged(Background)` |
| `UIApplicationWillTerminateNotification` | `Terminating` | `TerminationRequested` only |

Consecutive duplicate state events are coalesced. The queue holds 16 events; if it fills, the
oldest queued event is dropped to retain the newest transition. `framework_app` has no overflow
counter, so callers cannot recover the number or identity of dropped events. The state value
remains the latest state known to the backend. Unknown future `UIApplicationState` values produce
`framework_core::ErrorKind::Platform` with the raw signed native state value as the platform code.

## Threading, ownership, and availability

Create, query, poll, and drop the backend on UIKit's main thread. The backend owns its five
Notification Center observer tokens and removes each token on drop. Native callback closures hold
only a weak reference to backend event state; they do not retain the backend or observer-token
collection. A callback that arrives after teardown cannot update or resurrect that state. Callbacks
run on the main queue, and panics are contained before a callback returns through Objective-C.

The backend is available in an iOS application after construction. It needs no user permission,
entitlement, or usage-description key. It is not app-extension-safe because
`UIApplication.sharedApplication` is unavailable to extensions. The selected UIKit declarations
have an API-availability floor of iOS 4.0. The effective device deployment minimum is iOS 10.0
with the pinned Rust 1.94.1 `aarch64-apple-ios` target; its linker warns and clamps lower deployment
settings to 10.0. The registered Simulator deployment minimum is iOS 14.0.

## Aggregate app lifecycle, not per-scene lifecycle

These are `UIApplication` aggregate state and notification values. In a scene-based app, Apple
defines `UIApplication.applicationState` from the most active scene. This backend does not identify
which scene changed and does not promise one event per scene. For a point-in-time count of
`UIScene.activationState`, use the separate [iOS scene snapshot](app-scenes.md); B81 remains
unchanged. B454's `ios-app-scenes::IosSceneLifecycleEvents` separately receives per-scene UIKit
notifications without replacing the app's scene delegate.

The backend observes only after construction, so it does not replay earlier launch or lifecycle
events and does not emit `ApplicationState::Launching`. iOS may suspend or terminate an app without
delivering `UIApplicationWillTerminateNotification`; a termination request is best-effort and does
not promise cleanup time. The backend emits neither `LifecycleEvent::Terminated` nor
`ApplicationState::Terminated`, because the process cannot observe its own completed termination.

## Validation and evidence limits

Run `sh platform/ios/ios-app-lifecycle/check.sh`, then the registered
`ios-app-lifecycle` profile. The profile's link/import fixture links a small Rust consumer for
device and Simulator, checks public framework and Objective-C imports, rejects Swift runtime
imports, and verifies deployment metadata. Host regression tests use synthetic native state and
notification values. Device and Simulator target checks establish compilation; link/import checks
establish linkage only. These checks do not launch an app, deliver a UIKit notification, exercise
scene transitions, show runtime availability, or prove termination behavior.

The iOS 4.0 API-availability floor is supported by the installed Xcode 26.6 build 17F113 /
iPhoneOS SDK 26.5
`UIKit.framework/Headers/UIApplication.h`: `UIApplicationState` at lines 62–66,
`applicationState` at line 137, and the two iOS 4.0 foreground/background declarations at lines
551–552. Separately, `check-link-imports.sh` verifies the effective iOS 10.0 device minimum emitted
by Rust 1.94.1 and the iOS 14.0 Simulator minimum. Apple's public references:
[`UIApplication.applicationState`](https://developer.apple.com/documentation/uikit/uiapplication/applicationstate),
[`UIApplicationState`](https://developer.apple.com/documentation/uikit/uiapplication/state),
[`UIApplication.openSettingsURLString`](https://developer.apple.com/documentation/uikit/uiapplication/opensettingsurlstring),
[`UIApplication.open(_:options:completionHandler:)`](https://developer.apple.com/documentation/uikit/uiapplication/open%28_%3Aoptions%3Acompletionhandler%3A%29),
[`UIApplicationDidBecomeActiveNotification`](https://developer.apple.com/documentation/uikit/uiapplication/didbecomeactivenotification),
[`UIApplicationWillEnterForegroundNotification`](https://developer.apple.com/documentation/uikit/uiapplication/willenterforegroundnotification),
and [`UIApplicationWillTerminateNotification`](https://developer.apple.com/documentation/uikit/uiapplication/willterminatenotification).

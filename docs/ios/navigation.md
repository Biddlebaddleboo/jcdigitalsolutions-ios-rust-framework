# iOS native screen navigation

`ios-navigation` wraps UIKit's `UIViewController` and `UINavigationController` for a small Rust
screen-route stack. It composes with `ios-ui`: each `IosScreenController` takes a completed `IosView`
and an optional navigation title, and UIKit retains the view as that controller's root view

```rust
use framework_core::{Error, ErrorKind, Result};
use ios_navigation::{IosNavigationController, IosScreenController};
use ios_runtime::main_thread::MainThread;
use ios_ui::IosView;

fn main_thread() -> Result<MainThread> {
    MainThread::current().ok_or_else(|| Error::new(ErrorKind::Unavailable))
}

fn make_navigation(root_view: IosView, settings_view: IosView) -> Result<IosNavigationController> {
    let root = IosScreenController::new(main_thread()?, root_view, Some("Dashboard"));
    let mut navigation = IosNavigationController::new(main_thread()?, root);
    let settings = IosScreenController::new(main_thread()?, settings_view, Some("Settings"));
    navigation.push(&settings, true)?;
    drop(settings);
    Ok(navigation)
}

fn go_back(navigation: &mut IosNavigationController) -> bool {
    navigation.pop(true)
}
```

## Routes and titles

`IosNavigationController::new` installs one root screen. `push` asks UIKit to add an unparented
screen; it returns `framework_core::ErrorKind::InvalidInput` for a controller that already has a
parent. `pop` asks UIKit to remove the top screen and returns whether UIKit returned a controller
to remove. UIKit owns the stack, animation, navigation bar and native back-button behavior. No Rust
route registry or duplicate stack state is kept

Titles are optional UTF-8 values on `IosScreenController::new` and `set_title`. `None` clears a
title, while `Some("")` sets an empty title. Applications supply their own screen titles without
adding app-specific route types to the framework

## Navigation-bar action

`IosNavigationBarAction` creates a public UIKit `UIAction` and a `UIBarButtonItem` primary action.
Its required nonblank title is the accessible name; an optional SF-symbol name supplies the native
image. Creation checks for iOS 14.0 at runtime and returns `ErrorKind::Unavailable` on older
systems, while the package keeps its iOS 12 deployment floor. An unknown or unavailable system
symbol also returns `Unavailable`; empty and whitespace-only titles or symbol names return
`InvalidInput`

```rust
use ios_navigation::IosNavigationBarAction;

let action = IosNavigationBarAction::new(main_thread()?, "Settings", Some("gearshape"), || {
    // Application-owned settings behavior
})?;
screen.set_right_bar_action(Some(&action));
drop(action); // UIKit retains the item and callback
screen.set_right_bar_action(None); // Clear the right-side item
```

`set_right_bar_action(Some(...))` installs or replaces the right-side item; `None` clears it.
UIKit retains the bar item and the item copies its primary action, so dropping the Rust action value
does not clear the installed callback. The callback is `FnMut()`, runs synchronously when UIKit
invokes the action, contains Rust panics, and ignores reentrant invocation. A symbol lookup failure
is reported rather than silently showing a different image

This is one imperative UIKit navigation-bar control. It does not add route links, declarative
composition, state observation, bindings, or SwiftUI-style reconciliation. The application still
owns the callback's behavior, route changes, screen composition and state

## Thread and ownership

Create screens and navigation controllers with `ios_runtime::main_thread::MainThread`. Their
wrappers are `!Send` and `!Sync`. `IosScreenController::new` consumes its `IosView`; after
`UIViewController::setView`, UIKit retains the native view, so the wrapper does not retain a second
view handle. UIKit retains the root and pushed controllers in its navigation stack. Keep the
`IosNavigationController` alive while the app needs its Rust `push` and `pop` methods

`native_controller()` returns a borrow of the underlying `UINavigationController` for app-owned
scene or window setup. The framework does not create or own a `UIWindow`, `UIWindowScene`, app
delegate, or scene lifecycle. The app must retain its navigation wrapper and attach the borrowed
controller through its own UIKit lifecycle

## Scope and evidence

The host tests cover only the pure push preflight: an unparented controller reaches the push
operation once, while an already-parented controller is rejected before it is called. Navigation
action host tests cover input validation, panic containment, callback reentry and the Rust-side
handler-capture lifetime; they do not execute UIKit or prove its runtime retention behavior. Target
compilation and link/import checks can establish that the native types and selectors build and link;
they do not prove window attachment, visible action presentation, accessibility output, callback
delivery, a visible transition, back-button behavior, or simulator/device runtime. Those require an
app run on the target

# PLAN_IOS_SCENE_SNAPSHOT.md — B81 UIKit scene activity snapshot

## Status

B81 adds an iOS 13.0 point-in-time scene activity snapshot in `ios-app-scenes`. Device and Simulator compile and strict Clippy gates, rustdoc, and both link/import inspections pass on Xcode 26.6 / SDK 26.5 with Rust 1.94.1. The arm64 device probe minos is 13.0; the arm64 Simulator probe minos is 14.0. Both probes were inspected and not executed. Row `001-core-app-ui-application-lifecycle` remains partial because B81 does not deliver scene lifecycle events

## Objective

Add one iOS-only caller-invoked snapshot of the activation states in the app's current `UIApplication.connectedScenes` set

## Scope

- `platform/ios/ios-app-scenes/**`
- `docs/ios/app-scenes.md`
- This focused plan

The snapshot is a B81 additive partial surface for canonical row `001-core-app-ui-application-lifecycle`. It does not replace the app lifecycle contract or make that row complete. Capability row counts do not change

## Contract

- Require `ios_runtime::main_thread::MainThread`
- Read `UIApplication.connectedScenes` once and each returned `UIScene.activationState` once in one synchronous main-thread pass; make no atomic cross-scene guarantee
- Return counts for unattached, foreground-active, foreground-inactive, background, and unknown values
- Return owned scalar counts only; do not expose scene IDs, names, native handles, or callback state
- Treat an empty scene set as a valid empty snapshot
- Make no claim that a scene is onscreen, visible, foregrounded as a whole app, or still in that state after return
- Do not subscribe to notifications or delegates, create windows, present UI, or alter scene state

## API and host limits

- `UIApplication.connectedScenes` and `UIScene` first appear in iOS 13.0 in the installed iOS 26.5 SDK headers
- `UIScene.activationState` is part of the same iOS 13.0 `UIScene` API
- `UIApplication` and `UIScene` are UIKit main-thread APIs; the Rust entry point consumes the `ios-runtime` main-thread proof
- No permission, entitlement, usage string, or host scene manifest is needed to query the property
- Scene lifecycle callbacks need a host `UISceneDelegate` class selected in the app's `Info.plist` scene configuration or app-delegate configuration callback; this package does not implement that host work
- `UIApplication.sharedApplication` is unavailable to app extensions; this package targets apps only
- No runtime UI, user action, lifecycle event, or device-state evidence is claimed

## Generated bindings

Use `objc2-ui-kit 0.3.2` with `std`, `UIApplication`, `UIScene`, `UISceneDefinitions`, and `UIResponder`. Use `objc2-foundation 0.3.2` with `NSArray` and `NSEnumerator` to copy the set to a retained array and use its safe iterator. No hand-written Objective-C ABI or unsafe Rust is needed

## Validation

Root's `Cargo.lock` now contains the package record. `sh platform/ios/ios-app-scenes/check.sh` checks device and Simulator targets, strict Clippy, rustdoc, then builds and inspects a link/import probe without execution. Local probe minos is iOS 13.0 for device and 14.0 for Simulator; the Simulator minos is the Rust target default here and does not raise the public API floor. The gate adds or runs no tests, consumer binaries, or UI actions

## Evidence

- Installed headers: `$(xcrun --sdk iphoneos --show-sdk-path)/System/Library/Frameworks/UIKit.framework/Headers/UIApplication.h:170`, `connectedScenes` is marked `API_AVAILABLE(ios(13.0))`
- Installed headers: `$(xcrun --sdk iphoneos --show-sdk-path)/System/Library/Frameworks/UIKit.framework/Headers/UIScene.h:19,38`, `UIScene` and `activationState` are marked `API_AVAILABLE(ios(13.0))` and `NS_SWIFT_UI_ACTOR`
- Apple docs: [UIApplication.connectedScenes](https://developer.apple.com/documentation/uikit/uiapplication/connectedscenes)
- Apple docs: [UIScene.activationState](https://developer.apple.com/documentation/uikit/uiscene/activationstate-swift.property)
- Apple docs: [UISceneDelegate](https://developer.apple.com/documentation/uikit/uiscenedelegate)
- Apple docs: [UISceneConfiguration](https://developer.apple.com/documentation/uikit/uisceneconfiguration)

Apple defines a connected scene as in memory and potentially doing active work; it may be foreground or background and onscreen or offscreen. The result therefore reports only state values from a synchronous read pass and makes no cross-scene atomicity claim

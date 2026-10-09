# B81 iOS app scene snapshot

`ios-app-scenes` reads a point-in-time count of scenes returned by `UIApplication.connectedScenes`, grouped by `UIScene.activationState`

```rust,ignore
let main_thread = ios_runtime::main_thread::MainThread::current().ok_or_else(|| {
    framework_core::Error::new(framework_core::ErrorKind::Unavailable)
})?;
let state = ios_app_scenes::snapshot(main_thread);
let active_scenes = state.foreground_active_count();
```

The API needs the main thread and has an iOS 13.0 floor. The result holds only scalar counts from one synchronous pass, with no atomic cross-scene or after-return guarantee. It does not reveal scene identifiers or names, and it does not claim that a scene is visible or onscreen. `connectedScenes` may include a scene in the foreground or background, on or offscreen

The query does not need a permission, entitlement, usage string, or scene manifest. To receive scene lifecycle callbacks, the app host must select a `UISceneDelegate` class through its scene configuration; this crate does not install a delegate or poll for events. `UIApplication.sharedApplication` is unavailable to app extensions

This API does not create a scene or window, present UI, control a scene, expose application lifecycle events, or guarantee the state after the call returns

The Rust adapter uses `objc2-ui-kit 0.3.2` generated bindings and `objc2-foundation 0.3.2` safe array iteration. It contains no hand-written Objective-C ABI and no unsafe Rust

See [PLAN_IOS_SCENE_SNAPSHOT.md](../../PLAN_IOS_SCENE_SNAPSHOT.md) and [PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md](../../PLAN_VALIDATION_IOS_SCENE_SNAPSHOT.md) for source evidence and the scoped gate

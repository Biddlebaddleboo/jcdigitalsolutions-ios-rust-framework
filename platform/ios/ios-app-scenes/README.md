# B81 iOS app scene snapshot

`ios-app-scenes` reads scene activation-state counts from the caller app's `UIApplication.connectedScenes` in one synchronous main-thread pass. It makes no atomic cross-scene or after-return guarantee

- The API needs a caller-owned `ios_runtime::main_thread::MainThread` proof
- The public API floor is iOS 13.0 from both scene symbols in the iOS 26.5 SDK headers
- The result owns only integer counts and exposes no scene object, identifier, name, or callback
- A connected scene can be foreground or background and on or offscreen, so this API makes no visibility claim
- Zero scene values is a valid snapshot and does not mean that the app process is absent or terminated
- No permission, entitlement, usage string, scene manifest, or scene delegate is needed for this query
- An app that needs scene lifecycle callbacks must configure a `UISceneDelegate` through its app host; this crate does not register a delegate or receive events
- `UIApplication.sharedApplication` is unavailable to app extensions, so this package is for app targets only
- No scene control, window creation, presentation, process lifecycle, background-work, or runtime guarantee is part of this API

The narrow generated binding set is `objc2-ui-kit 0.3.2` with `std`, `UIApplication`, `UIScene`, `UISceneDefinitions`, and `UIResponder`, plus `objc2-foundation 0.3.2` with `NSArray` and `NSEnumerator` for safe set-to-array iteration

Run `sh platform/ios/ios-app-scenes/check.sh` after the root lock includes this workspace package

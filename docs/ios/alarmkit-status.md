# AlarmKit authorization state for iOS

`ios-alarmkit-status` exposes one read-only operation: the current `AlarmManager.authorizationState` snapshot

```rust
let state = ios_alarmkit_status::authorization_state()?;
```

The result is `NotDetermined`, `Denied`, `Authorized`, or `Unknown`. `Unknown` represents a future public enum case not mapped by this crate version

The crate requires an iOS 26.0+ deployment target and strongly links AlarmKit. It reads only the current authorization state. It does not call `requestAuthorization()`, display a prompt, schedule or change alarms, inspect alarm content, or observe authorization updates. The value does not guarantee a later alarm operation will succeed

The Swift property returns a resilient enum. The C bridge uses the compiler-derived `swiftcall` getter and the enum's runtime value witnesses for size, alignment, case tag, and destruction; it does not read or copy a guessed enum layout. The owned `AlarmManager.shared` reference uses the existing Swift retain/release wrapper

The ABI evidence uses Xcode 26.6 / iOS SDK 26.5, below the repository's Xcode 27.x baseline. Rust checks and static compiler evidence do not establish app-link, device/Simulator runtime, or Apple behavior parity. See [B280](../../PLAN_CAPABILITIES_ALARMKIT.md), Apple's [AlarmManager](https://developer.apple.com/documentation/alarmkit/alarmmanager), [AuthorizationState](https://developer.apple.com/documentation/alarmkit/alarmmanager/authorizationstate-swift.enum), and [`authorizationState`](https://developer.apple.com/documentation/alarmkit/alarmmanager/authorizationstate-swift.property)

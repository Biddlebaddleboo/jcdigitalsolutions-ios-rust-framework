# iOS ARKit world-tracking support snapshot

`ios-maps::world_tracking_support()` queries the inherited
`ARConfiguration.isSupported` class property with `ARWorldTrackingConfiguration` as the concrete
class receiver, then stores the returned boolean in `framework-maps::WorldTrackingSupport`.

```rust
match ios_maps::world_tracking_support() {
    Some(support) if support.is_supported() => { /* system reports world-tracking support */ }
    Some(_) => { /* system reports no support for this configuration */ }
    None => { /* non-iOS or runtime before iOS 11.0 */ }
}
```

Apple marks `ARConfiguration` and `ARWorldTrackingConfiguration` available from iOS 11.0. The
backend checks that runtime floor before the class query. The call does not create an `ARSession`,
start camera capture, read frames, request permission, or report authorization state. Apple says to
check `isSupported` before creating and running a configuration; the camera prompt is associated
with starting an AR session or otherwise using the camera. A later AR session still requires camera
consent and the app's `NSCameraUsageDescription`.

A `true` value is only the system's configuration-support result. It is not a guarantee of user
authorization, successful tracking, environmental feature quality, or an active session. Simulator
and compile/link checks do not establish hardware support or runtime tracking behavior. No ARKit
native handle or Swift ABI is exposed.

The backend uses `objc2-ar-kit` 0.3.2 with default features disabled and only `ARConfiguration`
and `objc2` enabled. The package import gate expects `ARKit`, `Foundation`, `libSystem.B.dylib`,
and `libobjc.A.dylib`, as observed in the linked probe; it rejects session, frame, camera, UIKit,
and Swift runtime surfaces. The build-only link/import gate is
`sh platform/ios/ios-maps/check-link-imports.sh`; it must not execute its iOS artifacts.

Apple references: [ARConfiguration.isSupported](https://developer.apple.com/documentation/arkit/arconfiguration/issupported), [ARWorldTrackingConfiguration](https://developer.apple.com/documentation/arkit/arworldtrackingconfiguration), and [Verifying Device Support and User Permission](https://developer.apple.com/documentation/arkit/verifying-device-support-and-user-permission).

# iOS Nearby Interaction capability query

`ios-nearby` reads only the public class property `NISession.deviceCapabilities` and its `NIDeviceCapability.supportsPreciseDistanceMeasurement` Boolean getter. Apple documents the field as device feature support and makes these APIs available from iOS 16. Apple's deprecated `NISession.isSupported` property is equivalent to this field; this adapter uses the current iOS 16 API instead.

The host app must set its iOS deployment target to 16.0 or later before it calls `IosNearbyInteractionBackend::snapshot`. The Rust crate cannot enforce the host app's deployment target. The backend module excludes Mac Catalyst. Device and simulator target checks establish compile and lint support only.

This query does not create or run a session, ask for permission, exchange discovery tokens, discover peers, or start ranging. Apple documents the permission check and `NSNearbyInteractionUsageDescription` prompt before a session starts. This query does not start a session, so it does not request that permission or require that key. This slice does not configure or validate entitlements, Xcode capabilities, background modes, or other host project settings. Apps that later add background ranging need separate review of Apple's Nearby Interaction background-mode requirements.

The Boolean is a single device-feature report, not a guarantee that Nearby Interaction can operate in the current app context. It does not establish permission, peer compatibility, a valid configuration, session readiness, operation success, measurement quality, or background support. No device/simulator runtime, prompt, session, peer, or ranging test is claimed.

## Binding and safety audit

The backend pins `objc2-nearby-interaction` 0.3.2 with default features disabled and only `NISession` and `NIDeviceCapability` enabled. Generated `NISession::deviceCapabilities()` returns an owned `Retained<ProtocolObject<dyn NIDeviceCapability>>`; the generated protocol getter returns a Rust `bool` for the public Objective-C `BOOL` property. The binding module declares `#[link(name = "NearbyInteraction", kind = "framework")]`. No raw Objective-C message, hand-written ABI, extra Apple framework feature, callback, session instance, token, or Swift source is used.

Both generated calls are `unsafe` in the binding. The private wrapper documents the iOS 16 API-floor invariant, the owned return value, and the scalar getter's lack of pointer or callback inputs. The wrapper does not expose the native protocol object.

## Apple references

- [NISession.deviceCapabilities](https://developer.apple.com/documentation/nearbyinteraction/nisession/devicecapabilities)
- [NIDeviceCapability](https://developer.apple.com/documentation/nearbyinteraction/nidevicecapability)
- [NISession.isSupported and its replacement](https://developer.apple.com/documentation/nearbyinteraction/nisession/issupported)
- [Initiating and maintaining a session](https://developer.apple.com/documentation/nearbyinteraction/initiating-and-maintaining-a-session)
- [NSNearbyInteractionUsageDescription](https://developer.apple.com/documentation/BundleResources/Information-Property-List/NSNearbyInteractionUsageDescription)

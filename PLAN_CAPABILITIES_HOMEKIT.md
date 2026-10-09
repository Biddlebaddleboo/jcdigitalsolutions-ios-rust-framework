# PLAN_CAPABILITIES_HOMEKIT.md — B173: HomeKit authorization no-go

## Audit scope

Recheck row `072-personal-data-system-stores-homekit` for a public iOS 26.5 API that can return a
meaningful HomeKit authorization or capability snapshot without first-manager prompting, HomeKit
lifecycle, or user UI. This focused audit does not change the aggregate capability manifest.

## Result

No qualifying Rust-callable snapshot exists. Keep row 072 unsupported (`X`).

`HMHomeManager.authorizationStatus` is the public authorization value, but it is an instance
property, not a static query. Apple states that first use of HomeKit—typically creation of an
`HMHomeManager`—automatically prompts the user for access. Calling the property therefore requires
the exact manager initialization that the no-prompt contract excludes. The getter reports access to
home data only; it does not establish HomeKit accessory presence, host setup, or general HomeKit
capability.

## SDK and binding evidence

Inspected Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5, and Rust/Cargo 1.94.1.

- The installed public header is
  `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMHomeManager.h`.
  It declares `HMHomeManager.authorizationStatus` as an instance `readonly`
  `HMHomeManagerAuthorizationStatus` property available from iOS 13.0. `HMHomeManager.init` is an
  instance initializer.
- Apple defines the status bits as `Determined`, `Restricted`, and `Authorized`. The Objective-C
  header says the system manages authorization and no explicit request is needed; it does not make
  manager creation non-prompting.
- Apple's current authorization documentation states that the first use of HomeKit, typically
  creating an `HMHomeManager`, automatically prompts for home-data access. If the app lacks
  `NSHomeKitUsageDescription`, first HomeKit use crashes.
- `objc2-home-kit` 0.3.2 is present in the local Cargo registry cache but is not a workspace
  dependency or in the root `Cargo.lock`. Its generated `HMHomeManager` binding exposes typed
  `init` and `authorizationStatus` methods; the status getter returns the typed
  `HMHomeManagerAuthorizationStatus` option set. Using that binding does not avoid the native
  manager's prompt behavior.
- The other public `isSupported` result found in the installed headers is
  `HMEvent.isSupportedForHome:`. It requires an existing `HMHome` instance and reports support for
  that event type on that home, not a global authorization/capability value. Acquiring the home
  requires HomeKit's manager-backed data path.

No new dependency, source code, runtime call, permission request, or device query was made.

## Primary sources

- [HomeKit `HMHomeManager.authorizationStatus`](https://developer.apple.com/documentation/homekit/hmhomemanager/authorizationstatus)
- [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app)
- [Configuring HomeKit access](https://developer.apple.com/documentation/xcode/configuring-homekit-access)
- [`objc2-home-kit` 0.3.2](https://docs.rs/objc2-home-kit/0.3.2/objc2_home_kit/struct.HMHomeManager.html)

## B322 follow-up: `HMAccessory.isReachable` has no safe standalone Rust contract

Audited the iOS 26.5 `HMAccessory.isReachable` Boolean as a distinct read-only HomeKit value. The iOS 8.0 `HMAccessory` header defines it as whether that accessory is currently reachable; Apple's docs sharpen this to whether it can be communicated with in the current network environment. This is a per-accessory, momentary network state, not HomeKit support, authorization, accessory discovery, or a guarantee that a later command succeeds.

An `HMAccessory` is not constructed by the caller. Apple says an app obtains it from an `HMHome` or `HMRoom` accessory list, which in turn requires the HomeKit data path. Apple's HomeKit setup docs state that the first use of HomeKit, typically `HMHomeManager` creation, prompts for permission and requires the HomeKit entitlement and `NSHomeKitUsageDescription`. The accessory delegate has a reachability-change callback, but the Apple pages reviewed do not specify its delivery queue or a synchronization contract for concurrent reads.

The public header declares `reachable` as `nonatomic`, `readonly`, and `getter=isReachable`. The local `objc2-home-kit` 0.3.2 generated binding exposes a typed `unsafe fn isReachable(&self) -> bool`; its safety docs repeat that the property is non-atomic and might not be thread-safe. The crate is cached locally but is not a workspace dependency or in `Cargo.lock`. A safe Rust facade cannot promise a race-free read without a documented serialized executor or another host-owned synchronization contract. Requiring only `MainThread` would not resolve the absent HomeKit queue guarantee.

Decision: no B322 Rust facade or dependency change; keep row `072-personal-data-system-stores-homekit` at `X`. Revisit only inside a selected HomeKit host that owns the accessory lifetime and a documented serialized callback/queue contract. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.isReachable`](https://developer.apple.com/documentation/homekit/hmaccessory/isreachable), [`HMAccessoryDelegate.accessoryDidUpdateReachability(_:)`](https://developer.apple.com/documentation/homekit/hmaccessorydelegate/accessorydidupdatereachability%28_%3A%29), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, build, link probe, test, app launch, Simulator run, or device query was performed for B322

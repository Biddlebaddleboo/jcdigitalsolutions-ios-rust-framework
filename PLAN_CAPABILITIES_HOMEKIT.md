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

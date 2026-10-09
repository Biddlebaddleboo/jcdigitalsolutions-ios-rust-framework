# PLAN_CAPABILITIES_HOMEKIT.md — B173: HomeKit authorization no-go

## Audit scope

Recheck row `072-personal-data-system-stores-homekit` for a public iOS 26.5 API that can return a
meaningful HomeKit authorization or capability snapshot without first-manager prompting, HomeKit
lifecycle, or user UI. This focused audit does not change the aggregate capability manifest.

## Result

No qualifying prompt-free HomeKit authorization or global capability snapshot exists. B173 remains
the no-go for `HMHomeManager.authorizationStatus` because the getter requires a manager whose first
creation may prompt. B339 later adds only a host-supplied per-accessory
`HMAccessory.supportsIdentify` snapshot, so row 072 is partial (`B`), not general HomeKit support.

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

Decision: no B322 Rust facade or dependency change. The standalone `isReachable` surface remains out of scope; B339 later makes row `072-personal-data-system-stores-homekit` partial (`B`) only for `supportsIdentify`. Revisit `isReachable` only inside a selected HomeKit host that owns the accessory lifetime and a documented serialized callback/queue contract. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.isReachable`](https://developer.apple.com/documentation/homekit/hmaccessory/isreachable), [`HMAccessoryDelegate.accessoryDidUpdateReachability(_:)`](https://developer.apple.com/documentation/homekit/hmaccessorydelegate/accessorydidupdatereachability%28_%3A%29), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, build, link probe, test, app launch, Simulator run, or device query was performed for B322

## B373 audit — `HMAccessory.isBlocked` has no safe standalone Rust contract

B373 audited `HMAccessory.isBlocked` as a distinct, useful per-accessory Boolean. The iOS 26.5 public `HMAccessory.h` declares `@property (nonatomic, readonly, getter=isBlocked) BOOL blocked` on `HMAccessory`, whose class is available from iOS 8.0. Apple's documentation defines the value only as whether that accessory is blocked. It does not report HomeKit support, authorization, reachability, or whether another operation will succeed.

The generated local `objc2-home-kit` 0.3.2 binding exposes the exact typed getter as `pub unsafe fn isBlocked(&self) -> bool`; its generated safety documentation repeats that the property is not atomic and might not be thread-safe. The public HomeKit sources reviewed do not supply a serialized executor or synchronization guarantee for reads of this mutable property. A main-thread-only wrapper would not establish that HomeKit updates are serialized on the main thread. `HMAccessory` is supplied from an `HMHome` or `HMRoom` accessory list rather than directly created; obtaining it remains under the existing host HomeKit lifecycle, authorization, `com.apple.developer.homekit` entitlement, and `NSHomeKitUsageDescription` requirements.

Decision: no B373 Rust facade or dependency change. Do not expose `isBlocked` as a safe scalar snapshot without a documented host-compatible synchronization contract. This does not alter row `072` beyond B339's `supportsIdentify` partial. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.isBlocked`](https://developer.apple.com/documentation/homekit/hmaccessory/isblocked), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, build, link probe, test, app launch, Simulator run, or device query was performed for B373

## B384 audit — `HMAccessory.isBridged` has no safe standalone Rust contract

B384 audited `HMAccessory.isBridged` as a distinct, per-accessory topology Boolean. The iOS 26.5 public `HMAccessory.h` declares `@property (nonatomic, readonly, getter=isBridged) BOOL bridged` on `HMAccessory`, whose class is available from iOS 8.0. Apple documents `true` for an accessory accessed through a bridge; the bridge itself and standalone accessories report `false`. This describes how a host-provided accessory is connected to HomeKit, not general HomeKit support, authorization, reachability, or operation success.

The local generated `objc2-home-kit` 0.3.2 binding exposes the typed getter as `pub unsafe fn isBridged(&self) -> bool`; its generated safety documentation repeats that the property is not atomic and might not be thread-safe. The reviewed public HomeKit docs do not define an executor or synchronization contract for concurrent reads and updates. A `MainThread` check alone would not establish that HomeKit state changes are serialized on that thread. An `HMAccessory` must still come from the host's `HMHome` or `HMRoom` data path and requires the host-owned HomeKit capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior authorization, and object lifetime described for B339.

Decision: no B384 Rust facade or dependency change. The no-go is specific to a safe scalar getter absent a documented host-compatible synchronization contract; it does not alter row `072` beyond B339's `supportsIdentify` partial. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.isBridged`](https://developer.apple.com/documentation/homekit/hmaccessory/isbridged), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, build, link probe, test, app launch, Simulator run, or device query was performed for B384

## B385 audit — bridged accessory identifiers have no safe standalone Rust contract

B385 audited `HMAccessory.uniqueIdentifiersForBridgedAccessories` as a distinct nullable identifier-list property. The iOS 26.5 public header declares `NSArray<NSUUID *> *` as `nonatomic`, `readonly`, and `copy`, available from iOS 9.0. Apple's documentation and the header specify `nil` for a standalone accessory or an accessory behind a bridge, and a non-empty UUID array when the receiver is itself a bridge; the UUIDs identify accessories vended by that bridge. This is topology metadata for a host-supplied accessory, not general HomeKit support, authorization, or operational readiness.

The generated local `objc2-home-kit` 0.3.2 binding returns `Option<Retained<NSArray<NSUUID>>>` from a typed `unsafe fn uniqueIdentifiersForBridgedAccessories(&self)`. Its generated safety documentation states that the property is non-atomic and might not be thread-safe. The reviewed HomeKit sources provide no executor or synchronization guarantee for this read. Copying a Rust-owned identifier list would therefore require both an undocumented serialization contract and array/UUID conversion; the values also disclose private home topology IDs without a selected Rust caller use case. The host still owns HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and the accessory source/lifetime from B339.

Decision: no B385 Rust facade or dependency change. Keep row `072-personal-data-system-stores-homekit` partial (`B`) only for B339's `supportsIdentify`; this identifier property adds no general capability claim. Revisit only inside a selected HomeKit host that owns the UUID data purpose and documents serialized access. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.uniqueIdentifiersForBridgedAccessories`](https://developer.apple.com/documentation/homekit/hmaccessory/uniqueidentifiersforbridgedaccessories), [`HMAccessory.isBridged`](https://developer.apple.com/documentation/homekit/hmaccessory/isbridged), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, UUID enumeration, build, link probe, test, app launch, Simulator run, or device query was performed for B385

## B386 audit — `HMAccessory.bridgedAccessories` has no typed binding

B386 audited `HMAccessory.bridgedAccessories` as a distinct bridge-topology collection. The iOS 26.5 public header declares `NSArray<HMAccessory *> *` as `nonatomic`, `readonly`, and `copy`, available from iOS 13.0. Apple documents that a bridge receiver returns the accessories behind that bridge, while a non-bridge receiver returns an empty array. This is host-provided accessory topology, not general HomeKit support, authorization, reachability, or operation success.

The installed `objc2-home-kit` 0.3.2 generated `HMAccessory` binding contains no `bridgedAccessories` accessor, even though the public Objective-C header declares it. The package's `HMAccessory` feature does expose other generated getters, but this property is absent from the binding source. Accessing it by an untyped selector is outside this workstream; a separate approved typed binding or compiler-derived bridge would be required. The header's nonatomic semantics and the retrieved accessory objects' ownership/lifetime further preclude an inferred safe Rust array snapshot.

Decision: no B386 Rust facade or dependency change. Keep row `072-personal-data-system-stores-homekit` partial (`B`) only for B339's `supportsIdentify`; the missing typed method does not justify a raw selector or bridge-object enumeration. Revisit only for a selected host and supported binding/ownership route. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.bridgedAccessories`](https://developer.apple.com/documentation/homekit/hmaccessory/bridgedaccessories), [`HMAccessory.isBridged`](https://developer.apple.com/documentation/homekit/hmaccessory/isbridged), and [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory).

No dependency, source, manager creation, permission request, accessory read, bridge-object enumeration, raw selector, build, link probe, test, app launch, Simulator run, or device query was performed for B386

## B388 audit — `HMAccessory.matterNodeID` is identity metadata, not capability

B388 audited the distinct iOS 16.1+ `HMAccessory.matterNodeID` property. The iOS 26.5 public header declares nullable `NSNumber *` as `nonatomic`, `readonly`, and `copy`, marked `NS_REFINED_FOR_SWIFT`; its comment defines the value as the node identifier used to identify the device on Apple's Matter fabric. Apple's Swift documentation exposes it as `UInt64?`. This is per-accessory fabric identity metadata, not HomeKit or Matter support, authorization, reachability, or operation readiness.

The generated local `objc2-home-kit` 0.3.2 binding provides a typed `unsafe fn matterNodeID(&self) -> Option<Retained<NSNumber>>`, and its safety documentation repeats that the property is non-atomic and might not be thread-safe. No HomeKit executor or synchronization guarantee for concurrent reads is documented in the reviewed sources. Converting and returning the value would disclose a fabric node identifier, with no selected Rust caller purpose; a main-thread-only wrapper would not cure the missing concurrency guarantee. The host still owns the HomeKit authorization, entitlement, usage description, and supplied accessory lifetime from B339.

Decision: no B388 Rust facade or dependency change. Keep row `072-personal-data-system-stores-homekit` partial (`B`) only for B339's `supportsIdentify`. Revisit only for a selected HomeKit/Matter host with a justified identity-data purpose and documented serialized access. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.matterNodeID`](https://developer.apple.com/documentation/homekit/hmaccessory/4098190-matternodeid), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, node-ID read, build, link probe, test, app launch, Simulator run, or device query was performed for B388

## B389 audit — `HMAccessory.category` has no safe standalone Rust contract

B389 audited `HMAccessory.category` as a distinct accessory-classification value. The iOS 26.5 public header declares `@property (nonatomic, readonly, strong) HMAccessoryCategory *category`, available from iOS 9.0. Apple's documentation says the category indicates what kind of accessory the host received, such as a light bulb, garage door opener, or faucet; `HMAccessoryCategory.categoryType` is an extensible string identifier, not a closed Rust enum or a support/authorization result.

The local `objc2-home-kit` 0.3.2 generated binding includes a typed `category(&self) -> Retained<HMAccessoryCategory>` under its `HMAccessoryCategory` feature, but marks the getter `unsafe` and repeats that the property is nonatomic and might not be thread-safe. `HMAccessoryCategory` itself is marked `NS_SWIFT_SENDABLE` and generated `Send`/`Sync`, which does not make the nonatomic getter on the live `HMAccessory` safe to call concurrently. A caller would need the host's documented serialized access/lifetime contract and would have to preserve unknown category-type strings rather than assume a fixed enum.

Decision: no B389 Rust facade or dependency change. Category metadata can support host presentation/classification but does not add HomeKit capability coverage; keep row `072-personal-data-system-stores-homekit` partial (`B`) only for B339's `supportsIdentify`. Revisit only for a selected host with a synchronization contract and a Rust-owned open-string API. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h` and `HMAccessoryCategory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs` and `HMAccessoryCategory.rs`; Apple [`HMAccessory.category`](https://developer.apple.com/documentation/homekit/hmaccessory/category), [`HMAccessoryCategory.categoryType`](https://developer.apple.com/documentation/homekit/hmaccessorycategory/categorytype), [Accessory Category Types](https://developer.apple.com/documentation/homekit/accessory-category-types), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory/category read, build, link probe, test, app launch, Simulator run, or device query was performed for B389

## B390 audit — `HMAccessory.uniqueIdentifier` has no safe standalone Rust contract

B390 audited `HMAccessory.uniqueIdentifier` as a distinct per-accessory identity value. The iOS 26.5 public header declares `NSUUID *` as `nonatomic`, `readonly`, and `copy`, available from iOS 9.0. Apple's documentation calls it a unique identifier for the accessory but does not define it as a general HomeKit capability or readiness value. It is host-sensitive home identity data.

The generated local `objc2-home-kit` 0.3.2 binding exposes a typed `unsafe fn uniqueIdentifier(&self) -> Retained<NSUUID>`; its generated safety documentation states that the property is non-atomic and might not be thread-safe. The `copy` ownership rule produces a retained Objective-C object but does not provide an atomic read or a documented HomeKit executor. A safe Rust wrapper would still need a host-owned serialized access contract and a concrete identifier-use purpose; the current row has neither. The host must also already own the HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and accessory lifetime from B339.

Decision: no B390 Rust facade or dependency change. Keep row `072-personal-data-system-stores-homekit` partial (`B`) only for B339's `supportsIdentify`; the per-accessory UUID is not a support snapshot. Revisit only for a selected host with a justified identity-data purpose and documented serialized reads. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.uniqueIdentifier`](https://developer.apple.com/documentation/homekit/hmaccessory/uniqueidentifier?language=objc), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory read, UUID read, build, link probe, test, app launch, Simulator run, or device query was performed for B390

## B391 audit — GO candidate for a host-supplied `HMAccessory.profiles` count

B391 audited the `HMAccessory.profiles` getter as a distinct, bounded read-only snapshot. The iOS 26.5 public header declares `NSArray<HMAccessoryProfile *> *profiles` as `readonly, copy` without `nonatomic`, so the Objective-C property is atomic by default; it is available from iOS 11.0. Apple defines the value as the array of profiles implemented by the accessory. The exact candidate contract is only the array's point-in-time count for a valid host-supplied `HMAccessory`.

The local `objc2-home-kit` 0.3.2 generated binding exposes `profiles(&self) -> Retained<NSArray<HMAccessoryProfile>>` under the `HMAccessoryProfile` feature. The method is generated `unsafe` as an Objective-C call but has no method-specific non-atomic or thread-safety warning. Both `HMAccessory` and `HMAccessoryProfile` are `NS_SWIFT_SENDABLE` in the public headers and generated as `Send`/`Sync`; `objc2-foundation` 0.3.2 exposes `NSArray::len()` as a safe count operation. The retained immutable array supplies the returned object's lifetime. A narrow wrapper can therefore contain the typed getter's `unsafe` call and read only its length without iterating, copying, or inspecting any profile objects.

GO candidate: an iOS 11.0+ `profile_count(&HMAccessory) -> usize` host-supplied snapshot. Its result means only the number of profiles in that accessory's returned array at the call time. It does not establish that a profile feature is available, enabled, reachable, or operational; it does not read profile/service identifiers, characteristics, camera controls, or other accessory data; and it does not create a manager, enumerate accessories, prompt, or invoke a HomeKit operation. The host continues to own HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and the supplied accessory lifetime. Do not change aggregate row coverage until implementation review and its focused gates pass.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h:123-126` and `HMAccessoryProfile.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs` and `HMAccessoryProfile.rs`; local `objc2-foundation` 0.3.2 `src/array.rs`; Apple [`HMAccessory.profiles`](https://developer.apple.com/documentation/homekit/hmaccessory/profiles?changes=_7_1&language=objc), [`HMAccessoryProfile`](https://developer.apple.com/documentation/homekit/hmaccessoryprofile?language=objc), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No source, dependency, package feature, manifest, manager creation, permission request, accessory/profile read, build, link probe, test, app launch, Simulator run, or device query was performed for B391

## B339 implementation: host-supplied `HMAccessory.supportsIdentify`

B327 rechecked the iOS 26.5 `HMAccessory.supportsIdentify` getter as a distinct, useful, read-only HomeKit value. The public header declares `@property (readonly) BOOL supportsIdentify` from iOS 11.3 without `nonatomic`; Apple documents that `false` makes `identifyWithCompletionHandler:` return an error, while `true` does not guarantee a later identify call succeeds. The `HMAccessory` class is marked `NS_SWIFT_SENDABLE`. The generated `objc2-home-kit` 0.3.2 getter is typed as `unsafe fn supportsIdentify(&self) -> bool`, with no method-specific thread-safety or `# Safety` note; unlike `isReachable`, it has no non-atomic warning. This supports an isolated scalar getter for a valid host-supplied live `HMAccessory` without a serialized queue or manager lifecycle inside the package.

B339 adds the separate `platform/ios/ios-homekit-identify-status` crate and `docs/ios/homekit-identify-status.md`. `ios_homekit_identify_status::supports_identify(&HMAccessory)` returns Apple's exact per-accessory action-support bit on iOS 11.3+ and returns `NativeApiUnavailable` on older iOS. The call borrows but does not create or retain the accessory. The crate does not create `HMHomeManager`, request permission, enumerate accessories, invoke identify, or claim HomeKit readiness, authorization, reachability, or identify success.

The iOS host must provide a valid live accessory from its own HomeKit lifecycle and owns the HomeKit capability, `com.apple.developer.homekit` entitlement, `NSHomeKitUsageDescription`, prior authorization, and object source/lifetime. `objc2-home-kit` 0.3.2 is pinned with only the `HMAccessory` feature and links `HomeKit.framework`; this slice does not implement a portable HomeKit contract or standalone discovery/status query. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

The focused `platform/ios/ios-homekit-identify-status/check.sh` gate passed with Rust 1.94.1: `cargo fmt --check`; locked host, arm64 iOS device, and arm64 iOS Simulator Cargo checks; strict Clippy for all three targets; host and iOS-device rustdoc; `xtask docs-check`; `xtask zero-swift-source`; and scoped `git diff --check`. It does not add or run tests, invoke HomeKit, create a manager, perform a link probe, launch an app, use a Simulator at runtime, or query a device.

## B394 implementation: host-supplied `HMAccessory.profiles` count

B394 adds `platform/ios/ios-homekit-accessory-profile-count` and `docs/ios/homekit-accessory-profile-count.md`. Its public API is `profile_count(&HMAccessory) -> Result<usize, HomeKitAccessoryProfileCountError>` from iOS 11.0. The host supplies a live accessory already obtained through its HomeKit lifecycle. The function requests the typed `HMAccessory.profiles` getter and reads only `NSArray::len()`; it does not inspect or copy profile elements and does not claim profile-feature availability, readiness, reachability, or operation success.

The contained unsafe getter is grounded in the iOS 26.5 public `HMAccessory.h` declaration `@property (readonly, copy) NSArray<HMAccessoryProfile *> *profiles`, available from iOS 11.0. The declaration omits `nonatomic`, so the Objective-C property uses the atomic default. The exact `objc2-home-kit` 0.3.2 generated typed getter returns a retained `NSArray<HMAccessoryProfile>` and has no non-atomic/thread-safety warning. `objc2-foundation` 0.3.2 exposes safe `NSArray::len()`. The retained array remains alive through the count read; no element getter is called.

The host owns the HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and accessory source/lifetime. The crate does not create `HMHomeManager`, request permission, enumerate accessories, read profile data, or invoke a HomeKit operation. This count does not change row 072 beyond a partial implementation and does not establish profile availability or general HomeKit readiness. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

The focused `platform/ios/ios-homekit-accessory-profile-count/check.sh` gate passed with Rust 1.94.1: formatting; locked/offline host, arm64 iOS device, and arm64 iOS Simulator checks; strict Clippy for all three targets; host and iOS-device rustdoc; `xtask docs-check`; and `xtask zero-swift-source`. No tests, runtime calls, link probes, apps, or device queries were run. CI runs this focused script on macOS.

## B396 audit — `HMAccessory.services` has no safe standalone Rust count contract

B396 audited the typed `HMAccessory.services` array as a potential count-only snapshot. The iOS 26.5 public header declares `@property (nonatomic, readonly, copy) NSArray<HMService *> *services`; Apple's documentation describes the array as the services provided by an accessory. The property is explicitly nonatomic, so a count-only wrapper would still need to make the property read itself safe before it could call `NSArray::len()`.

The local `objc2-home-kit` 0.3.2 generated binding provides a typed `unsafe fn services(&self) -> Retained<NSArray<HMService>>`. Its generated safety documentation states both that the property is not atomic and that the call might not be thread-safe. The copied, retained array describes object ownership after the getter returns; it does not make the nonatomic getter race-free. The reviewed HomeKit public docs specify no executor, queue, or serialization rule that would support a generally safe Rust call, and a main-thread check alone would not supply one. In addition, Apple describes `HMService` as a controllable accessory feature, so exposing service objects or inspecting their contents would exceed a count-only capability snapshot.

Decision: no B396 Rust facade, package, or dependency change. Do not wrap the unsafe getter as a safe Rust count without a documented host-owned serialization contract. This does not alter row 072 or the B394 profile-count contract. The host continues to own the HomeKit lifecycle, capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and supplied accessory lifetime. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h:121`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; local `objc2-foundation` 0.3.2 `src/array.rs`; Apple [`HMAccessory.services`](https://developer.apple.com/documentation/homekit/hmaccessory/services?language=objc) and [`HMService`](https://developer.apple.com/documentation/homekit/hmservice).

No dependency, source, manager creation, permission request, accessory/service read, service enumeration, build, link probe, test, app launch, Simulator run, or device query was performed for B396

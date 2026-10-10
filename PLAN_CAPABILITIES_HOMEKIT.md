# PLAN_CAPABILITIES_HOMEKIT.md — B173: HomeKit authorization no-go

## Audit scope

## Completed shared tooling and HomeKit-specific evidence

The PATH-installed validator is completed infrastructure, and its current `ios-homekit-identify-status` profile covers only the explicitly declared host-supplied identify snapshot assertions. This does **not** validate every HomeKit slice recorded below, authorize a prompt-triggering `HMHomeManager` initialization, or prove live accessory behavior. Continue to preserve each distinct B-numbered no-go, host-supplied-object, entitlement, availability, and test limitation in this historical record. For additional HomeKit slices, extend versioned declarative validation and optional bounded Python adapters only where they express the actual acceptance conditions; keep existing focused scripts and compiler/import gates until their coverage is matched, including negative regression tests. Read `docs/SHARED_TOOLING.md`; do not restore the engine source.


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

## B437 audit — `HMAccessory.firmwareVersion` has no safe standalone Rust snapshot contract

B437 audited the typed, nullable `HMAccessory.firmwareVersion` string as a possible host-supplied snapshot. The iOS 26.5 public header declares it `nonatomic`, `readonly`, and `copy`, available from iOS 11.0. Apple defines it as the accessory's firmware version and also documents a delegate callback when that value changes; the reviewed sources do not define a serialized callback queue or concurrent-read contract.

The local `objc2-home-kit` 0.3.2 binding exposes `pub unsafe fn firmwareVersion(&self) -> Option<Retained<NSString>>`. Its generated safety documentation states that the property is not atomic and that the call might not be thread-safe. `copy` and retained-object ownership do not make the nonatomic property read race-free. A Rust-owned string snapshot would still need a safe getter call and would expose firmware metadata rather than HomeKit support, authorization, or readiness. A main-thread check alone does not establish HomeKit update serialization.

Decision: no B437 Rust facade or dependency change. Revisit only within a selected host that documents serialized access and has a concrete firmware-metadata purpose. This leaves row 072 and B394 unchanged; the host still supplies the live accessory and owns the HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and object lifetime. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h:147`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.firmwareVersion`](https://developer.apple.com/documentation/homekit/hmaccessory/firmwareversion), [`HMAccessoryDelegate` firmware update callback](https://developer.apple.com/documentation/homekit/hmaccessorydelegate/accessory%28_%3Adidupdatefirmwareversion%3A%29), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory metadata read, build, link probe, test, app launch, Simulator run, or device query was performed for B437

## B438 audit — `HMAccessory.model` has no safe standalone Rust snapshot contract

B438 audited the distinct nullable `HMAccessory.model` string. The iOS 26.5 public header declares `model` as `nonatomic`, `readonly`, and `copy`, available from iOS 11.0; Apple defines it as the accessory's model name. The value may be useful to a selected HomeKit host for presentation, but it does not state support, authorization, reachability, or readiness.

The local `objc2-home-kit` 0.3.2 binding exposes `pub unsafe fn model(&self) -> Option<Retained<NSString>>` and generates the same property-not-atomic and might-not-be-thread-safe warning. Copy semantics do not supply an atomic read, and the reviewed public HomeKit sources specify no executor or serialized read/update contract. A safe Rust-owned string conversion therefore cannot be justified for an arbitrary caller-supplied live accessory; checking for the main thread alone would not establish the missing HomeKit synchronization guarantee.

Decision: no B438 Rust facade or dependency change. Revisit only inside a selected host that owns a documented serialized access contract and a concrete model-metadata purpose. This leaves row 072 and B394 unchanged; the host continues to own HomeKit lifecycle, capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and accessory lifetime. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h:137`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.model`](https://developer.apple.com/documentation/homekit/hmaccessory/model), [`HMAccessory`](https://developer.apple.com/documentation/homekit/hmaccessory), and [Enabling HomeKit in your app](https://developer.apple.com/documentation/homekit/enabling-homekit-in-your-app).

No dependency, source, manager creation, permission request, accessory metadata read, build, link probe, test, app launch, Simulator run, or device query was performed for B438

## B440 audit — `HMAccessory.name` has no safe standalone Rust snapshot contract

B440 was unused before audit. It rechecked the host-visible accessory name as a distinct read-only property. The iOS 26.5 public header declares `name` as `nonatomic`, `readonly`, and `copy`; Apple documents that HomeKit associates the name with the accessory and exposes an update operation and delegate callback when it changes. The reviewed Apple sources do not define the callback queue or a concurrent getter/update contract.

The local `objc2-home-kit` 0.3.2 binding exposes `pub unsafe fn name(&self) -> Retained<NSString>` and explicitly warns that the property is not atomic and might not be thread-safe. Copy/retain semantics do not resolve the nonatomic read race. The name is host presentation metadata, not a support, authorization, or readiness signal; returning a Rust-owned string would still require an unsupported safe call contract.

Decision: no B440 Rust facade or dependency change. Revisit only inside a selected host with a documented serialized access contract and a concrete name-display need. This leaves row 072 and B394 unchanged; the host owns the live accessory, HomeKit capability, entitlement, usage description, prior authorization, and lifetime. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessory.h:45`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessory.rs`; Apple [`HMAccessory.name`](https://developer.apple.com/documentation/homekit/hmaccessory/name), [`HMAccessory.updateName`](https://developer.apple.com/documentation/homekit/hmaccessory/updatename%28_%3Acompletionhandler%3A%29), and [`HMAccessoryDelegate.accessoryDidUpdateName`](https://developer.apple.com/documentation/homekit/hmaccessorydelegate/accessorydidupdatename%28_%3A%29).

No dependency, source, manager creation, permission request, accessory name read, build, link probe, test, app launch, Simulator run, or device query was performed for B440

## B441 audit — `HMService.isUserInteractive` has no safe standalone Rust snapshot contract

B441 audited a different HomeKit class for a useful host-supplied scalar. Apple documents `HMService.isUserInteractive` as whether a service supports user interaction and directs applications to use it to filter services that users should not directly interact with. This is a meaningful presentation/filtering hint for a host that already owns a valid `HMService`, not a HomeKit authorization or general capability result.

The iOS 26.5 `HMService.h:77` declares `@property (nonatomic, readonly, getter=isUserInteractive) BOOL userInteractive`, available from iOS 9.0. The local `objc2-home-kit` 0.3.2 binding exposes the typed `pub unsafe fn isUserInteractive(&self) -> bool`; its generated safety docs explicitly say the property is not atomic and might not be thread-safe. Although `HMService` is marked `NS_SWIFT_SENDABLE`, that class annotation does not add a serialized executor guarantee to this nonatomic getter. The reviewed public docs define no queue or synchronization rule that would make an unrestricted safe Rust call sound. A main-thread check alone would not establish HomeKit serialization.

Decision: no B441 Rust facade or dependency change, even for a host-supplied service. Revisit only inside a selected HomeKit host that owns a documented serialized access contract; do not obtain a service through B396's rejected nonatomic `HMAccessory.services` getter. This leaves row 072 and B394 unchanged. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMService.h:77`; local `objc2-home-kit` 0.3.2 `src/generated/HMService.rs`; Apple [`HMService.isUserInteractive`](https://developer.apple.com/documentation/homekit/hmservice/isuserinteractive) and [`HMService`](https://developer.apple.com/documentation/homekit/hmservice).

No dependency, source, service creation, manager creation, permission request, service getter call, build, link probe, test, app launch, Simulator run, or device query was performed for B441

## B442 audit — `HMAccessoryProfile` has no safe typed metadata getter

B442 was unused before audit. The iOS 26.5 public `HMAccessoryProfile.h` declares three read-only object properties: `uniqueIdentifier` is `nonatomic, copy`, `services` is `nonatomic, strong`, and `accessory` is `nonatomic, weak`. The class is available from iOS 10.0, marked `NS_SWIFT_SENDABLE`, and its initializer is unavailable. Apple documents this as an abstract superclass for profile subclasses; profiles come from accessory lifecycle data rather than direct construction.

The local `objc2-home-kit` 0.3.2 generated binding has typed `uniqueIdentifier`, `services`, and `accessory` getters, but each is `unsafe` with explicit generated notes that the property is not atomic and might not be thread-safe. `HMAccessoryProfile`'s class-level `Send`/`Sync` implementations do not remove those per-property warnings. A retained UUID or service array does not make the nonatomic getter safe; the weak accessory result also needs its native lifetime semantics preserved. A host may already have a profile object, for example from an accessory callback, but the reviewed public sources do not define a serialized callback/access contract that would make these getters safe for a general Rust wrapper.

Decision: no B442 Rust facade or dependency change. Do not read profile elements through B394's `HMAccessory.profiles` getter and do not use a raw selector or guessed profile-layout/object contract. B394 remains a length-only read that never enters an element. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Next candidate for root review: a caller-supplied `HMAccessoryCategory.categoryType` snapshot has a better property-level contract. `HMAccessoryCategory.h` marks the type `NS_SWIFT_SENDABLE` and declares `categoryType` as `readonly, copy` without `nonatomic`, while the generated typed getter has no non-atomic/thread-safety warning. However, `HMAccessoryCategory.init` is unavailable and the usual source is `HMAccessory.category`, which B389 found nonatomic and unsafe. Consider that candidate only if a host can supply an already-owned live category object and the Rust API claims no category support/readiness semantics; do not acquire it through B389's getter inside the crate.

Evidence: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessoryProfile.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessoryProfile.rs`; Apple [`HMAccessoryProfile`](https://developer.apple.com/documentation/homekit/hmaccessoryprofile?language=objc) and [`HMAccessoryProfile.services`](https://developer.apple.com/documentation/homekit/hmaccessoryprofile/services?language=objc). The next-candidate evidence is `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HomeKit.framework/Headers/HMAccessoryCategory.h`; local `objc2-home-kit` 0.3.2 `src/generated/HMAccessoryCategory.rs`; Apple [`HMAccessoryCategory.categoryType`](https://developer.apple.com/documentation/homekit/hmaccessorycategory/categorytype), [`Accessory Category Types`](https://developer.apple.com/documentation/homekit/accessory-category-types), and [`HMAccessory.category`](https://developer.apple.com/documentation/homekit/hmaccessory/category).

No dependency, source, manager creation, permission request, profile object read, profile element read, build, link probe, test, app launch, Simulator run, or device query was performed for B442

## B449 implementation — host-supplied `HMAccessoryCategory.localizedDescription`

B449 adds `platform/ios/ios-homekit-category-description` and `docs/ios/homekit-category-description.md`. The API is `localized_description(&HMAccessoryCategory) -> Result<String, HomeKitCategoryDescriptionError>` on iOS 9.0+. It reads only the localized category description from an already host-supplied live category object and copies the returned retained `NSString` into Rust-owned storage. Its value is presentation text, not a stable identifier, HomeKit support, authorization, reachability, profile availability, or readiness.

The caller-supplied object is useful for a host that already holds an accessory's category and needs localized text to help a person distinguish accessory types. This crate does not create the category, call `HMAccessory.category` (the nonatomic B389 no-go), access profiles or services, or manage HomeKit lifecycle. Apple marks `HMAccessoryCategory.init` unavailable, so the host owns the category object's acquisition and lifetime as well as HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, and prior authorization.

The iOS 26.5 SDK declares `HMAccessoryCategory` `NS_SWIFT_SENDABLE` and available from iOS 9.0. Its `localizedDescription` property is `readonly, copy` without `nonatomic`, so Objective-C's atomic default applies. The generated `objc2-home-kit` 0.3.2 getter is a typed unsafe Objective-C call with no non-atomic/thread-safety warning; the class is generated `Send`/`Sync`; the returned retained `NSString` remains alive while `to_string()` copies it into Rust-owned storage. This is separate from B445's raw `categoryType` identifier and uses no Swift ABI.

The focused `platform/ios/ios-homekit-category-description/check.sh` runs format, locked/offline host/device/Simulator checks, strict Clippy for all three targets, host/device rustdoc, `xtask docs-check`, and `xtask zero-swift-source`; CI runs it on macOS. No tests, manager/prompt/runtime calls, category acquisition, profile/service reads, app launch, Simulator run, or device query are included. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

## B452 implementation — host-supplied `HMAccessorySetupResult` identifier count

B452 adds `platform/ios/ios-homekit-setup-result-count` and `docs/ios/homekit-setup-result-count.md`. Its API is `setup_accessory_identifier_count(&HMAccessorySetupResult) -> Result<usize, HomeKitSetupResultCountError>` on iOS 15.4+. The caller supplies the result from its own successful setup flow. The function reports only the length of the result's `accessoryUniqueIdentifiers` array; it does not read or disclose UUID elements, enumerate services or profiles, or query the current home inventory. Apple's contract says these IDs correspond to the accessories set up and that bridge setup can return multiple IDs, which gives a host a narrow count for that result without exporting identity data.

The iOS 26.5 SDK declares `HMAccessorySetupResult` available from iOS 15.4, `NS_SWIFT_SENDABLE`, `NSCopying`, and `-init`/`+new` unavailable. `accessoryUniqueIdentifiers` is `readonly, copy` with no `nonatomic`, so the property has Objective-C's atomic default. The local generated `objc2-home-kit` 0.3.2 typed getter returns a retained `NSArray<NSUUID>` and has no non-atomic/thread-safety warning; the generated class is `Send`/`Sync`; `objc2-foundation` 0.3.2 provides safe `NSArray::len()`. The array remains retained through the count read and no UUID element is accessed.

The host owns the accessory-setup flow, user UI, supplied result lifetime/source, HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, and prior authorization. The crate does not create `HMAccessorySetupManager`, run setup, prompt, enumerate accessories, or access a manager. It does not claim HomeKit authorization, ongoing availability, reachability, operation success, or general readiness. Apple's current `HMAccessorySetupResult` docs page carries a preliminary/beta documentation notice; this slice claims only the installed iOS 26.5 public header contract and does not claim runtime behavior. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

The focused `platform/ios/ios-homekit-setup-result-count/check.sh` runs format, locked/offline host/device/Simulator checks, strict Clippy for all three targets, host/device rustdoc, `xtask docs-check`, and `xtask zero-swift-source`; CI runs it on macOS. No tests, setup/UI/manager/prompt/runtime calls, home or accessory identifier reads, service/profile enumeration, app launch, Simulator run, or device query are included.

## B445 implementation — host-supplied `HMAccessoryCategory.categoryType`

B445 adds `platform/ios/ios-homekit-category-type` and `docs/ios/homekit-category-type.md`. The API is `category_type(&HMAccessoryCategory) -> Result<String, HomeKitCategoryTypeError>` on iOS 9.0+. It reads only the raw category identifier from a host-supplied live category object and copies the retained native `NSString` into Rust-owned storage. It preserves unknown identifier strings and performs no enum conversion or normalization. It does not claim HomeKit support, authorization, reachability, profile availability, or readiness.

The host-supplied object contract is deliberate: the crate never calls `HMAccessory.category`, whose nonatomic getter is B389's no-go, and does not create a category object. Apple says `HMAccessoryCategory` instances describe accessory classification and that `categoryType` contains the associated accessory's category identifier. The SDK makes `init` unavailable, so the host must already have obtained a valid object through its own HomeKit lifecycle and transfer/borrow it for this call. The host owns HomeKit capability, `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and the category object's source/lifetime.

The iOS 26.5 SDK declares `HMAccessoryCategory` `NS_SWIFT_SENDABLE` and available from iOS 9.0; `categoryType` is `readonly, copy` without `nonatomic`, so it has Objective-C's atomic default. The generated `objc2-home-kit` 0.3.2 typed getter is `unsafe` as an Objective-C call but has no non-atomic/thread-safety note, and the generated class is `Send`/`Sync`. Its retained `NSString` keeps the native result alive until `to_string()` copies it. The crate uses only the `HMAccessoryCategory` binding feature and no Swift ABI.

The focused `platform/ios/ios-homekit-category-type/check.sh` runs format, locked/offline host/device/Simulator checks, strict Clippy for all three targets, host/device rustdoc, `xtask docs-check`, and `xtask zero-swift-source`; CI runs it on macOS. No tests, manager/prompt/runtime calls, category acquisition, profile/service reads, app launch, Simulator run, or device query are included. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

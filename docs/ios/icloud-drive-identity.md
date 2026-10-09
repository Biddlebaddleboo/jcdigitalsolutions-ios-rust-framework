# iOS iCloud Drive identity-presence backend

`ios-cloud` implements the portable `framework-cloud::UbiquityIdentityBackend` contract by reading
`NSFileManager.defaultManager().ubiquityIdentityToken` and immediately reducing the optional token
to a Boolean presence value. The only public result is `TokenPresent` or `TokenAbsent`. The native
token is not returned, retained beyond the getter's temporary result, compared, logged, persisted,
or converted to text.

## API and availability evidence

The installed iOS SDK header declares `ubiquityIdentityToken` as nullable and
`API_AVAILABLE(ios(6.0))`. This is the adapter's minimum iOS API floor; apps must not call it on an
older runtime. The generated `objc2-foundation` 0.3.2 surface exposes `NSFileManager::defaultManager`
and the safe `ubiquityIdentityToken(&self) -> Option<Retained<AnyObject>>` getter. The backend needs
Foundation features `std`, `NSFileManager`, and `NSObject`; `std` is required by this binding crate
to enable allocation support. It makes no direct unsafe call or Objective-C binding of its own.
Apple describes the property as fast enough to check at app launch and states
the shared `FileManager` methods are thread-safe. The backend is synchronous and does not require a
main-thread marker.

The API floor is declaration-derived, not runtime-device evidence. The device and simulator
`cargo check` commands validate binding and compile compatibility against the installed SDK only.

## Availability and host setup

Apple documents `nil` when iCloud is unavailable or no user is logged in. QA1935 further identifies
missing iCloud capability, a disabled per-app iCloud Drive setting, and device account configuration
as causes of an absent token. The adapter maps every `nil` result to `TokenAbsent` without trying to
classify its cause. A present token represents the current iCloud Drive Documents identity, but
does not prove container access or CloudKit account status. Accessing this property does not connect
to an ubiquity container. CloudKit clients must use CloudKit's account APIs instead.

For a host app to make this observation useful, enable its iCloud capability, select the iCloud
Drive Documents service, use valid signing/provisioning, and enable iCloud Drive for the signed-in
test account and app. The backend itself adds no entitlement and needs no `Info.plist` key. No
specific entitlement payload is inferred or embedded by this crate.

No runtime guarantee is made for an unconfigured host, iCloud outage, device settings, identity
change after the call, container access, CloudKit, sync, or account diagnosis. No iOS device or
simulator runtime behavior is claimed by the compile/lint gates.

## Out of scope

CloudKit requests, ubiquity-container lookup, file access, account names or identifiers, token
comparison or persistence, logging, notifications, sync, and account-change observation are not
implemented. See [the portable contract guide](../capabilities/icloud-drive-identity.md) and the
[B35 plan](../../PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md).

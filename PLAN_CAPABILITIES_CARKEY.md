# PLAN_CAPABILITIES_CARKEY.md — D93: CarKey row 107 feasibility audit

## Scope

Report-only audit of row `107-extension-entitlement-capabilities-carkey`. Inspect the installed iPhoneOS SDK, public CarKey and PassKit declarations, cached Rust bindings, and Apple primary documentation. No global matrix, Cargo, CI, source, or index edits; no builds or tests

## Recommendation

Keep row 107 at `X`. The public CarKey API offers no general static support, eligibility, entitlement, or authorization query. Its nearest status values either describe the generic Wallet/Secure Element surface or require an entitled, foreground vehicle session tied to automaker and Wallet state

`PKPassLibrary.isPassLibraryAvailable()` reports only pass-library availability, not whether passes can be added or whether CarKey works. `PKPassLibrary.isSecureElementPassActivationAvailable` reports whether the device supports Secure Element pass activation; Apple says a special Apple-provided entitlement is required, and the value is false without it. Neither value means that CarKey is supported, a vehicle is compatible, a key is provisioned, or a key can connect

CarKey's own `CarKeyRemoteControl.start(delegate:subscriptionRange:with:)` creates a session to access provisioned vehicles of the app's automaker make. It is an `async throws` Swift API, requires `com.apple.developer.carkey.session`, and Apple says only automakers enrolled in the MFi Program may request that entitlement. `vehicleReports` and `isPassiveEntryAvailable(forVehicle:)` describe the entitled app's current session and specific Wallet vehicles, not a device-level support bit

PassKit has public Objective-C CarKey provisioning configuration and an older vehicle-connection session. The configuration is for a user-facing Wallet provisioning flow; the connection session starts a vehicle connection for a specific Secure Element pass and is deprecated in iOS 16. Neither is a stable, generic, non-session CarKey status query

## Installed SDK and Rust binding evidence

Toolchain: Xcode 26.6, build `17F113`; iPhoneOS SDK 26.5

- The installed framework is `iPhoneOS.sdk/System/Library/Frameworks/CarKey.framework`
- Its public files are `CarKey.tbd`, `Modules/CarKey.swiftmodule/arm64e-apple-ios.swiftinterface`, and `arm64e-apple-ios.swiftdoc`; this framework has no public `Headers/` directory or `module.modulemap`
- The Swift interface declares `CarKeyRemoteControl`, `CarKeyRemoteControlSession`, `CarKeyRemoteControlSessionDelegate`, `VehicleReport`, actions, and value types. The base API is marked iOS 16.0, watchOS 9.0, and macOS 13.3. Selected later APIs have higher floors, such as iOS 18.0 for configurable enduring actions and iOS 26.0 for CarKey-event launch registration
- `CarKeyRemoteControl.start` is `async throws` and requires a delegate. Its optional `DispatchQueue` controls delegate callbacks; when nil, Apple documents a new serial dispatch queue. The API is not declared `@MainActor`, but Apple requires the app to start a session in the foreground
- `CarKey.tbd` exports Swift-mangled symbols and lists target `arm64e-ios`; the interface has no public C or Objective-C declaration for CarKey. This SDK inspection does not establish a Simulator import/link path
- No CarKey-specific crate or CarKey binding exists in the local Cargo registry or repository Rust, C, or Objective-C sources
- Cached `objc2-pass-kit` 0.3.2 does expose generated Objective-C bindings for PassKit neighbors, including `PKAddCarKeyPassConfiguration`, `PKPassLibrary`, `PKSecureElementPass`, and `PKVehicleConnectionSession`. The config type export needs `PKAddCarKeyPassConfiguration` plus `PKAddSecureElementPassConfiguration`; the legacy session method needs `PKVehicleConnectionSession`, `PKObject`, `PKPass`, `PKSecureElementPass`, and `block2` (the session feature enables Foundation `NSData` and `NSError`). The Apple header marks that session API deprecated in iOS 16.0
- The public CarKey Swift `async throws` entry has no direct generated Rust binding. The current Swift ABI feasibility record finds no supported public Swift task-entry/context/resume contract; its C++ generation probe also omits async entries. See `PLAN_SWIFT_ABI_ASYNC.md` and `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`

## Callable status-like APIs and limits

| Public API | API floor | What it reports | Why it does not establish CarKey support |
| --- | --- | --- | --- |
| `PKPassLibrary.isPassLibraryAvailable()` | iOS 6.0 | Whether the generic Wallet pass library is available | Apple explicitly says this does not show whether the device can add passes; it says nothing about CarKey, vehicle, or key state |
| `PKPassLibrary.isSecureElementPassActivationAvailable` | iOS 13.4 | Whether this device can create Secure Element passes | Apple says a special entitlement is required and the value is false without it; this is not a CarKey support test |
| `PKPassLibrary.passesOfType(PKPassTypeSecureElement)` | `passesOfType:` iOS 8.0; Secure Element pass type iOS 13.4 | Secure Element passes accessible to the app | `PKPassType` has generic `Barcode` and `SecureElement` cases, not a CarKey pass type. PassKit returns only passes the process may access under its Wallet entitlement |
| `PKSecureElementPass.passActivationState` | iOS 13.4 | Activation state for one Secure Element pass | This is a per-pass value, not a generic CarKey capability; the input pass may be a payment or another Secure Element pass |
| `CarKeyRemoteControlSession.vehicleReports` | iOS 16.0 | Provisioned vehicles of the entitled app's make in an active session | Requires session creation and exposes Wallet vehicle state; an empty list cannot distinguish no matching key from other access or environment limits |
| `CarKeyRemoteControlSession.isPassiveEntryAvailable(forVehicle:)` | iOS 16.0 | Whether passive entry is currently available for one vehicle in that session | Vehicle-specific and session-bound, not device-level or general CarKey support |
| `PKVehicleConnectionSession.connectionStatus` | iOS 15.4; deprecated iOS 16.0 | State of a connection session for one `PKSecureElementPass` | Starts from a specific pass through a delegate/session API; the API is deprecated and its states do not report platform support |

`PKPassTypeSecureElement` and `PKSecureElementPass` are generic Secure Element abstractions. Apple says sensitive passes such as payment cards and digital car keys use `PKSecureElementPass`, but the public `PKPassType` enum does not provide a CarKey-specific case. `localizedName` or `localizedDescription` is display content, not a stable machine discriminator. A manufacturer-specific pass identifier could only describe that manufacturer's own Wallet integration; it would not yield a general CarKey capability value

The generic pass-list route also has a privacy boundary: it can expose sensitive Wallet pass data, and `PKPassLibrary` instances are not thread-safe. The Pass Type IDs entitlement, `com.apple.developer.pass-type-identifiers`, scopes which pass types an app may access. This Wallet entitlement is distinct from the restricted CarKey session entitlement

## Entitlement, device, Wallet, and host limits

- Apple requires `com.apple.developer.carkey.session` to use CarKey and says the entitlement request is for automakers enrolled in Apple's MFi Program
- CarKey is for automaker apps and vehicles already provisioned to Apple Wallet. The remote-control session filters the user's Wallet vehicle records to the app's company make; vehicle functions, action identifiers, and returned execution states are manufacturer-defined
- A CarKey session can be active only one at a time. Apple says to start it in the foreground; the system ends it when the app enters the background, and the app must create a new session after it returns to the foreground
- `registerForLaunchOnCarKeyEvent()` is an iOS 26.0 lifecycle registration, not a status query; Apple notes that the system may not relaunch an app under some conditions
- `PKAddCarKeyPassConfiguration` is a separate PassKit provisioning input with an iOS 13.4 floor. Its one-time `password` comes from the vehicle manufacturer; Apple says PassKit verifies it while it creates a pass through `PKAddSecureElementPassViewController`. This user-facing provisioning flow is not a passive support check
- Apple Wallet access also depends on the Wallet capability and `com.apple.developer.pass-type-identifiers` when an app reads passes; the entitlement lists pass types available to the app and is code-signing configuration, not a runtime capability query
- Apple's current Wallet setup guidance requires a compatible car, a signed-in Apple Account, and supported iPhone or Apple Watch hardware. The guide lists iPhone XS or later / iPhone SE (2nd generation), or Apple Watch Series 5 or later / Apple Watch SE, for key setup. Passive and remote entry have higher device requirements; remote entry also requires Bluetooth range. These product and vehicle requirements are distinct from the CarKey framework's iOS 16 API floor
- Vehicle compatibility, key provisioning, Apple Account state, radio reach, vehicle configuration, entitlement approval, and automaker service state cannot be inferred from an OS version or generic Secure Element value

## Disposition and root integration

Row 107 remains `X`; no portable value or iOS backend is proposed. A no-prompt generic status contract would mislabel Wallet or Secure Element availability as CarKey support. The only CarKey-specific state values inspected require a privileged active session and relate to particular Wallet vehicles. The CarKey Swift surface also lacks a supported Rust call path in the current ABI work

No package, dependency, workspace, lock, CI, docs-index, matrix, or count change is required. Keep the canonical row's unverified null metadata unchanged unless root elects to add a scoped blocker note. If root updates `status_reason`, a precise summary is: `No general Rust-callable CarKey support query exists; CarKey remote-control APIs are Swift-only, require the restricted com.apple.developer.carkey.session entitlement for MFi automakers, and operate on provisioned Wallet vehicles through foreground sessions`

## Audit record

- Read `CarKey.framework` Swift interface and `.tbd`, PassKit public headers for CarKey and Secure Element APIs, and row `107-extension-entitlement-capabilities-carkey`
- Inspected local `objc2-pass-kit` 0.3.2 feature and generated binding files; searched Cargo registry and repository Rust, C, and Objective-C source for CarKey-specific bindings
- Reviewed Apple CarKey, PassKit Wallet, Pass Type IDs entitlement, MFi, Secure Element, vehicle-session, and Wallet setup documentation
- No code, manifest, lock, CI, matrix, build, link probe, test, entitlement, app, or runtime query changed or ran

## Apple primary sources

- [CarKey overview](https://developer.apple.com/documentation/carkey)
- [`CarKeyRemoteControl`](https://developer.apple.com/documentation/carkey/carkeyremotecontrol)
- [`CarKeyRemoteControl.start`](https://developer.apple.com/documentation/carkey/carkeyremotecontrol/start%28delegate%3Asubscriptionrange%3Awith%3A%29)
- [`CarKeyRemoteControlSession`](https://developer.apple.com/documentation/carkey/carkeyremotecontrolsession)
- [`registerForLaunchOnCarKeyEvent()`](https://developer.apple.com/documentation/carkey/carkeyremotecontrol/registerforlaunchoncarkeyevent%28%29)
- [`PKPassLibrary.isPassLibraryAvailable()`](https://developer.apple.com/documentation/passkit/pkpasslibrary/ispasslibraryavailable%28%29)
- [`PKPassLibrary.isSecureElementPassActivationAvailable`](https://developer.apple.com/documentation/passkit/pkpasslibrary/issecureelementpassactivationavailable)
- [`PKPassLibrary.passes(of:)`](https://developer.apple.com/documentation/passkit/pkpasslibrary/passes%28of%3A%29)
- [`PKPass.secureElementPass`](https://developer.apple.com/documentation/passkit/pkpass/secureelementpass)
- [`PKSecureElementPass`](https://developer.apple.com/documentation/passkit/pksecureelementpass)
- [`PKAddCarKeyPassConfiguration`](https://developer.apple.com/documentation/passkit/pkaddcarkeypassconfiguration)
- [`PKVehicleConnectionSession`](https://developer.apple.com/documentation/passkit/pkvehicleconnectionsession)
- [Pass Type IDs entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.pass-type-identifiers)
- [Apple Wallet: Add your car key](https://support.apple.com/en-us/118271)
- [Apple MFi Program](https://developer.apple.com/mfi/)

## Internal evidence paths

- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/CarKey.framework/Modules/CarKey.swiftmodule/arm64e-apple-ios.swiftinterface`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/CarKey.framework/CarKey.tbd`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/PassKit.framework/Headers/PKAddCarKeyPassConfiguration.h`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/PassKit.framework/Headers/PKVehicleConnectionSession.h`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/PassKit.framework/Headers/PKPassLibrary.h`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/PassKit.framework/Headers/PKPass_Types.h`
- `/Users/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-pass-kit-0.3.2/src/generated/PKVehicleConnectionSession.rs`
- `/Users/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-pass-kit-0.3.2/src/generated/PKAddCarKeyPassConfiguration.rs`
- `PLAN_SWIFT_ABI_ASYNC.md`, `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`
- `docs/capabilities/capability-status.json`, row `107-extension-entitlement-capabilities-carkey`

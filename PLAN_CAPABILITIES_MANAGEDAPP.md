# D89 — ManagedApp/Distribution feasibility (row 103)

## Disposition

Keep row `103-extension-entitlement-capabilities-managedapp-distribution` at X. The installed SDK has useful managed configuration and app catalog APIs, but both frameworks expose Swift-only modules. The generated `objc2` framework list also classifies `ManagedApp` and `ManagedAppDistribution` as Swift-only; no Rust, C, or Objective-C binding exists in the inspected local source. A direct Rust slice would need a new Swift bridge and, for ManagedAppDistribution, a special entitlement and a device-management app context.

Do not map ordinary app state to MDM-managed or supervised state. No generic `isManaged` or `isSupervised` query appears in either framework interface. `ManagedAppConfigurationProvider` yields app-specific admin configuration, and `ManagedAppLibrary.availableApps` yields a managed app catalog. Neither is a general device-management status property.

## Local SDK and binding evidence

The inspected toolchain is Xcode 26.6, build `17F113`, with iPhoneOS SDK 26.5.

`ManagedApp.framework` has a Swift module interface with iOS 18.4 availability and no public `Headers` directory or Objective-C `module.modulemap`. Its Swift API includes `ManagedAppConfigurationProvider`, `ManagedAppPasswordsProvider`, `ManagedAppCertificatesProvider`, and `ManagedAppIdentitiesProvider`. The framework export map lists Swift-mangled API symbols.

`ManagedAppDistribution.framework` has a Swift module interface with iOS 17.2 availability for `ManagedAppLibrary`, `ManagedApp`, and `ManagedAppDistributionError`. Its iOS interface exposes `ManagedAppLibrary.currentDistributor.availableApps` as an async sequence of `Result<[ManagedApp], ManagedAppDistributionError>`. `ManagedAppDistributionError.appNotManaged` and `.licenseNotFound` have iOS 18.0 availability. The package and framework contain no public Objective-C headers for these APIs.

The generated Rust framework list marks both frameworks “Swift-only.” A search of local Cargo-registry sources, `bindings/`, and `platform/` found no `ManagedApp` Rust, C, or Objective-C binding. No Cargo command, build, test, or runtime probe ran for this audit.

## ManagedApp configuration and secret boundary

Apple describes ManagedApp as a route for an MDM administrator to provision configuration and secrets for a managed app or extension. `ManagedAppConfigurationProvider.configurations(_:)` returns an async sequence of optional, app-defined decoded values. The sequence yields `nil` when the administrator supplied no configuration or decoding fails; therefore `nil` does not mean “device is unmanaged” or “app is unmanaged.” Configuration can change at runtime, and Apple warns against logging decoded sensitive fields.

Passwords, certificates, and identities are fetched through separate asynchronous providers using identifiers that the administrator provisions. These are app credentials, not scalar policy/status values. The device verifies the requesting executable’s code signature against the installed managed app before it supplies declarative configuration. The inspected ManagedApp docs do not specify a generic permission prompt or a ManagedApp-specific entitlement; access depends on MDM-provisioned data and signature verification. The docs do not say this framework is limited to supervised devices, so do not claim supervision is required or proven.

## ManagedAppDistribution catalog and install lifecycle

Apple describes ManagedAppDistribution as an app-distribution UI framework for device-management solutions. It lists managed apps assigned to a device, can display app metadata and install status/progress, verifies that someone initiated an installation, and can launch an app after download. This is not a general-purpose app-store or MDM-policy query for an arbitrary app.

Use requires the `Managed App Installation UI` entitlement:

- Key: `com.apple.developer.managed-app-distribution.install-ui`
- Type: array of strings
- Normal value: `managed-app`

`ManagedAppLibrary.currentDistributor.availableApps` can yield `ManagedAppDistributionError.deviceNotManaged`, which Apple defines as “this device isn’t managed.” That is a real service error, not a fake boolean. It comes from the entitlement-gated async app-catalog operation; Apple also documents a network failure when the catalog metadata is unavailable. The framework error enum has separate unsupported-platform, app-not-managed, and license-not-found cases for its operations. Success yields the catalog, not a generic MDM/supervision proof; an empty catalog is not evidence that the device is unmanaged. `ManagedAppDistributionError.appNotManaged` means the calling app is not managed, not that the device is unmanaged.

App availability and licenses originate in organization/device-management assignment. Installation, status, progress, and launch belong to that management UI lifecycle. A generic Rust helper cannot control or replace the administrator’s assignments, licensing, download, or install decision.

## Feasibility conclusion

The closest user-visible operation is a managed app catalog, not a portable status snapshot. Its relevant APIs are Swift-only, its use needs `com.apple.developer.managed-app-distribution.install-ui`, and its result is an admin-owned async sequence with network and license errors. The ManagedApp configuration sequence is also Swift-only and app-specific; absent or invalid config cannot safely map to “unmanaged.” Neither path forms a useful direct Rust/C/Objective-C slice within this row without a new Swift interop boundary and an entitled host app.

Keep row 103 at X. If a later workstream accepts Swift interop and the required host capability, define a distinct `ManagedAppDistribution` catalog result that preserves catalog, device-not-managed, app-not-managed, network, unsupported-platform, and license outcomes. Do not add `is_supervised`, `is_managed`, or install-success status based on ordinary app state or absence of MDM config.

## Evidence and checks

- Installed SDK frameworks: `iPhoneOS.sdk/System/Library/Frameworks/ManagedApp.framework` and `ManagedAppDistribution.framework`
- Installed API interfaces: `ManagedApp.framework/Modules/ManagedApp.swiftmodule/arm64e-apple-ios.swiftinterface` and `ManagedAppDistribution.framework/Modules/ManagedAppDistribution.swiftmodule/arm64e-apple-ios.swiftinterface`
- Generated Rust framework coverage: `objc2` generated framework list marks `ManagedApp` and `ManagedAppDistribution` as Swift-only
- Read-only environment checks: Xcode 26.6 build `17F113`; iPhoneOS SDK 26.5
- No tests, builds, link probes, or runtime probes ran

## Primary references

- [ManagedApp framework](https://developer.apple.com/documentation/managedapp)
- [ManagedAppConfigurationProvider](https://developer.apple.com/documentation/managedapp/managedappconfigurationprovider)
- [ManagedApp configuration specification and MDM flow](https://developer.apple.com/documentation/managedapp/specifying-and-decoding-a-configuration)
- [ManagedApp configuration sequence semantics](https://developer.apple.com/documentation/managedapp/managedappconfigurationprovider/configurations%28_%3A%29)
- [ManagedAppDistribution framework](https://developer.apple.com/documentation/managedappdistribution)
- [Fetching and displaying managed apps](https://developer.apple.com/documentation/managedappdistribution/fetching-and-displaying-managed-apps)
- [`ManagedAppLibrary.availableApps`](https://developer.apple.com/documentation/managedappdistribution/managedapplibrary/availableapps)
- [`ManagedAppDistributionError`](https://developer.apple.com/documentation/managedappdistribution/managedappdistributionerror)
- [Managed App Installation UI entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.managed-app-distribution.install-ui)
- [Declarative management for managed apps and extensions](https://developer.apple.com/documentation/devicemanagement/configuring-managed-apps-and-extensions)
- [Generated `objc2` framework coverage list](https://docs.rs/objc2/latest/aarch64-apple-ios/objc2/topics/about_generated/list/index.html)

## B227 follow-up: ManagedApp remains Swift-only and admin-scoped

Rechecked the installed iOS 26.5 `ManagedApp` and `ManagedAppDistribution` interfaces for a C/Objective-C value that could support a Rust-owned management snapshot. The frameworks still expose Swift module interfaces without public Objective-C headers or module maps for the meaningful APIs. `ManagedAppConfigurationProvider.configurations(_:)` is an async sequence of optional, app-defined decoded values; `nil` still conflates absent admin configuration and decode failure. `ManagedAppLibrary.currentDistributor.availableApps` is an async managed catalog sequence, not a generic `isManaged`, `isSupervised`, device-management, or installation-readiness query. The generated `objc2` framework catalog marks both frameworks Swift-only.

The only typed catalog error that directly says `deviceNotManaged` is yielded by the entitlement-gated `ManagedAppDistribution` catalog operation. That operation depends on the `com.apple.developer.managed-app-distribution.install-ui` entitlement and organization/device assignment, and also has catalog/network/license outcomes. It cannot be turned into an entitlement-free Boolean without losing the operation context and error semantics. The app-specific configuration sequence similarly does not establish device supervision or management. No public C/Objective-C query or local generated binding changes these constraints.

Decision: no Rust backend, new interop path, or dependency change for B227; keep row `103-extension-entitlement-capabilities-managedapp-distribution` at `X`. Revisit only for a concrete MDM-managed product that accepts the relevant entitlement and async catalog or configuration contract with all result/error variants preserved. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: installed `ManagedApp.framework/Modules/ManagedApp.swiftmodule/arm64e-apple-ios.swiftinterface` and `ManagedAppDistribution.framework/Modules/ManagedAppDistribution.swiftmodule/arm64e-apple-ios.swiftinterface`; [Apple ManagedApp configuration](https://developer.apple.com/documentation/managedapp/managedappconfigurationprovider), [`ManagedAppLibrary.availableApps`](https://developer.apple.com/documentation/managedappdistribution/managedapplibrary/availableapps), [`ManagedAppDistributionError`](https://developer.apple.com/documentation/managedappdistribution/managedappdistributionerror), and [Managed App Installation UI entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.managed-app-distribution.install-ui).

No source, dependency, build, link probe, test, MDM query, app launch, Simulator run, or device query was performed for B227

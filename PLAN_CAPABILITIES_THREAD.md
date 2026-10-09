# PLAN_CAPABILITIES_THREAD.md — D75: Thread system API feasibility

## Objective

Audit row `045-sensors-connectivity-thread-network-system-pieces` for a Rust-callable, non-entitled, non-prompting Thread status or value that reports real system capability, not only an OS-version proxy

## Status

No honest qualifying slice is available in the inspected public API surface. Keep row 045 unsupported (`X`)

`THClient.isPreferredNetworkAvailable(completion:)` is a meaningful scalar for a different, narrower claim: whether a preferred Thread network is available in Apple’s framework. The API is part of the entitlement-gated `ThreadNetwork` framework, does not establish that this iPhone has Thread radio support or that a border router is active, and does not meet the no-entitlement constraint

`MatterAddDeviceRequest.isSupported` is a Swift-only Matter setup capability, not a Thread capability. The installed Rust binding catalog marks `MatterSupport` Swift-only, and the property does not report Thread radio, network, or border-router state

## SDK and Rust binding evidence

Inspection used Xcode 26.6 build 17F113, iPhoneOS 26.5 SDK, Swift interface compiler 6.3.2, and Rust/Cargo 1.94.1

- `ThreadNetwork.framework` has public Objective-C headers and a module map. The inspected files were `System/Library/Frameworks/ThreadNetwork.framework/Headers/THClient.h`, `THCredentials.h`, and `Modules/module.modulemap` under the iPhoneOS SDK. `THClient` and `THCredentials` have an iOS 15.0 floor
- `THClient.checkPreferredNetworkForActiveOperationalDataset:completion:` has an iOS 15.5 floor; `retrieveAllActiveCredentials:` and `isPreferredNetworkAvailableWithCompletion:` have iOS 16.4 floors
- `isPreferredNetworkAvailableWithCompletion:` returns a Boolean for preferred-network availability. Apple recommends this query before a credential-consent request; its result is not a device hardware or Thread-radio support result
- The installed `objc2` 0.6.5 generated-framework catalog at `src/topics/about_generated/list_data.md` names `objc2-thread-network`, but that crate is not present in the local Cargo registry cache or root `Cargo.lock`
- `MatterSupport.framework` has a public Swift interface and only a Foundation import in its public umbrella header. The inspected files were `System/Library/Frameworks/MatterSupport.framework/Headers/MatterSupport.h` and `Modules/MatterSupport.swiftmodule/arm64e-apple-ios.swiftinterface` under the iPhoneOS SDK. `MatterAddDeviceRequest` is available from iOS 16.1; its `isSupported` property is available from iOS 17.0
- `MatterAddDeviceRequest.isSupported` is a Swift static `Bool`, with no Objective-C or public C declaration. The installed `objc2` 0.6.5 catalog at `src/topics/about_generated/list_unsupported.md` classifies `MatterSupport` as Swift-only; no generated Rust binding is cached
- The inspected interfaces do not document a callback queue for `THClient` or arbitrary-thread guarantees for the Matter status getter. Do not claim a main-thread requirement or unrestricted thread safety without a source-backed contract

## Entitlement, consent, and host limits

- Apple’s ThreadNetwork setup guide directs an app to enable the Manage Thread Network Credentials development capability. The public entitlement key is `com.apple.developer.networking.manage-thread-network-credentials`, documented as the Boolean that permits ThreadNetwork use
- App Store distribution requires Apple approval for the Thread Network distribution entitlement after conformance tests; the development entitlement alone is not the distribution grant
- `THClient.retrievePreferredCredentials:` and `retrieveCredentialsForExtendedPANID:completion:` document a consent alert. `isPreferredNetworkAvailableWithCompletion:` is a pre-consent availability check, not credential retrieval; the check still sits behind ThreadNetwork access and its entitlement
- `THClient.retrieveAllCredentials:` and `retrieveAllActiveCredentials:` return credential records, not a minimal capability status; do not read or expose those records in a status facade
- Thread border-router setup requires a configured Border Router and a real Thread network. Apple identifies HomePod, HomePod mini, and Apple TV 4K as Border Router examples; iPhone app access to ThreadNetwork does not make the iPhone a border router
- Apple’s ThreadNetwork documentation does not establish that an OS-version check or presence of the framework proves compatible Thread hardware, a reachable Thread network, a configured preferred network, or a working border router
- `MatterAddDeviceRequest.isSupported` only describes Matter add-device request support. `perform()` starts a user interface flow, so it is out of scope for a non-prompting Thread status slice
- The optional Matter entitlement `com.apple.developer.matter.allow-setup-payload` permits an app to supply a Matter setup payload in supported setup flows; it does not grant ThreadNetwork access or prove Thread capability

## Recommendation

Do not add a Thread scalar based only on iOS version, framework presence, `MatterAddDeviceRequest.isSupported`, or preferred-network state. None states that the device can operate as a Thread client or border router

If a later workstream has an approved Thread entitlement and explicit product scope, `isPreferredNetworkAvailable(completion:)` can support a separate async snapshot named for preferred-network availability only. That slice must retain the iOS 16.4 API floor and entitlement/distribution caveat. Apple documents this as a pre-consent check, not credential retrieval; it must not claim radio, active-network, or border-router capability

## Apple primary sources

- [ThreadNetwork framework](https://developer.apple.com/documentation/threadnetwork/)
- [Getting started with ThreadNetwork](https://developer.apple.com/documentation/threadnetwork/getting-started-with-threadnetwork)
- [Manage Thread Network Credentials entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.networking.manage-thread-network-credentials)
- [Managing Thread network credentials](https://developer.apple.com/documentation/threadnetwork/managing-thread-network-credentials)
- [THClient.isPreferredNetworkAvailable(completion:)](https://developer.apple.com/documentation/threadnetwork/thclient/ispreferrednetworkavailable%28completion%3A%29)
- [Configuring a Border Router](https://developer.apple.com/documentation/threadnetwork/configuring-a-border-router)
- [MatterAddDeviceRequest](https://developer.apple.com/documentation/mattersupport/matteradddevicerequest)
- [Adding Matter support to your ecosystem](https://developer.apple.com/documentation/MatterSupport/Adding-Matter-support-to-your-ecosystem)
- [Matter Allow Setup Payload entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.matter.allow-setup-payload)
- [Apple Thread Test Plan — THClient API](https://developer.apple.com/apple-home/downloads/Thread-Test-Plan-THClient-API-R1.pdf)

No tests, builds, link probes, entitlement requests, credential reads, or Matter setup flows were performed

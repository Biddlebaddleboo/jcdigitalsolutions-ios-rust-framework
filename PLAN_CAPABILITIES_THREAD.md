# PLAN_CAPABILITIES_THREAD.md — D75: Thread system API feasibility

## Objective

Audit row `045-sensors-connectivity-thread-network-system-pieces` for a Rust-callable, non-entitled, non-prompting Thread status or value that reports real system capability, not only an OS-version proxy

## Status

The broad row has no non-entitled, general Thread-capability query. B206 adds one explicitly
entitlement-scoped partial: a Rust future for ThreadNetwork's preferred-network availability
Boolean. It does not report device support, local Thread radio, connectivity, or border-router
state. Root owns the aggregate row update; no aggregate matrix change is part of B206.

`THClient.isPreferredNetworkAvailable(completion:)` is a meaningful scalar for a different, narrower claim: whether a preferred Thread network is available in Apple’s framework. The API is part of the entitlement-gated `ThreadNetwork` framework, does not establish that this iPhone has Thread radio support or that a border router is active, and does not meet the no-entitlement constraint

`MatterAddDeviceRequest.isSupported` is a Swift-only Matter setup capability, not a Thread capability. The installed Rust binding catalog marks `MatterSupport` Swift-only, and the property does not report Thread radio, network, or border-router state

## SDK and Rust binding evidence

Inspection used Xcode 26.6 build 17F113, iPhoneOS 26.5 SDK, Swift interface compiler 6.3.2, and Rust/Cargo 1.94.1

- `ThreadNetwork.framework` has public Objective-C headers and a module map. The inspected files were `System/Library/Frameworks/ThreadNetwork.framework/Headers/THClient.h`, `THCredentials.h`, and `Modules/module.modulemap` under the iPhoneOS SDK. `THClient` and `THCredentials` have an iOS 15.0 floor
- `THClient.checkPreferredNetworkForActiveOperationalDataset:completion:` has an iOS 15.5 floor; `retrieveAllActiveCredentials:` and `isPreferredNetworkAvailableWithCompletion:` have iOS 16.4 floors
- `isPreferredNetworkAvailableWithCompletion:` returns a Boolean for preferred-network availability. Apple recommends this query before a credential-consent request; its result is not a device hardware or Thread-radio support result
- The installed `objc2` 0.6.5 generated-framework catalog at `src/topics/about_generated/list_data.md` names `objc2-thread-network`. It was absent from the local registry cache and root `Cargo.lock` at the start of B206; regular Cargo resolution fetched the published 0.3.2 crate and added its lock entry
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

## D75 recommendation

Do not add a Thread scalar based only on iOS version, framework presence, `MatterAddDeviceRequest.isSupported`, or preferred-network state. None states that the device can operate as a Thread client or border router

Before B206, the recommendation was to defer this method until an approved Thread entitlement and explicit product scope existed. B206 now exposes only the caller-requested preferred-network availability scalar and documents the host entitlement/distribution requirement; it does not claim radio, active-network, or border-router capability.

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

D75 performed no tests, builds, link probes, entitlement requests, credential reads, or Matter setup flows

## B206 — preferred Thread network availability partial

`ios-thread-network::request_preferred_network_availability()` invokes only the generated
`THClient.isPreferredNetworkAvailableWithCompletion:` API and returns
`PreferredThreadNetworkAvailabilityFuture<Output = bool>`. The Boolean is the method's exact
preferred-network availability result. The method has an iOS 16.4 floor and is documented by Apple
as the pre-consent availability check before requesting preferred-network credentials. The API
does not return an error, so the Rust future adds no invented failure states.

The host app must carry
`com.apple.developer.networking.manage-thread-network-credentials`; Apple requires distribution
access for publication after its approval and required Thread conformance work. This slice is
therefore not non-entitled Thread support. It reads no credential value, does not prompt or mutate
the Thread credential database, and does not prove Thread radio hardware, active connectivity,
preferred-network membership, Matter support, or border-router capability. It does not change the
existing HomeKit, MatterSupport, or Thread-wide contract into general device readiness.

The installed Xcode 26.6 build `17F113` iPhoneOS SDK 26.5 declares `THClient` in
`ThreadNetwork.framework/Headers/THClient.h` and
`isPreferredNetworkAvailableWithCompletion:` at header lines 218–233 with
`API_AVAILABLE(ios(16.4))`. The crate was not cached locally at the start of B206; regular Cargo
resolution fetched the published `objc2-thread-network` 0.3.2 binding, which exposes typed
`THClient` and the generated Boolean completion method under the `THClient` and `block2` features.
The crate dependency uses
defaults off with only `THClient`, `block2`, and `std`; no Objective-C selector or ABI is handwritten.
`THClient` is `!Send` and `!Sync` in the generated binding. The Rust future retains it on the caller
thread, while its callback captures only an `Arc<Mutex<...>>` and can safely wake a `Waker` if Apple
delivers on an unspecified queue.

Primary references: installed `THClient.h`; Apple's [`isPreferredNetworkAvailable(completion:)`](https://developer.apple.com/documentation/threadnetwork/thclient/ispreferrednetworkavailable%28completion%3A%29), [ThreadNetwork entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.networking.manage-thread-network-credentials), [ThreadNetwork setup guide](https://developer.apple.com/documentation/threadnetwork/getting-started-with-threadnetwork), and [credential management guide](https://developer.apple.com/documentation/threadnetwork/managing-thread-network-credentials); plus the published [`objc2-thread-network` 0.3.2 `THClient`](https://docs.rs/objc2-thread-network/0.3.2/objc2_thread_network/struct.THClient.html) binding.

The focused gate `sh platform/ios/ios-thread-network/check.sh` passed with Rust 1.94.1: format,
iOS device and Simulator checks, strict Clippy for both targets, warnings-denied rustdoc, shell
syntax, source-boundary checks, the selected-feature tree, and documentation index validation.
Cargo resolution added only the
`ios-thread-network` workspace package and `objc2-thread-network` 0.3.2 binding records to
`Cargo.lock`. No test, link probe, runtime query, entitlement request, credential read, or device
call is part of B206. The local Xcode 26.6 / iOS SDK 26.5 evidence is below the repository Xcode
27.x baseline and does not establish Xcode 27.x behavior. Compile success does not establish
runtime behavior or distribution entitlement approval.

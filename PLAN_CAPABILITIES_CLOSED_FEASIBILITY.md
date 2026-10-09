# PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md — completed research and current X-row limitations

This closure index replaces sixteen **completed bounded feasibility audits**, not sixteen implemented capability workstreams. For the exact original audit sources consult Git commit `9bc42dbe7e3c7561d2c0da3e13a8c93c4d852c81`. Their outcomes and detailed current caveats are canonical in `docs/capabilities/capability-status.json`, keyed by capability ID. `X` means unsupported by this framework at the reviewed baseline, **not** impossible on Apple platforms or proof that Rust cannot reach an API. When new SDKs, public interfaces, entitlement approvals, host workflows or Swift-ABI machinery change the premise, open a new bounded workstream rather than silently treating a status scalar as full coverage.

## 033: PushKit

Former `PLAN_CAPABILITIES_PUSHKIT.md` — feasibility audit concluded, row remains **X**.

D100 found no standalone PushKit status query; a write to desiredPushTypes starts PushKit registration, and VoIP delivery needs host launch/delegate, token, provider, and CallKit/LiveCommunicationKit lifecycle; row 033 stays X until a real VoIP product accepts those duties

## 042: SensorKit

Former `PLAN_CAPABILITIES_SENSORKIT.md` — feasibility audit concluded, row remains **X**.

B203 confirms the only narrow Objective-C surface is per-sensor `SRSensorReader.authorizationStatus`, which Apple now deprecates in favor of beta `SRReader<Sensor>` absent from the installed iOS 26.5 SDK and generated binding. `SRSensorReader` requires the Apple-approved `com.apple.developer.sensorkit.reader.allow` research entitlement; the OS closes an app without it. This is not general sensor support or a safe unentitled capability query.

## 075: DeviceActivity/ManagedSettings where residual support exists

Former `PLAN_CAPABILITIES_DEVICE_ACTIVITY.md` — feasibility audit concluded, row remains **X**.

The iOS 17.0 Objective-C DeviceActivityAuthorization.isAuthorized getter is Rust-callable in principle through a minimal typed objc2 binding, but Apple does not define what the Boolean authorizes, its prompt behavior, thread guarantees, or whether query-only use requires com.apple.developer.family-controls. No honest status contract is claimed until these semantics are established. B176 confirms `isAuthorized`, `sharingEnabled`, and `authorizedClientIdentifiers` have no documented useful authorization meaning or query guarantees; `isOverridden` is mutable and does not supply a read-only capability contract.

## 084: PushToTalk

Former `PLAN_CAPABILITIES_PUSHTOTALK.md` — feasibility audit concluded, row remains **X**.

D83 found no standalone PushToTalk support or authorization query. PTServiceStatus is app-set UI state and activeChannelUUID requires manager/channel lifecycle; a useful PTT capability needs entitlement, background mode, microphone consent, APNs channel restoration, and audio/channel lifecycle. Row 084 remains X. B179 confirms `activeChannelUUID` is an instance value only after manager/delegate/restoration lifecycle; `PTServiceStatus` is app-set, and manager errors arise only during that lifecycle. No standalone support or authorization query is exposed.

## 085: CarPlay

Former `PLAN_CAPABILITIES_CARPLAY.md` — feasibility audit concluded, row remains **X**.

D84 found no public CarPlay-wide isAvailable/isSupported/isConnected query. CPSessionConfiguration values describe the connected vehicle environment; useful access requires a category-entitled host app, CarPlay scene manifest, and CPTemplateApplicationScene/session lifecycle. Row 085 remains X. B182 confirms the iOS 26.4 `supportsVideoPlayback` property describes a connected CarPlay session, not device availability or pre-session readiness; category entitlement and host scene/session lifecycle remain required.

## 091: WeatherKit through REST/native HTTP where appropriate

Former `PLAN_CAPABILITIES_WEATHERKIT.md` — feasibility audit concluded, row remains **X**.

D79 found the native WeatherKit API is a Swift-only SDK surface with no public Objective-C header or generated Rust binding. The REST route is a separate product contract: requests require a trusted server-signed ES256 developer token, typed response/data scope, and attribution; generic HTTP/raw JSON alone is not WeatherKit support. Row 091 remains X. B250 confirms Apple WeatherKit REST availability is callable for coordinate-specific data-set availability only, not device, forecast, authorization, or general service readiness. A native product still needs an explicit host-owned ES256 developer-token and attribution boundary plus a typed response contract; `framework-network` supplies only transport and opaque bytes. Row 091 remains X.

## 092: TipKit only if system TipKit behavior is specifically requested; otherwise framework-owned tip logic may be portable

Former `PLAN_CAPABILITIES_TIPKIT.md` — feasibility audit concluded, row remains **X**.

TipKit exposes per-tip eligibility through the Swift-only Tip.status API; this is not global framework readiness or proof that a tip was presented. It requires a host-defined Swift Tip, has no public C/Objective-C interface or generated Rust binding, and its queue/thread-safety contract is undocumented. No supported Rust capability facade is claimed. B191 confirms `Tip.status`/`shouldDisplay` are meaningful only for a concrete Swift `Tip`; there is no C/Objective-C declaration or generated Rust binding, and `TipUIView` is presentation-only.

## 100: ExtensionKit/Foundation where feasible

Former `PLAN_CAPABILITIES_EXTENSIONKIT.md` — feasibility audit concluded, row remains **X**.

D86 found only a Swift-only, asynchronous AppExtensionPoint.Monitor snapshot for a host-defined extension point (iOS 26.0+) and UIKit host/browser controllers requiring extension-point metadata, host UI, owner approval, and process/XPC lifecycle. No generic Rust-callable ExtensionFoundation API or iOS objc2 binding exists; no host-specific contract is selected. See PLAN_CAPABILITIES_EXTENSIONKIT.md. B221 confirms ExtensionFoundation has no Objective-C/C API and AppExtensionPoint.Monitor is a Swift-only async inventory; no standalone host/process support query exists.

## 101: BrowserEngineKit for entitled apps

Former `PLAN_CAPABILITIES_BROWSERENGINEKIT.md` — feasibility audit concluded, row remains **X**.

D88 found generated Objective-C Rust bindings, but useful BrowserEngineKit work needs browser web-content/network/rendering process and XPC lifecycles, Swift-only extension protocol implementations, Apple-issued entitlements, and region-specific distribution approval. BEProcessCapabilityGrant.isValid() is useful only for an already-owned live grant and is not an entitlement/support query. No host or extension implementation exists; keep row 101 X. B218 confirms BEProcessCapabilityGrant.isValid applies only to a live grant from an already-launched browser process; it does not establish entitlement, app eligibility, or BrowserEngineKit support.

## 102: ContactProvider

Former `PLAN_CAPABILITIES_CONTACTPROVIDER.md` — feasibility audit concluded, row remains **X**.

D87 found ContactProvider APIs require iOS 18.0+; ContactProviderManager.isEnabled is Swift-only with no public C/Objective-C entry point or generated Rust binding. It reports whether the person enabled the domain, not extension health or sync; enable() may prompt and manager initialization can register a default domain. The extension requires EXExtensionPointIdentifier=com.apple.contact.provider.extension. See PLAN_CAPABILITIES_CONTACTPROVIDER.md. B224 confirms ContactProviderManager.isEnabled is Swift-only person-enabled state, not extension or sync health; its throwing initializer may register a default domain.

## 103: ManagedApp/Distribution

Former `PLAN_CAPABILITIES_MANAGEDAPP.md` — feasibility audit concluded, row remains **X**.

D89 found ManagedApp and ManagedAppDistribution are Swift-only with no public Objective-C headers or generated Rust binding. ManagedAppConfigurationProvider may return nil for either absent config or decode failure and does not establish supervision; ManagedAppLibrary.currentDistributor.availableApps is an async catalog operation that needs the managed-app distribution entitlement. No generic managed-device status facade is supported; see PLAN_CAPABILITIES_MANAGEDAPP.md. B227 confirms configuration/catalog APIs remain Swift-only and admin/entitlement-scoped, not generic managed-device status.

## 104: MarketplaceKit

Former `PLAN_CAPABILITIES_MARKETPLACEKIT.md` — feasibility audit concluded, row remains **X**.

D90 found AppDistributor.current is an iOS 17.4+ async throws Swift-only installation-source value; AppDistributor.eligibilityRegion and AppLibrary.catalogRegion are narrower async region values. None establishes entitlement, Apple approval, marketplace eligibility, or installation/vending readiness. No public C/Objective-C declaration, generated Rust binding, or supported Rust async Swift ABI path exists. Marketplace operations are region-, approval-, host-, server-, and entitlement-gated; installation requires user action and system UI. Row 104 remains X; see PLAN_CAPABILITIES_MARKETPLACEKIT.md. B230 confirms AppDistributor source/region values are async Swift-only and the Objective-C ActionButton is a user install control, not a status API.

## 106: SecureElementCredential

Former `PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md` — feasibility audit concluded, row remains **X**.

D92 found CredentialSession.isEligible is an iOS 18.1+ Swift async throws query for whether an app or extension is eligible to start a credential session, not generic Secure Element, NFC, or credential-presence support. Secure Element Credential framework calls require com.apple.developer.secure-element-credential, so this is not an unentitled entitlement check. No public C/Objective-C surface or generated Rust binding exists. Useful sessions additionally require Apple NFC & SE Platform approval, eligible product/territory, ABR onboarding, registered applet configuration, foreground use, and user consent. Row 106 remains X; see PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md.; B270 reconfirms CredentialSession.isEligible is an iOS 18.1+ Swift actor async-throws getter with no synchronous swiftcall path; the current Rust boundary has no Swift task or async-error contract. Apple documents SecureElementCredential calls without com.apple.developer.secure-element-credential as fatalError, so the query is not an unentitled entitlement check. Row 106 remains X; see PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md

## 107: CarKey

Former `PLAN_CAPABILITIES_CARKEY.md` — feasibility audit concluded, row remains **X**.

D93 found no general static CarKey support, eligibility, entitlement, or authorization query. PassKit Wallet and Secure Element availability values do not establish CarKey support; CarKey-specific state is Swift-only and bound to an active foreground session with provisioned Wallet vehicles. `CarKeyRemoteControl.start` requires com.apple.developer.carkey.session, which Apple limits to MFi automakers; no CarKey Rust binding or public C/Objective-C CarKey declaration exists. Row 107 remains X; see PLAN_CAPABILITIES_CARKEY.md.

## 109: LockedCameraCapture

Former `PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md` — feasibility audit concluded, row remains **X**.

D95 found no public LockedCameraCapture support or readiness query. The closest APIs are SwiftUI/ExtensionKit capture-extension session state and containing-app content handoff; they do not report whether a compatible extension is installed, configured, authorized, or ready. Useful capture requires an iOS 18.0+ host/extension lifecycle, user-added controls, and camera authorization; the undocumented hasActiveSession symbol is not a supported API. Row 109 remains X; see PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md.; B269 reconfirms sessionContentURLs is a Swift [Foundation.URL] property whose owned array cannot be safely reduced or released as Rust Vec with the current swift-abi-core contract. It is content state, not a capability/readiness predicate; no array adapter or guessed layout is added. Row 109 remains X; see PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md

## 110: App Intents: use result of C Stage 0/1; if unsupported, expose no fake runtime implementation.

Former `PLAN_CAPABILITIES_APP_INTENTS.md` — feasibility audit concluded, row remains **X**.

C7/D99 found Xcode 26.6 metadata extraction but no documented stable Rust/C metadata input, schema, or processor API; the installed processor and observed build arguments do not establish a supported contract. AppIntent.perform() is async, and C6 found no public C/C++ task-entry contract. Stage 1 is unsupported on the audited toolchain; no fake runtime API is claimed.


## Future ownership and evidence

- The corresponding `PLAN_CAPABILITIES_REMAINING_GAPS.md`, `PLAN_CAPABILITIES.md`, and `PLAN_SWIFT_ABI.md` retain the integration and architecture ownership for future implementation where appropriate.
- Public API, entitlement/permission, regional eligibility, callbacks/lifecycle, Swift ABI requirements and host-defined metadata must all be satisfied before changing a row to partial B.
- Never claim Xcode 27 compatibility from the completed Xcode 26.6 / SDK 26.5 feasibility reports.
- For App Intents, `PLAN_SWIFT_APP_INTENTS.md` remains the explicit Xcode-27 follow-up owner; production metadata integration remains blocked.
- The old report text is preserved permanently by commit history. Reopen it only to verify an original evidence detail; do not include it in routine executor context.

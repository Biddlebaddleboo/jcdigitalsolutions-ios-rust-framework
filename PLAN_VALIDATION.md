# PLAN_VALIDATION.md — Workstream G: Tooling, Parity, Performance, CI and Documentation

## Objective

Build the shared machinery that makes the architectural rules enforceable: toolchain manifests, no_std gates, SDK inventory, parity infrastructure, linkage/dependency audits, ABI audits, performance benchmarks, codegen inspection, archive smoke tests, and documentation freshness.

## Dependencies

Foundation must establish workspace/crate names first.
Thereafter this workstream runs continuously and does not own capability implementation.

## Execution decomposition

G46 is [PLAN_VALIDATION_IOS_CLOUDKIT_ACCOUNT_STATUS.md](PLAN_VALIDATION_IOS_CLOUDKIT_ACCOUNT_STATUS.md), which gates D47/B52's portable account-status contract and iOS device/Simulator compile, strict Clippy, and CloudKit/Foundation link/import checks without a live query. These slices may consume the minimal app and C ABI without changing their runtime/API ownership.
G47 is [PLAN_VALIDATION_IOS_SAFETYKIT.md](PLAN_VALIDATION_IOS_SAFETYKIT.md), G48 is [PLAN_VALIDATION_IOS_ARKIT.md](PLAN_VALIDATION_IOS_ARKIT.md), and G49 is [PLAN_VALIDATION_IOS_GAMEKIT.md](PLAN_VALIDATION_IOS_GAMEKIT.md). They gate only D48/B53's SafetyKit support bit, D49/B54's ARKit world-tracking configuration query, and D50/B55's Game Center status snapshot. Each package gate compiles/lints the bounded surface and audits linked imports; none runs a test, executes an Apple probe, or claims live device/service behavior.
G50 is [PLAN_VALIDATION_IOS_COREML.md](PLAN_VALIDATION_IOS_COREML.md), which gates D51/B56's iOS 17.0+ Core ML compute-device-list snapshot and exact CoreML/Foundation import surface without loading a model or running inference.
G51 is [PLAN_VALIDATION_IOS_VISION.md](PLAN_VALIDATION_IOS_VISION.md), which gates D52/B57's portable revision-support value and iOS 13.0+ `VNRecognizeTextRequest.supportedRevisions` membership query without constructing a request or reading image data.
G52 is [PLAN_VALIDATION_IOS_SPEECH_STATUS.md](PLAN_VALIDATION_IOS_SPEECH_STATUS.md), which gates D53/B58's iOS 10.0+ Speech authorization-status snapshot without requesting permission or creating a recognizer.
G53 is [PLAN_VALIDATION_IOS_NATURALLANGUAGE_STATUS.md](PLAN_VALIDATION_IOS_NATURALLANGUAGE_STATUS.md), which gates D54/B59's iOS 17.0+ English contextual-model asset-status query without loading a model or accepting text.
G54 is [PLAN_VALIDATION_IOS_STOREKIT_STATUS.md](PLAN_VALIDATION_IOS_STOREKIT_STATUS.md), which gates D55/B60's deprecated legacy purchase-ability query without processing a purchase or transaction.
G55 is [PLAN_VALIDATION_IOS_ROOMPLAN.md](PLAN_VALIDATION_IOS_ROOMPLAN.md), which gates D56/B61's iOS 16.0+ `RoomCaptureSession.isSupported` query and compiler-verified Swift ABI thunk.
G56 is [PLAN_VALIDATION_IOS_CAMERA_DEVICE_STATUS.md](PLAN_VALIDATION_IOS_CAMERA_DEVICE_STATUS.md), which gates D57/B62's iOS 4.0+ default-video-device presence query without permission or capture.
G57 is [PLAN_VALIDATION_IOS_STOREKIT2_STATUS.md](PLAN_VALIDATION_IOS_STOREKIT2_STATUS.md), which gates D58/B63's iOS 15.0+ `AppStore.canMakePayments` weak-import Swift ABI call.
G58 is [PLAN_VALIDATION_IOS_SAFARI.md](PLAN_VALIDATION_IOS_SAFARI.md), which gates B64's typed `SFSafariViewController` constructor, HTTPS validation, device/Simulator compile and strict Clippy, and exact SafariServices link/import surface without presenting a view or starting a request.
G59 is [PLAN_VALIDATION_IOS_ACCELERATE.md](PLAN_VALIDATION_IOS_ACCELERATE.md), which gates D59/B65's iOS-only `vDSP_vadd` single-precision vector-add wrapper and exact Accelerate/libSystem release import surface without executing a numerical probe.
G60 is [PLAN_VALIDATION_IOS_CRYPTO.md](PLAN_VALIDATION_IOS_CRYPTO.md), which gates D60/B66's iOS-only CommonCrypto `CC_SHA256` wrapper, strict device/Simulator checks, and exact `libSystem.B.dylib`/`_CC_SHA256` release import surface without executing a probe or claiming digest parity.
G61 is [PLAN_VALIDATION_IOS_MODELIO_STATUS.md](PLAN_VALIDATION_IOS_MODELIO_STATUS.md), which gates D61/B67's iOS-only `MDLAsset.canImportFileExtension` query, its ModelIO/Foundation feature boundary, strict target Clippy, and exact device/Simulator link imports without loading an asset or executing a probe.
G62 is [PLAN_VALIDATION_IOS_MPS_STATUS.md](PLAN_VALIDATION_IOS_MPS_STATUS.md), which gates D62/B68's iOS-only default-option `MPSGetPreferredDevice` presence query, its MPSCore feature boundary, strict target Clippy, and exact device/Simulator link imports without submitting GPU work or executing a probe.
G63 is [PLAN_VALIDATION_IOS_KEY_SUPPORT.md](PLAN_VALIDATION_IOS_KEY_SUPPORT.md), which gates D63/B69's portable borrowed P-256 public-key contract and iOS Security algorithm-suitability query, including exact device/Simulator imports and deployment floors without verifying a signature or executing a probe.
G64 is [PLAN_VALIDATION_IOS_SPRITEKIT.md](PLAN_VALIDATION_IOS_SPRITEKIT.md), which gates D64/B70's finite portable SpriteKit position contract and detached iOS `SKNode.position` adapter, including strict device/Simulator Clippy, rustdoc, and exact link/import checks without scene rendering or probe execution.
G65 is [PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md](PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md), which gates D65/B71's iOS 9.3+ MediaPlayer library authorization snapshot, strict device/Simulator Clippy, rustdoc, and feature isolation without requesting access, reading media, or contacting Apple Music services.
G66 is [PLAN_VALIDATION_C_ABI_SPRITEKIT.md](PLAN_VALIDATION_C_ABI_SPRITEKIT.md), which gates F11's opt-in SpriteKit C ABI, C11/C++17 static links, feature isolation, symbol parity, strict host/device/Simulator checks, exact import allowlists, and deployment metadata without executing probes.
G67 is [PLAN_VALIDATION_IOS_CALL_OBSERVER.md](PLAN_VALIDATION_IOS_CALL_OBSERVER.md), which gates D76/B72's synchronous CallKit count/state snapshot, strict device/Simulator checks, rustdoc, feature isolation, and exact release imports without reading live call state.
G68 is [PLAN_VALIDATION_IOS_MAPKIT.md](PLAN_VALIDATION_IOS_MAPKIT.md), which gates D77/B73's portable finite map geometry and MapKit conversion/distance calls, strict device/Simulator checks, rustdoc, feature isolation, and exact release imports without executing probes.
G69 is [PLAN_VALIDATION_C_ABI_CALL_OBSERVER.md](PLAN_VALIDATION_C_ABI_CALL_OBSERVER.md), which gates F12's opt-in CallKit snapshot ABI, C11/C++17 links, feature isolation, symbol parity, strict host/device/Simulator checks, exact import allowlists, and deployment metadata without executing consumers.
G70 is [PLAN_VALIDATION_C_ABI_MAPS.md](PLAN_VALIDATION_C_ABI_MAPS.md), which gates F13's opt-in MapKit geometry ABI, C11/C++17 links, feature isolation, symbol parity, strict host/device/Simulator checks, exact MapKit/libSystem imports, and deployment metadata without executing consumers.
G71 records D79's WeatherKit feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); the native API is Swift-only and the REST route is not implemented as a typed, authenticated product contract.
G72 records D80's RealityKit feasibility audit in [PLAN_CAPABILITIES_REALITYKIT.md](PLAN_CAPABILITIES_REALITYKIT.md); the Objective-C-compatible ARView shell does not expose useful scene/entity operations.
G73 is [PLAN_VALIDATION_IOS_CLASSKIT.md](PLAN_VALIDATION_IOS_CLASSKIT.md), which gates D81/B74's borrowed `NSUserActivity.isClassKitDeepLink` query, strict device/Simulator checks, rustdoc, feature isolation, and exact release imports without executing probes.
G74 records D82's NetworkExtension feasibility audit in [PLAN_CAPABILITIES_NETWORK_EXTENSION.md](PLAN_CAPABILITIES_NETWORK_EXTENSION.md); B79 later implemented a read-only Personal VPN status slice, while provider and broader VPN APIs remain unsupported.
G75 is [PLAN_VALIDATION_C_ABI_CLASSKIT.md](PLAN_VALIDATION_C_ABI_CLASSKIT.md), which gates F14's opt-in caller-borrowed ClassKit marker C ABI, C11/C++17 links, feature isolation, symbol parity, strict host/device/Simulator checks, exact imports, selector and forbidden-data guards, and deployment metadata without executing consumers.
G76 records D83's PushToTalk feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); no standalone support query or channel/audio implementation exists.
G77 records D84's CarPlay feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); the public configuration values do not establish vehicle availability, and no entitled scene/session is implemented.
G78 is [PLAN_VALIDATION_IOS_FILEPROVIDER.md](PLAN_VALIDATION_IOS_FILEPROVIDER.md), which gates D85/B75's caller-app registered-domain presence snapshot; host/device/Simulator checks, strict Clippy, rustdoc, docs-check, and link/import audit passed, while probes were inspected but not executed.
G79 records D86's ExtensionKit/Foundation feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); the host-scoped ExtensionFoundation inventory is Swift-only, and no host-specific extension point, UI, process, or XPC implementation is claimed.
G80 records D87's ContactProvider feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); `isEnabled` is Swift-only and no Rust status facade or extension lifecycle is implemented.
G81 tracks F15's opt-in Location C ABI through [PLAN_BINDINGS_LOCATION.md](PLAN_BINDINGS_LOCATION.md); host/device/Simulator checks, strict Clippy, Release builds, C11/C++17 links, import/deployment audits, and static pointer-contract assertions passed; probes were inspected but not executed. B5's device and Simulator feature-tree assertions also passed with exactly `CLLocation`, `CLLocationManager`, and `CLLocationManagerDelegate`.
G82 records D88's BrowserEngineKit feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); generated Objective-C bindings exist, but no browser host, entitled extension set, or process/XPC lifecycle is implemented.
G83 records D89's ManagedApp/Distribution feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); no Swift-free generic managed-device status facade is available.
G84 records D90's MarketplaceKit feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); its async Swift-only installation-source and region queries do not establish distribution readiness or have a supported Rust async route.
G85 records D91's MatterSupport feasibility audit in [PLAN_CAPABILITIES_MATTERSUPPORT.md](PLAN_CAPABILITIES_MATTERSUPPORT.md); its Swift-only `MatterAddDeviceRequest.isSupported` query does not establish generic Matter or setup readiness.
G86 records D92's SecureElementCredential feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); its Swift-only eligibility query requires the entitlement it would need to assess and useful sessions require Apple approval and user consent.
G87 records D93's CarKey feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); no general Rust-callable CarKey support query exists, and the Swift session surface is restricted to approved automakers.
G88 records D94's ProximityReader feasibility audit in [PLAN_CAPABILITIES_PROXIMITYREADER.md](PLAN_CAPABILITIES_PROXIMITYREADER.md); its Swift-only device-model predicate does not establish Tap to Pay readiness.
G89 records D95's LockedCameraCapture feasibility audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); the SwiftUI/ExtensionKit flow has no general readiness query and depends on host and camera lifecycle.
G90 tracks F16's opt-in FileProvider C ABI through [PLAN_BINDINGS_FILEPROVIDER.md](PLAN_BINDINGS_FILEPROVIDER.md); its host/device/Simulator compile, Clippy, Release, C11/C++17 link, feature-isolation, import, and deployment gates passed; probes were not executed.
G91 records D96's extension-bundle metadata audit in [PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md](PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md); public Foundation/CoreFoundation readers exist, but no bounded helper or key schema is implemented.
G92 is [PLAN_VALIDATION_IOS_PROXIMITYREADER.md](PLAN_VALIDATION_IOS_PROXIMITYREADER.md), which gates B76's iOS 15.4+ `PaymentCardReader.isSupported` call, compiler-derived Swift ABI, target checks, and exact Release link imports without executing probes or claiming payment readiness.
G93 records D97's WidgetKit feasibility audit in [PLAN_CAPABILITIES_WIDGETKIT.md](PLAN_CAPABILITIES_WIDGETKIT.md); provider/timeline and SwiftUI interfaces lack a proven Rust route.
G94 records D98's ActivityKit ABI follow-up in [PLAN_CAPABILITIES_ACTIVITYKIT.md](PLAN_CAPABILITIES_ACTIVITYKIT.md); compiler signatures match, but owned-value linkage/runtime support remains unproven.
G95 records D99's App Intents audit in [PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md](PLAN_CAPABILITIES_CLOSED_FEASIBILITY.md); no documented stable metadata input, schema, or processor contract was found.
G96 records B77's bounded extension metadata reader in [PLAN_IOS_EXTENSION_SUPPORT.md](PLAN_IOS_EXTENSION_SUPPORT.md); host/device/Simulator compilation, strict Clippy, rustdoc, feature closure, and link/import gates passed, while probes were not executed. Row 113 is `B` for the one-key `.appex` metadata read only.
G97 records B78's Sign in with Apple credential-state package in [PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md](PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md); device/Simulator checks, strict Clippy, rustdoc, and AuthenticationServices import gates passed. Row 021 is `B` only for the conservative entitlement-scoped prior-user query; no query or linked probe was executed.
G98 records F19 in [PLAN_BINDINGS_COMPLETED_C_ABI.md](PLAN_BINDINGS_COMPLETED_C_ABI.md); its opt-in C ABI gate passed host/device/Simulator feature isolation, strict Clippy, C11/C++17 linking, exact CommonCrypto imports, deployment floors, and static pointer-order/ownership assertions without executing consumers or probes.
G99 records F20 in [PLAN_BINDINGS_COMPLETED_C_ABI.md](PLAN_BINDINGS_COMPLETED_C_ABI.md); its opt-in C ABI gate passed host/device/Simulator feature isolation, strict Clippy, C11/C++17 linking, exact ModelIO imports, selector/message-send checks, deployment-floor checks, and static output-memory validity/lifetime/range/overlap assertions without executing consumers or probes.
G100 records B79 in [PLAN_IOS_VPN_STATUS.md](PLAN_IOS_VPN_STATUS.md); host/portable and device/Simulator checks, strict Clippy, rustdoc, feature closure, and NetworkExtension/Foundation link/import gates passed. Device and Simulator probes were inspected but not executed; no entitled VPN query ran.
G101 records F21 in [PLAN_BINDINGS_COMPLETED_C_ABI.md](PLAN_BINDINGS_COMPLETED_C_ABI.md); the static C11/C++17 syntax/symbol gate and native arm64 device/Simulator C11/C++17 link/import/export/symbol/minos gate passed. Consumers and probes were not executed.
G102 records F22 in [PLAN_BINDINGS_COMPLETED_C_ABI.md](PLAN_BINDINGS_COMPLETED_C_ABI.md); the opt-in C ABI gate passed host/device/Simulator feature isolation, strict Clippy, C11/C++17 links, Accelerate/libSystem import checks, `_vDSP_vadd`, minos 10.0/14.0, and static pointer-bound/alias-order/ownership assertions. Consumers and probes were not executed.
G103 records B4 in [PLAN_IOS_NOTIFICATIONS.md](PLAN_IOS_NOTIFICATIONS.md); the UserNotifications package-local device/Simulator link/import gate passed exact framework-import and undefined-symbol denylist checks, and its compile-only probe selects `pending_request_count()`. The gate is wired in macOS CI, but no passing workflow run is recorded; no pending-request callback, live count, or linked probe was executed.
G104 records F23 in [PLAN_BINDINGS_F23.md](PLAN_BINDINGS_F23.md); static C11/C++17 syntax and symbol checks and the host/device/Simulator feature-isolation, strict-Clippy, Release-build, C11/C++17 link/import, export, forbidden-symbol, pointer-range/overlap, output-memory, and minos gates passed. Consumers and probes were not executed; the gate is wired in macOS CI, with no passing workflow run recorded.
G105 records F24 in [PLAN_BINDINGS_F24.md](PLAN_BINDINGS_F24.md); the opt-in MPS status C ABI passed host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact imports, required symbols, forbidden-symbol checks, output-pointer preconditions, export parity, and minos 12.2/14.0. Consumers and probes were not executed; both gates are wired in macOS CI, with no passing workflow run recorded.
G106 records F25 in [PLAN_BINDINGS_F25.md](PLAN_BINDINGS_F25.md); the opt-in VideoToolbox hardware-decode C ABI passed host/device/Simulator feature isolation, strict Clippy, Release archives, C11/C++17 links, exact VideoToolbox/libSystem imports, export parity, minos 11.0/14.0, and null-check/zero-write ordering assertions. Consumers and probes were not executed; both gates are wired in macOS CI, with no passing workflow run recorded.
G107 records F26 in [PLAN_BINDINGS_F26.md](PLAN_BINDINGS_F26.md); the opt-in default-video-device-presence C ABI passed host/device/Simulator feature isolation, strict Clippy, Release archives, C11/C++17 links, exact AVFoundation/libSystem/libobjc imports, export parity, and probe minos 10.0/14.0. The API floor is iOS 4.0; consumers and probes were not executed. Both gates are wired in macOS CI, with no passing workflow run recorded.
G108 records the current host integration gates: locked offline workspace check, strict all-target/all-feature workspace Clippy, and the 47-package no-std check passed on 2026-10-09. The host Clippy pass follows target-gating iOS-only link examples and imports; no tests or example binaries were executed.
G109 records F27 in [PLAN_BINDINGS_F27.md](PLAN_BINDINGS_F27.md); the opt-in Core ML compute-device-presence C ABI passed host/device/Simulator feature isolation, strict Clippy, rustdoc, Release archives, C11/C++17 links, exact CoreML/Foundation/libSystem/libobjc imports, export parity, and minos 11.0/14.0. Host C/C++ imported only libSystem; the C++ link used `-nostdlib++`. The API floor is iOS 17.0; consumers and probes were not executed. Both gates are wired in macOS CI, with no passing workflow run recorded.

G110 is [PLAN_VALIDATION_C_ABI_SPEECH_STATUS.md](PLAN_VALIDATION_C_ABI_SPEECH_STATUS.md), which gates F28's opt-in raw Speech authorization-status C ABI, host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact imports, export parity, and deployment metadata without executing consumers or probes. G111 gates B1's `ios-files` and `ios-preferences` Release link/import probes on device and Simulator; it checks exact per-crate imports and deployment metadata without executing the probes. G112 is [PLAN_VALIDATION_C_ABI_NATURAL_LANGUAGE.md](PLAN_VALIDATION_C_ABI_NATURAL_LANGUAGE.md), which gates F29's opt-in English NaturalLanguage asset-status C ABI, host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact imports, export parity, forbidden operations, and deployment metadata without executing consumers or probes. G113 gates B6's clipboard-only Release link/import/minos example on device and Simulator, with exact UIKit/Foundation imports and selector markers; it does not execute the probes. G114 is [PLAN_VALIDATION_C_ABI_EXTENSION_SUPPORT.md](PLAN_VALIDATION_C_ABI_EXTENSION_SUPPORT.md), which gates F30's caller-selected extension metadata C ABI, host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact imports, exports, forbidden extension-loading surfaces, and deployment metadata without executing consumers or probes. G115 is [PLAN_VALIDATION_C_ABI_ROOMPLAN.md](PLAN_VALIDATION_C_ABI_ROOMPLAN.md), which gates F31's opt-in RoomPlan support Boolean, host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 headers and links, exact RoomPlan/libSystem target imports, required RoomPlan symbols, and minos 16.0 without executing consumers or calling RoomPlan. G116 extends the B7 sharing validation plan with the share-only Release link/import/minos gate; it checks exact UIKit/Foundation/runtime imports and selector markers for device and Simulator without executing its probes.

G117 gates F32's opt-in StoreKit 2 purchase-ability C ABI with host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, weak-import checks, exact imports, export parity, and deployment metadata; consumers and probes are not executed. G118 gates F33's opt-in Game Center local-player status C ABI with host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact imports, export parity, and deployment metadata; consumers and probes are not executed.

## Status

As of 2026-10-09, G1–G11 evidence remains recorded in this plan and its linked workstream plans. G12 device/simulator check, strict Clippy, and import-audit CI wiring, the B18 crate, and the import script are present; its target checks and import script passed locally on Xcode 26.6 / SDK 26.5, while no passing CI workflow run is recorded. G13 device/Simulator check, strict Clippy, and import-script wiring and local gates are also complete on Xcode 26.6 / SDK 26.5; the B19 link/import probe was not executed, and no passing CI workflow run is recorded. G14's B20 package and script are present; device/Simulator checks, strict Clippy, and the corrected import/deployment audit passed locally on Xcode 26.6 / SDK 26.5. The probes were not executed, and no passing CI workflow run is recorded. G15's locked device/Simulator checks, strict all-target Clippy, and CoreGraphics/layout link/import script passed locally on Xcode 26.6 / SDK 26.5. The arm64 probes were not executed, and no passing CI workflow run is recorded. G16's locked device/Simulator checks, strict Clippy, and final CoreMedia/layout import script passed locally on Xcode 26.6 / SDK 26.5. The arm64 probes were not executed, and no passing CI workflow run is recorded. G17's shared `ios-ui` target checks and strict Clippy gates plus the text-metrics signature/layout and Release import script are wired in CI and passed locally on Xcode 26.6 / SDK 26.5; the linked probes were not executed. G18's `ios-connection` device/Simulator compile, strict Clippy, and focused link/import gates passed locally; G19's portable no-default check/Clippy, `ios-background-tasks` target checks, strict Clippy, and import gates also passed locally. G20 portable tests/check/Clippy/rustdoc and device/Simulator checks, strict Clippy, and ImageIO/CoreFoundation import/feature audit passed locally; G21 portable and iOS tests, checks, strict Clippy, rustdoc, API-floor fixture, and Photos/Foundation/libSystem/libobjc import audit passed locally. G22 portable no-default check/Clippy/rustdoc and device/Simulator compile/Clippy plus UIKit selector/import audits passed locally on Xcode 26.6 / SDK 26.5; the probes were not executed. G23 portable tests/no-default check and device/Simulator compile/strict Clippy, formatting, docs, and generated-feature review passed in an isolated worktree; no live prompt, contact data access, import audit, or passing CI run is claimed. G24 portable/host tests, no-default check, warning-denied rustdoc, device/Simulator checks and strict Clippy, docs check, and EventKit import probes passed in an isolated worktree; no live prompt or Calendar data access was exercised. G25 HealthKit and G26 Bluetooth portable and iOS target gates passed locally and are wired in CI; there is no passing CI run, and neither workstream claims a live permission or operation flow. G27–G31 and G33–G34 scoped gates passed locally; G32 is a feasibility audit only and has no implementation gate. G35 portable Metal tests/no-default check/Clippy/rustdoc and iOS device/Simulator checks, strict Clippy, exact Release import/symbol audit, docs check, and zero-Swift-source gate passed in the integrated checkout; probes were not executed, and no passing CI run is recorded. G36–G77 package gates, audits, and CI steps passed locally on Xcode 26.6 / iOS SDK 26.5; where scoped link probes were built, they were not executed, and no passing CI workflow run is recorded. The G3 parity and benchmark harnesses exist, but no real Apple reference/candidate suite or representative-device performance result is available. G2 archive evidence is unsigned and does not meet the Xcode 27.x baseline. For implemented iOS capability slices in G4–G68, target checks are compile/Clippy gates; G32 is feasibility-only and has no implementation gate. G64, G65, and G66 passed their integrated package gates; SpriteKit, MediaPlayer, and SpriteKit C ABI link probes were built and inspected, not executed. G67, G68, and G69 also passed their integrated gates; CallKit, MapKit, and CallKit C ABI probes were built and inspected, not executed. G70 passed its integrated C ABI gate; MapKit C/C++ probes were inspected, not executed. G71/G72 are feasibility-only WeatherKit and RealityKit audits; G73 passed the ClassKit marker package gate, and its probes were inspected, not executed. G74 is a feasibility-only NetworkExtension audit; no VPN query ran. G75 passed the opt-in ClassKit C ABI gate; its C/C++ consumers and probes were inspected, not executed. G76/G77 are feasibility-only PushToTalk and CarPlay audits; no service query or entitled host flow ran. The linked subplans retain each capability's runtime limits

G78, G81, G90, and G92 passed their focused non-test gates; G96–G100 cover B77, B78, F19, F20, and B79; G101–G102 cover F21 and F22. G103 covers B4’s UserNotifications device/Simulator link/import gate; it is wired in macOS CI, with no passing workflow run recorded. G104 covers F23’s P-256 Security suitability C ABI gate, wired in macOS CI; no passing workflow run is recorded. G105 covers F24’s MPS status C ABI gate, wired in macOS CI; no passing workflow run is recorded. G106 covers F25’s VideoToolbox hardware-decode C ABI gate, wired in macOS CI; no passing workflow run is recorded. G107 covers F26’s default-video-device-presence C ABI gate, wired in macOS CI; no passing workflow run is recorded. G108 records current locked workspace host check, strict all-target/all-feature Clippy, and the 47-package no-std check; no tests or example binaries ran. G109 covers F27’s Core ML C ABI host/device/Simulator link/import gate, wired in macOS CI; no passing workflow run is recorded. G79, G80, G82–G89, G91, and G93–G95 are feasibility-only audits. Rows 100–107 and 109–112 remain `X`; rows 021, 098, and 113 are `B` for B78's entitlement-scoped credential-state query, B79's Personal VPN profile-status query, and B77's one-key extension metadata read, respectively. Row 099 is `B` for registered-domain presence, and row 108 is `B` for the ProximityReader device-model predicate only. The matrix is 85/113 `B` (75.2% row coverage), 28 `X`, all with specific gaps. No tests or linked C/C++ consumers were executed.

G110 records F28's Speech authorization-status C ABI gates, wired in macOS CI. Host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact imports, export parity, and minos 10.0/14.0 passed locally; host imports were libSystem only, and target C++ links used `-nostdlib++`. No passing workflow run is recorded. No tests, linked consumers, or probes were executed. The capability matrix remains 85/113 `B` (75.2%) with 28 `X`.

G111 records B1's focused app-data Release link/import gate, wired in macOS CI. `ios-files` imports CoreFoundation, Foundation, `libSystem.B.dylib`, `libiconv.2.dylib`, and `libobjc.A.dylib`; `ios-preferences` imports Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. Probe minos is device 10.0 / Simulator 14.0. Target builds, strict Clippy, and link/import checks passed locally; probes were not executed. No passing CI workflow run is recorded.

G112 records F29's NaturalLanguage asset-status C ABI gates, wired in macOS CI. Host C/C++ import only `libSystem.B.dylib`; device/Simulator C/C++ imports are Foundation, NaturalLanguage, `libSystem.B.dylib`, and `libobjc.A.dylib`. C++ links use `-nostdlib++`; minos is 17.0 for device and Simulator. The API floor is iOS 17.0. Both gates passed in the integrated checkout; consumers and probes were not executed, and no passing CI workflow run is recorded.

G113 records B6's clipboard-only Release link/import/minos gate, wired in macOS CI. The probe selects only `clipboard`; device and Simulator imports are exactly Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`. Selector markers and minos 10.0/14.0 passed locally; probes were not executed, and no passing workflow run is recorded.

G114 records F30's extension metadata C ABI gates, wired in macOS CI. Host C/C++ import only `libSystem.B.dylib`; device/Simulator C/C++ import exactly Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos is 12.0/14.0. The backend API floor is iOS 4.0. Consumers and probes were not executed, and no passing CI workflow run is recorded.

G115 records F31's RoomPlan support C ABI static/build and link/import gates, wired in macOS CI. Host/device/Simulator feature isolation, Rust checks, strict Clippy, rustdoc, C11/C++17 header syntax and links, exact RoomPlan/libSystem target imports, required RoomPlan symbols, and minos 16.0 passed. Consumers and probes were not executed, no RoomPlan call ran, and no passing workflow run is recorded.

G116 records B7's share-only Release link/import/minos gate, wired in macOS CI. Device and Simulator probes import exactly CoreFoundation, Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`; expected UIKit selector markers, clipboard-feature exclusion, and Swift/Python runtime exclusions passed. `vtool` reports minos 10.0/14.0 with SDK 26.5. The gate was run locally, but probe artifacts were not executed; no live share UI behavior or passing CI workflow run is claimed.

G117 records F32's StoreKit 2 purchase-ability C ABI gates, wired in macOS CI. Host C/C++ imports only `libSystem.B.dylib`; device/Simulator C/C++ imports weak StoreKit and `libSystem.B.dylib`, with the StoreKit getter as a weak symbol. Link minos is 10.0/14.0; the API/symbol floor is iOS 15.0. Both static and link/import gates passed after root integration. No tests, consumers, or probes were executed; no passing CI workflow run is recorded.

G118 records F33's Game Center local-player status C ABI gates, wired in macOS CI. Host C/C++ imports only `libSystem.B.dylib`; device/Simulator imports Foundation, GameKit, `libSystem.B.dylib`, and `libobjc.A.dylib`, with minos 10.0/14.0. The SDK API floor is iOS 4.1 and the signed app requires `com.apple.developer.game-center`. Both static and link/import gates passed after root integration. No tests, auth flow, consumers, or probes were executed; no passing CI workflow run is recorded.

G119 records the post-`caab477` integration gates on Xcode 26.6 build 17F113 / iOS SDK 26.5: `cargo +1.94.1 fmt --all -- --check`, locked offline workspace check, strict workspace Clippy, `xtask docs-check`, `xtask zero-swift-source`, staged diff check, and capability-manifest consistency all passed. The manifest has 113 rows: 89 `B` (78.8%) and 24 `X`. This is row coverage, not whole-plan completion. No tests, probes, linked consumers, or runtime calls ran. At that checkpoint, the B228 attributed user-input-label getter was still a draft; B228 later passed its focused gates (G124). B233, B236, B239, B235, and B242 also postdate that checkpoint.

G120 wires B209 ActivityKit validation in macOS CI through `sh platform/ios/ios-activitykit-status/check.sh`. The focused package script includes Rust checks, strict Clippy, rustdoc, compiler-oracle, and link/import gates; its probes are inspected, not executed. Local gates passed before `caab477`; no CI workflow run is recorded.

G121 wires B233 MatterSupport request-support checks in macOS CI through `sh platform/ios/ios-matter-support-status/check.sh`. The focused format, host/device/Simulator check, strict Clippy, rustdoc, docs-check, compiler ABI, weak-import, and link/import gates passed locally. No request, setup flow, app, probe, test, or runtime call ran; no CI workflow run is recorded.

G122 wires B236 Foundation Models default-model readiness checks in macOS CI through `sh platform/ios/ios-foundation-models-status/check.sh`. Host/device/Simulator Rust gates, strict Clippy, rustdoc, format, docs-check, compiler ABI, and static FoundationModels link/import gates passed at min iOS 26.0. Apple marks the API documentation beta; the static libraries were not executed, and no model call, test, or runtime parity check ran. No CI workflow run is recorded.

G123 wires B239 AdAttributionKit app-impression support checks in macOS CI through `sh platform/ios/ios-ad-attribution-status/check.sh`. Host/device/Simulator checks, strict Clippy, rustdoc, format, docs-check, compiler ABI, weak-import, and static link/import gates passed. No impression, token generation, network call, test, or runtime query ran; no CI workflow run is recorded.

G124 records B228/B234/B243/B246/B249/B252/B255 accessibility getters. Locked offline device/Simulator checks, strict Clippy on both targets, device rustdoc, format, docs-check, and scoped diff checks passed. The operations preserve native nil or raw values as documented, use main-thread proof for B249/B252/B255, preserve the Guided Access gate for AssistiveTouch, and invoke no callback; no tests, probes, or runtime UIKit calls ran.

G125 records B235, B242, B244, B248, and B251 app-data metadata gates: device/arm64 Simulator checks, strict Clippy, rustdoc, format, docs-check, and scoped diff checks passed. B235 reports allocated bytes of the directory object; B242 reports logical total size across a regular file's forks; B244 reports data-fork allocated bytes only and excludes resource-fork allocation; B248 reports resource-fork allocated bytes only; B251 reports its logical length only. Neither reads fork contents or infers absence from zero. B237/B238/B240/B241/B247/B253/B256/B257/B259/B260/B262/B263/B264/B265 are source audits only. No tests, probes, consumer, live filesystem query, or runtime operation ran.

G126 records the integrated aggregate update after B233, B236, and B239: 92 of 113 capability rows are partial (`B`) (81.4%), and 21 remain `X`. The capability manifest and human-readable summaries agree, the three focused package gates are wired in macOS CI, and local `docs-check`, manifest consistency, CI YAML parse, zero-Swift-source, formatting, locked workspace check, strict Clippy, rustdoc, and diff checks passed on Xcode 26.6 build 17F113 / iOS SDK 26.5. This is row coverage, not whole-plan completion. No tests, linked consumer, probe execution, runtime API call, parity suite, performance run, signed archive, or Xcode 27.x validation is claimed; no CI workflow run is recorded.

G127 records B245 DockKit system-tracking status, wired in macOS CI through `sh platform/ios/ios-dockkit-status/check.sh`. Host/device/Simulator checks, strict Clippy, rustdoc, format, docs-check, ABI oracle, and static physical-device link/import gates passed. The Simulator SDK lacks the framework/module and returns `NativeApiUnavailable`; no accessory/runtime call or test ran. The current manifest has 93/113 partial rows (82.3%) and 20 `X`; aggregate formatting, manifest consistency, docs, CI YAML, workspace checks, strict Clippy, rustdoc, zero-Swift-source, and diff checks are re-run under G131.

G128 records B249 and B252 accessibility running-state snapshots. Both require `ios_runtime::MainThread`, return the current UIKit Boolean, and add no observer or UI action. Locked offline device/Simulator checks, strict Clippy, rustdoc, format, docs-check, and scoped diff checks passed; no test, probe, or runtime query ran.

G129 records B250 WeatherKit REST feasibility. Apple documents a coordinate-specific availability endpoint, but the result is data-set availability only. A useful host-selected REST client still requires an ES256 developer-token boundary, typed response scope, and attribution; no token, network call, build, or runtime query ran, and row 091 remains `X`.

G130 records B254 WidgetKit timeline-reload request, wired in macOS CI through `sh platform/ios/ios-widgetkit-reload/check.sh`. Host/device/Simulator checks, strict Clippy, rustdoc, format, docs-check, ABI oracles, and static link/import gates passed. The operation only requests a reload of the containing app's configured widgets; no provider, render, runtime, app, or test ran. Row 111 is partial (`B`); the matrix now has 94/113 (`B`, 83.2%) and 19 `X`.

The integrated F25–F29 static/build gates also assert output-pointer contract wording across the
owned source, header, guide, plan, and manifest where applicable: caller-owned valid, aligned,
writable storage for the full synchronous call, zero initialization before platform handling,
caller protection from unsynchronized access, nullness-only checks, and no pointer retention.
These assertions document preconditions; they do not prove the validity of arbitrary C memory.

`cargo +1.94.1 xtask no-std-link-probe` passed on Xcode 26.6 (build 17F113) with iPhoneOS and iPhoneSimulator SDK 26.5. The x86_64 macOS host artifact linked and ran; separate arm64 iOS device and arm64 iOS Simulator probe dylibs linked and passed archive, linker-map, architecture, platform, symbol, and import audits. The Apple artifacts were not executed. The detailed no_std section below records the bounded API coverage and remaining proof limits

The latest `cargo +1.94.1 xtask no-std-check` passed on 2026-10-09 for all 47 current `PORTABLE_CRATES` entries. No link probe was run in this refresh; the last recorded link-probe report below covers 45 packages

## Integrated workspace audit (2026-10-08)

The integrated checkout passed:

- `cargo +1.94.1 fmt --all -- --check` after applying rustfmt to two `framework-abi` conditionals
- `cargo +1.94.1 check --workspace --locked`
- `cargo +1.94.1 clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo +1.94.1 test --workspace --locked`, including unit and doctest targets
- `cargo +1.94.1 xtask no-std-check`
- `cargo +1.94.1 xtask no-std-link-probe`
- `cargo +1.94.1 xtask codegen-audit` for four installed targets; structural `OperationId` evidence only, not a performance result
- `cargo +1.94.1 xtask docs-check`, `zero-swift-source`, dependency audit, and ABI source audit
- `cargo +1.94.1 doc --workspace --no-deps`
- `cargo +1.94.1 tree --workspace --locked --depth 1` and `cargo +1.94.1 tree -d --locked`; no duplicate versions were reported
- `cargo +1.94.1 xtask archive-smoke` and `cargo +1.94.1 xtask ios-build --simulator --release`
- `cargo xtask linkage-audit --binary target/ios-minimal/archive/ios-minimal.xcarchive/Products/Applications/ios-minimal.app/ios-minimal`
- `git diff --check`

The archive and simulator app build use Xcode 26.6 build 17F113 / iOS SDK 26.5, below the planned Xcode 27.x baseline. The archive is unsigned. The archive and link reports show no Swift/Python runtime imports in the minimal app; these gates do not establish arm64 simulator launch, physical-device execution, or live behavior for the capability backends. The separate x86_64 simulator launch and programmatic callback evidence remains limited as stated in `PLAN_IOS_NATIVE.md`

## Write scope

- `tools/xtask/**`
- `tools/sdk-inventory/**`
- `tools/swift-oracle/**`
- `tools/linkage-audit/**`
- `tools/abi-audit/**`
- `tools/parity-harness/**` shared harness
- `tests/abi/**`
- `tests/linkage/**`
- `tools/bench-harness/**` shared harness
- CI workflows
- global support/generated reports
- global docs indexes

## Toolchain manifest

Command should capture:
- macOS version/arch;
- Xcode path/version/build;
- Swift compiler version;
- clang/LLVM version;
- Rust toolchain;
- iphoneos and simulator SDK paths/versions;
- target triples/deployment versions.

Store reproducibility manifest as build/test artifact, not necessarily a committed machine-specific file.

## no_std enforcement

CI must build every portable crate:
- with default features appropriate to portable use;
- with `--no-default-features`;
- without linking `std`.

`cargo xtask no-std-check` runs `cargo check` for each portable crate. It does not link an artifact or prove no linked `std`; `no-std-link-probe` supplies bounded host, device, and simulator link evidence for the probe artifact, not every framework code path

`no-std-link-probe` selects every package in the current 47-entry `PORTABLE_CRATES` registry and is designed to build a transient `#![no_std]` static library with `--no-default-features`, a `#[panic_handler]`, `panic=abort`, and an explicit C `malloc`/`free` global allocator. Its C `main` invokes one selected public API check per crate, calls the ABI destructor symbol, and requests/frees one aligned allocation. The macOS host artifact is linked, audited, and run. For `aarch64-apple-ios` and `aarch64-apple-ios-sim`, the static library is linked with a C harness into arm64 iOS device and Simulator dylibs; these Apple artifacts are audited but not executed. All three archives and linker maps are checked for Rust `std` members, and the linked Mach-O artifacts are checked for architecture/platform metadata, required symbols, unresolved Rust symbols, and their dynamic import allowlists. The latest recorded successful report remains the 45-package run from 2026-10-08; no 47-package link evidence is claimed

The Apple link commands use the matching Xcode SDK and explicit `-Wl,-undefined,error`; they do not use `dynamic_lookup` or suppress unresolved symbols. The host and Apple target archives have `rust_eh_personality` references from compiler-built Rust code, so the generated host/target C fallback with the same symbol calls `abort()`. The fixture uses `panic=abort` and its `#[panic_handler]` also aborts; the fallback is a fail-fast guard, not a Rust unwinding personality implementation. Any call into it terminates rather than attempting unwind. The target `nm -u` audit reports only symbols imported from the sole allowed `libSystem.B.dylib` dependency and rejects unresolved Rust-mangled symbols or `rust_eh_personality`

The `ios-ui` package under `platform/ios/ios-ui` is a platform crate and stays outside this portable list

The host and both Apple target maps have separate archive-object evidence for 13 crates. Thirty-one API-only crates, including HDR playback eligibility, Bluetooth discovery, HealthKit authorization, cloud identity, web URL/navigation contracts, Game Center local-player status, ARKit world-tracking support, and Vision text-recognition revision support, have invoked checks in the Rust probe object without separate crate-object attribution; `framework-platform` is checked at compile time. This is selected-API link evidence, not full per-crate archive-object proof; the report keeps `all_portable_crates_no_std_linkage_proven` false. The last recorded 24-package run passed on 2026-10-08 with Rust 1.94.1, Xcode 26.6 (build 17F113), and iPhoneOS/iPhoneSimulator SDK 26.5. D21–D34 added twelve portable crates; the next probe covered 36 packages. The 45-package `no-std-check` and selected-API host/device/Simulator `no-std-link-probe` passed locally on 2026-10-08; the latest 47-package `no-std-check` passed on 2026-10-09 with Rust 1.94.1. The link probe was not rerun, so recorded link evidence remains 45 packages. The Apple outputs are `target/xtask/no-std-link-probe/no-std-link-probe-ios-device.dylib` (iOS minimum 12.0) and `target/xtask/no-std-link-probe/no-std-link-probe-ios-sim.dylib` (Simulator minimum 14.0); both are arm64 and import only `/usr/lib/libSystem.B.dylib`. Their `nm -u` output includes `__Unwind_Resume` and common C/runtime imports resolved by that dylib, but no unresolved Rust-mangled symbol or `rust_eh_personality`. This does not establish execution, panic behavior, or runtime behavior on either target. The explicit allocator path requests and frees one aligned block through `alloc::alloc`; allocation-backed collection APIs use empty values, so nonempty collection allocation is not exercised. A direct `cargo +1.94.1 rustc --locked -p framework-core -- --crate-type staticlib` probe had failed with `#[panic_handler] function required, but not found` and `unwinding panics are not supported without std`; `otool -L` alone cannot see static Rust `std`

`cargo xtask no-std-check` classifies direct dependency edges for each portable crate by querying separate normal, dev, and build Cargo trees with all features and targets. It rejects a direct package ID that appears in both the normal and dev/build trees, and it rejects any internal workspace package under `platform/` from the full normal dependency tree

The last pre-expansion gate passed for all 24 portable crates on 2026-10-08; D21–D34 added twelve portable crates, and eight more have since joined the probe. The 45-package `no-std-check` and selected-API host/device/Simulator link probe passed locally on 2026-10-08; the latest `no-std-check` passed for all 47 current registry entries on 2026-10-09, while the link probe remains evidenced for 45. Each of the 24 earlier crates had zero direct dev and build edges. The link report still records `all_portable_crates_no_std_linkage_proven: false` because 31 invoked APIs have no separate archive-object attribution. The gate verifies direct portable-crate dependency kinds, not third-party semantic intent or every transitive target/proc-macro/build-script role; dependency-purpose review and full transitive `std` proof remain separate

## Dependency audit

Track:
- direct deps;
- transitive deps;
- duplicate versions;
- enabled features;
- proc-macro/build deps;
- std requirements;
- linked native/system libs.

Fail or require explicit approval when a narrow capability adds disproportionate graph growth.

Do not optimize for dependency count at the expense of security/correctness.

## SDK capability inventory

Generate machine-readable inventory from installed SDK:
- frameworks/modules;
- Objective-C headers;
- public C symbols/headers;
- `.swiftinterface` declarations;
- availability;
- `@objc` exposure;
- async/throws;
- generic signatures;
- actor isolation;
- relevant protocol conformances.

Use it to detect SDK drift and update the human support matrix.

Do not treat generated inventory as a replacement for public API/compliance review.

## Swift oracle harness

For selected Swift-only public APIs:
- generate tiny temporary Swift reference call;
- emit SIL;
- emit LLVM IR;
- compile optimized object;
- inspect undefined/defined symbols;
- inspect assembly;
- compare with framework thunk.

Repository/shipping source remains Swift-free; oracle source is test/tool input only.

## Parity harness

Shared differential API:
```text
fixture/generator
 -> Apple reference adapter
 -> Rust candidate
 -> comparator
 -> regression fixture on mismatch
```

Support:
- fixed fixtures;
- property/generated inputs;
- documented nondeterminism normalization;
- OS-version expected differences;
- error comparison;
- serialization compatibility where claimed.

## ABI audit

Check:
- C ABI layout;
- symbol export list;
- thunk calling convention;
- Swift runtime symbol provenance/availability;
- panic containment;
- ownership creator/destroyer pairs.

## Linkage audit

For minimal examples:
- Mach-O imported dylibs/frameworks;
- Swift runtime linkage when not requested;
- Python absence;
- unrelated capability absence;
- binary size.

Required examples:
- portable core;
- preferences;
- secure storage;
- network;
- location;
- UI;
- StoreKit/Translation Swift residual when enabled.

## Performance harness

Authoritative iOS performance claims require physical-device Release measurements where hardware/system services matter.

Harness supports:
- latency distributions;
- CPU;
- allocations;
- copies;
- RSS;
- hot working set;
- code size;
- startup;
- energy where available.

CI microbenchmarks are advisory; deterministic structural gates may fail CI.

## Codegen audit

The first bounded audit targets `framework_core::OperationId` with `cargo xtask codegen-audit`:
- build a transient `#![no_std]` Rust-ABI probe with no `extern "C"` declarations at Release `opt-level = 3`;
- emit optimized LLVM IR for the host and each installed target whose triple contains `apple-ios`;
- require `OperationId::get` and `OperationId::new` followed by `get` to lower to an identity `i64` return;
- require both `OperationId` and `Option<OperationId>` to lower to an `i64` constant size of 8;
- reject any call instruction, stack or heap allocation instruction, or atomic/lock instruction in those probe functions;
- write the IR and a machine-readable report to `target/xtask/codegen-audit/` and `target/xtask/codegen-audit.json`.

This is structural compiler evidence for the selected wrapper only. It is not a benchmark, a full-program LTO or linked-binary audit, an FFI ABI proof, or a runtime/performance claim. LLVM/Rust version changes can change IR and require review; additional wrappers and hot kernels need separate named probes before any broader claim.

## Archive smoke

Current G2 evidence: `cargo xtask archive-smoke` always sets `CODE_SIGNING_ALLOWED=NO` and checks archive/app plist syntax, Swift/Python runtime import absence, `.swift` source absence, and unsigned status; it does not assess entitlements, app-profile data, signed archives, export, install, launch, or device use

Using Xcode public tooling:
- build Release device artifact;
- bundle/link;
- sign where credentials/environment permit;
- archive;
- verify no `.swift` shipping source;
- verify framework/runtime linkage;
- inspect entitlements/Info.plist expectations.

App Store upload itself is not required for every CI run.

## CI matrix

At minimum:
- format;
- clippy;
- unit tests;
- no_std checks;
- a bounded macOS host plus arm64 iOS device and Simulator link probe for all registered portable crates;
- simulator build;
- C header compile;
- dependency/linkage audit;
- docs;
- zero-Swift-source check.

Periodic/manual:
- device integration;
- physical benchmarks;
- archive/signing;
- entitlement-specific capabilities.

## G1–G131 gate audit (2026-10-09)

- `.github/workflows/ci.yml` configures host formatting, Clippy, workspace tests, portable default/no-default checks, the macOS host and Apple-target `no-std-link-probe`, dependency and ABI inventories, rustdoc, documentation/zero-Swift checks, C/C++ consumers, the minimal Release app build/link checks, and device/simulator check plus strict Clippy gates for integrated iOS packages. G12's device/simulator target commands and focused import-script invocation are wired; the corresponding local gates passed, but no CI workflow run is recorded. G13's device/simulator target commands and import-script invocation are wired and passed locally; no CI workflow run is recorded. G14's target commands and corrected import-script invocation are wired and passed locally; no CI workflow run is recorded. G15's device/Simulator check, strict Clippy, and CoreGraphics/layout import script are wired and passed locally; no CI workflow run is recorded. G16's locked target checks, strict Clippy, and CoreMedia/layout import script are wired and passed locally; no CI workflow run is recorded
- G2 archive evidence and its unsigned/signing, Xcode-baseline, simulator, and physical-device limits are recorded in `PLAN_VALIDATION_ARCHIVE.md`
- G3 fixture tests validate the harness APIs only; no real Apple reference adapter, Rust candidate suite, framework workload, parity result, or representative-device performance result exists, and `cargo xtask parity` remains unavailable
- G22 and G23 target checks, strict Clippy, and their package scripts are wired in `.github/workflows/ci.yml`; their local scoped gates passed, but no passing CI workflow run is recorded. G22 does not exercise a live UIKit expiry callback; G23 does not prompt or access contact data
- G24 target checks, strict Clippy, package tests/docs/import probes, and its package script are wired in `.github/workflows/ci.yml`; local gates passed in the isolated worktree, but no passing CI workflow run is recorded. No live Calendar prompt or data access was exercised
- G25 portable HealthKit tests/no-default check/Clippy/rustdoc and iOS device/Simulator checks/strict Clippy passed in the isolated worktree and again after root integration; no app link, entitlement validation, or live permission flow is claimed. CI target gates are wired; no passing CI run is recorded
- G26 portable Bluetooth tests/no-default check, iOS device/Simulator compile and strict Clippy passed in the isolated worktree and root target script; these gates do not initialize a manager or exercise authorization/radio behavior. CI gates are wired; no passing CI run is recorded
- G27 portable `framework-web` tests/no-default check/strict Clippy, iOS device/Simulator checks/strict Clippy, rustdoc, and the forbidden-surface audit passed in its isolated worktree and root checkout; no page, UI, or network action was run. Root CI gates are wired; no passing CI run is recorded
- G28 Bluetooth discovery portable tests/no-default check, device/Simulator checks/strict Clippy, rustdoc, and docs gates passed in the isolated worktree; after integration the root package-local target script also passed. The bounded queue tests are host-only; no prompt, manager, scan, radio, peer, or background behavior was exercised. CI gates are wired; no passing CI run is recorded
- G29 iCloud identity portable test/no-default check, device/Simulator checks/strict Clippy, formatting, diff, and docs gates passed in an isolated validation copy and the root package script; only token presence is exposed, with no live account query or entitlement validation. Root CI gates are wired; no passing CI run is recorded
- G30 camera/microphone authorization portable and iOS target gates passed in the isolated worktree; strict Clippy, rustdoc, and the status-only surface guard also passed. No permission query, prompt, device access, or capture ran; CI calls the package gate script, but no workflow run is recorded
- G31 NFC portable and iOS device/Simulator check and strict-Clippy gates passed in the isolated worktree; no session, prompt, scan, tag operation, or live hardware result was exercised. Root CI target gates are wired; no passing CI run is recorded
- G32 HomeKit feasibility review found no prompt-free status-only query under the scoped constraints. No implementation gate ran and row 072 remains `X`; see the HomeKit status reason in `docs/capabilities/capability-status.json`
- G33 Nearby Interaction portable tests/no-default check/strict Clippy and iOS device/Simulator check/strict Clippy passed in the integrated checkout. The query reads only the precise-distance device-capability field; no session, permission, token, peer, or ranging operation ran. Root CI calls its package gate script; no passing CI run is recorded
- G34 App Tracking Transparency status portable tests/no-default check/strict Clippy, iOS device/Simulator compile and strict Clippy, and `sh platform/ios/ios-auth/check-link-imports.sh` passed in the integrated checkout. The import probe now links AppTrackingTransparency alongside the existing Foundation and LocalAuthentication imports. `objc2-app-tracking-transparency` 0.3.2 uses default features off; its optional `block2` request API is not enabled, while `ios-auth` still uses `block2` for the separate LocalAuthentication backend. No live status read, prompt, user choice, identifier access, or tracking operation ran. Root CI calls both package and import gates; no passing CI run is recorded
- G41 D42/B47 other-audio portable no-default check/strict Clippy, iOS device/Simulator check/strict Clippy, rustdoc, dependency-feature audit, and exact Release import/symbol gate passed through `sh platform/ios/ios-media/check-audio-playback.sh` on Xcode 26.6 / iOS SDK 26.5. Probes import AVFoundation, CoreFoundation, CoreMedia, Foundation, libobjc, and libSystem; minos is device 12.0 and Simulator 14.0. Probes were not executed, no live audio query or CI workflow run is claimed
- G42 D43/B48 portable no-default check, strict Clippy, iOS device/Simulator and host checks, strict Clippy, rustdoc, and Release import/minimum-OS audit passed through `sh platform/ios/ios-playback/check.sh`. Probes import exactly AVFoundation, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos is device 13.4 and Simulator 14.0. Probes were linked but not executed. The package gate is wired in macOS CI; no passing CI workflow run or live HDR behavior is claimed
- G43 D44/B49 MessageUI and SharedWithYou device/Simulator checks, strict Clippy, rustdoc, exact import audits, source guards, docs, zero-Swift, and diff gates passed through both package scripts. The probes were linked but not executed; no live status, message composition, account, highlight, or collaboration behavior is claimed. Package scripts are wired in macOS CI; no passing CI workflow run is recorded
- G44 D46/B51 `framework-payments` host/device/Simulator checks, strict Clippy, rustdoc, source guard, and exact Release import audit passed through `sh crates/framework-payments/scripts/check.sh`. Probes import Foundation, PassKit, `libSystem.B.dylib`, and `libobjc.A.dylib`; they were built but not executed. CI wiring is present; no live Apple Pay status or transaction behavior and no CI workflow run are claimed
- G45 D45/B50 `framework-media` portable no-default/Clippy/rustdoc plus `ios-media` host/device/Simulator checks, strict Clippy, rustdoc, and binding-feature guard passed through `sh platform/ios/ios-media/check.sh`. No tests, link probes, live codec query, or device behavior were run. The gate is wired in CI; no passing CI workflow run is recorded
- G46 D47/B52 `framework-cloud` no-default check/strict Clippy and `ios-cloud` device/Simulator check/strict Clippy passed locally. Both link audits import CloudKit, Foundation, libSystem, and libobjc with no Swift runtime or unrelated capability frameworks. Probes were built, not executed; no live account query or CI workflow run is claimed. See `PLAN_VALIDATION_IOS_CLOUDKIT_ACCOUNT_STATUS.md`
- G47 D48/B53 `ios-safety` host/device/Simulator checks, strict Clippy, rustdoc, and SafetyKit/Foundation/libSystem/libobjc import audits passed in the isolated worktree. Link probes were built, not executed; the getter-specific entitlement requirement, live device bit, and event behavior remain unverified. See `PLAN_VALIDATION_IOS_SAFETYKIT.md`
- G48 D49/B54 `sh platform/ios/ios-maps/check.sh` passed in the integrated checkout after its import allowlist was corrected from the probe evidence. Portable no-default check/Clippy/docs, host/device/Simulator check/Clippy, feature review, device/Simulator Release link audits, exact ARKit/Foundation/libSystem/libobjc imports, selector/symbol/string filters, and minos 12.0/14.0 passed. Probes were not executed; no live support query, camera use, or runtime tracking is claimed. See `PLAN_VALIDATION_IOS_ARKIT.md`
- G49 D50/B55 `framework-game` no-default check/strict Clippy and `ios-game` device/Simulator check/strict Clippy/import audits passed in the isolated worktree. Link probes were built, not executed; no signed entitlement, live authentication state, or UI behavior is claimed. See `PLAN_VALIDATION_IOS_GAMEKIT.md`
- G50 D51/B56 `sh platform/ios/ios-core-ml-status/scripts/check.sh` passed in the integrated checkout. Host/device/Simulator check, strict Clippy, rustdoc, and exact CoreML/Foundation/libSystem/libobjc link/import/symbol audits passed. Probes were built, not executed; no live hardware list, model compatibility, or prediction behavior is claimed. See `PLAN_VALIDATION_IOS_COREML.md`
- G51 D52/B57 `sh platform/ios/ios-vision/check.sh` passed in the integrated checkout. Portable no-default check/strict Clippy/docs and iOS host/device/Simulator checks, strict Clippy, feature gates, exact Vision/Foundation/libobjc/libSystem imports, symbol/string filters, and minos 13.0/14.0 passed. Probes were built, not executed; no image, recognition, or live revision query is claimed. See `PLAN_VALIDATION_IOS_VISION.md`
- G52 D53/B58 `sh platform/ios/ios-speech-status/scripts/check.sh` passed in the integrated checkout. Host/device/Simulator checks, strict Clippy, rustdoc, the API-surface gate, exact Speech/Foundation/libobjc/libSystem imports, and docs/zero-Swift gates passed. Probes were built, not executed; no live authorization, prompt, audio, or recognition behavior is claimed. See `PLAN_VALIDATION_IOS_SPEECH_STATUS.md`
- G53 D54/B59 `sh platform/ios/ios-natural-language-status/scripts/check.sh` passed in the integrated checkout. Host/device/Simulator checks, strict Clippy, rustdoc, the API-surface gate, exact NaturalLanguage/Foundation/libobjc/libSystem imports, and docs/zero-Swift gates passed. Probes were built, not executed; no live asset state, model load, or vector operation is claimed. See `PLAN_VALIDATION_IOS_NATURALLANGUAGE_STATUS.md`
- G54 D55/B60 `sh platform/ios/ios-storekit-status/scripts/check.sh` passed in the integrated checkout. Host/device/Simulator checks, strict Clippy, rustdoc, source/feature guards, exact Foundation/StoreKit/libobjc/libSystem imports, and docs/zero-Swift gates passed. Probes were built, not executed; no live purchase ability, product availability, account state, or transaction behavior is claimed. See `PLAN_VALIDATION_IOS_STOREKIT_STATUS.md`
- G55 D56/B61 `sh platform/ios/ios-roomplan/check.sh` passed in both the isolated worktree and integrated checkout. Portable `no_std`, host/device/arm64 Simulator checks, strict Clippy, rustdoc, the compiler-oracle `swiftcall` audit including `swift_context`/`swiftself`, and exact RoomPlan/libSystem Release imports passed. Probes were built and inspected, not executed; no live query or scan is claimed. See `PLAN_VALIDATION_IOS_ROOMPLAN.md`
- G56 D57/B62 `sh platform/ios/ios-camera-device-status/check.sh` passed in the isolated worktree and integrated checkout. Host/device/Simulator checks, strict Clippy, rustdoc, source-surface guard, exact AVFoundation/Foundation/libobjc/libSystem imports, docs, zero-Swift, and diff gates passed. Probes were built, not executed; no live camera, authorization, or capture behavior is claimed. See `PLAN_VALIDATION_IOS_CAMERA_DEVICE_STATUS.md`
- G57 D58/B63 `sh platform/ios/ios-storekit2-status/check.sh` passed in the isolated worktree and integrated checkout. Host/device/Simulator checks, strict Clippy, rustdoc, transient Swift/Clang IR comparison, SDK symbol check, exact weak StoreKit/libSystem imports, docs, zero-Swift, and diff gates passed. Probes were not executed; no live payment query or purchase behavior is claimed. See `PLAN_VALIDATION_IOS_STOREKIT2_STATUS.md`
- G58 B64 SafariServices device/Simulator checks and strict Clippy, rustdoc, exact Foundation/SafariServices/UIKit/libSystem/libobjc Release imports, class/selector metadata, Objective-C lookup/message imports, iOS deployment floors, formatting, docs, zero-Swift, and diff gates passed in the isolated worktree and integrated checkout. Link probes were inspected, not executed; no controller presentation, URL request, page load, Safari routing, or browser parity is claimed. See `PLAN_VALIDATION_IOS_SAFARI.md`
- G59 D59/B65 iOS device/Simulator checks, strict Clippy, rustdoc, exact Accelerate/libSystem Release imports, `_vDSP_vadd`, deployment floors, formatting, docs, zero-Swift, and diff gates passed in the isolated validation workspace and integrated checkout. Probes were inspected, not executed; no numerical result, parity, live-device behavior, or performance claim is made. See `PLAN_VALIDATION_IOS_ACCELERATE.md`
- G60 D60/B66 locked iOS device/Simulator checks, strict Clippy, rustdoc, exact `libSystem.B.dylib` Release import, `_CC_SHA256`, iOS 10.0/device and iOS 14.0/Simulator link floors, formatting, docs, zero-Swift, and diff gates passed in the integrated checkout. Probes were inspected, not executed; no digest parity, security review, certification, live-device behavior, or performance claim is made. See `PLAN_VALIDATION_IOS_CRYPTO.md`
- G61 D61/B67's locked package gate passed after root integration: format, host and Apple target checks, strict Clippy, `MDLAsset` feature isolation, device/Simulator Release link/import probes, and iOS rustdoc. Exact imports were Foundation, ModelIO, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos was 10.0 device / 14.0 Simulator. Probes were inspected, not executed; no asset parsing, runtime support, or parity is claimed. See `PLAN_VALIDATION_IOS_MODELIO_STATUS.md`
- G62 D62/B68 package format, host/device/Simulator checks, strict Clippy, rustdoc, and MPSCore feature isolation passed; the exact device/Simulator Release import and deployment-floor audit also passed. Imports were Foundation, Metal, MetalPerformanceShaders, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos was 12.2 device / 14.0 Simulator. Probes were inspected, not executed; no GPU work, operation support, live behavior, or performance is claimed. See `PLAN_VALIDATION_IOS_MPS_STATUS.md`
- G63 D63/B69 portable no-default check, strict Clippy, rustdoc, iOS device/Simulator checks and strict all-target Clippy passed. Release probes imported CoreFoundation, Security, and `libSystem.B.dylib`, with `SecKeyCreateWithData` and `SecKeyIsAlgorithmSupported`; minos was 10.0 device / 14.0 Simulator. Probes were inspected, not executed; no live key query, signature verification, persistence, or Secure Enclave behavior is claimed. See `PLAN_VALIDATION_IOS_KEY_SUPPORT.md`
- After D55/B60 root integration, locked workspace `cargo check`, strict all-target/all-feature Clippy, `xtask docs-check`, formatting, `git diff --check`, and dependency audit passed. No tests were run in this pass.
- After D57/B62 root integration, locked workspace `cargo check`, strict all-target/all-feature Clippy, the focused camera-device gate, docs-check, formatting, `git diff --check`, and dependency audit passed. The 45-crate portable set did not change. No tests or live camera operations were run.
- After D58/B63 root integration, locked workspace `cargo check`, strict all-target/all-feature Clippy, the focused StoreKit 2 gate, docs-check, formatting, `git diff --check`, dependency audit, and capability JSON parsing passed. `cargo test --locked -p framework-sharing` passed 11 tests across clipboard and outgoing-share contracts. No full workspace tests, live purchase query, or device/Simulator probe execution occurred; the portable set remains 45 crates.
- After D47/B52 root integration, `cargo +1.94.1 check --locked --workspace`, strict all-target/all-feature Clippy, the 41-crate no-std check, `no-std-link-probe`, and dependency audit passed. The probe's host executable ran with exit code 0; device and Simulator dylibs linked and passed audits without execution. The report still marks aggregate `all_portable_crates_no_std_linkage_proven` false because 27 API-only crate calls lack separate object attribution; it records 13 object-evidence entries. No workspace tests were run in this pass
- After D26/B31 and D27/B32 integration, `cargo +1.94.1 check --locked --workspace`, `cargo +1.94.1 clippy --locked --workspace --all-targets --all-features -- -D warnings`, `cargo +1.94.1 test --locked --workspace`, and `cargo +1.94.1 fmt --all -- --check` passed on Xcode 26.6 / SDK 26.5. The workspace Clippy run caught an existing `clippy::drop_non_drop` warning in the Contacts future-borrow test; a lexical scope now releases the borrow, and the full strict Clippy run passes
- After D28/B33, D29/B34, and D30/B35 integration, the full workspace check, strict all-target Clippy, workspace tests/doctests, formatting, workspace rustdoc, docs/zero-Swift checks, and dependency audit passed. The 33-package no-std check and host/device/Simulator link probe also passed. These are compile/link/host-contract gates only; no live WebKit page, Bluetooth prompt/scan, or iCloud account query was exercised
- G4–G11 target-gate evidence and individual command results are recorded in their focused plan files. G4 and G6 additionally have macOS link/import probes; those probes link but do not execute. The other backend compile/Clippy gates likewise do not establish live capability behavior
- G12 wiring and exact local results are recorded in `PLAN_VALIDATION_IOS_CONNECTIVITY.md`. The arm64 device and Simulator probe executables import exactly Network.framework and `/usr/lib/libSystem.B.dylib`; they were not executed. Compile/link/import evidence does not establish path-change delivery, connectivity, endpoint reachability, request success, or cancellation timing
- G13 wiring and exact local results are recorded in `PLAN_VALIDATION_IOS_DATA.md`. The arm64 device and Simulator probe executables import exactly CoreFoundation.framework and `/usr/lib/libSystem.B.dylib`; their observed deployment versions are device 10.0 and Simulator 14.0, SDK 26.5 for both. They were not executed. Compile/link/import evidence does not establish live app use, `NSData` runtime behavior, parity, memory-pressure allocation behavior, or performance
- G14's exact local results and initial script failure/resolution are recorded in `PLAN_VALIDATION_IOS_URL.md`. Both arm64 probes import exactly Foundation.framework, `/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` confirms Objective-C runtime symbols, and `strings` contains `NSString`, `NSURL`, and `URLWithString:encodingInvalidCharacters:` but not legacy `URLWithString:`. `vtool` reports iOS 17.0 minimum and SDK 26.5 for device and Simulator, matching the separate B20 API floor of 17.0. Probes were linked, not executed. Compile/link/import evidence does not establish Foundation parser parity, acceptance of every D8 `Uri`, URL-open behavior, runtime availability under an incorrect app deployment target, or performance
- G15 exact local results are recorded in `PLAN_VALIDATION_IOS_GEOMETRY.md`. Device/Simulator checks and strict Clippy passed; the arm64 probe links exactly CoreGraphics.framework and `/usr/lib/libSystem.B.dylib`, with `_CGRectIntersection`, `_CGRectIsNull`, and `_CGRectIsEmpty` imported from CoreGraphics, and the C/Rust layout assertions pass. The linker strips unused UIKit/Foundation/CoreFoundation/libobjc load commands. Probes were built but not executed; these checks do not establish native visual behavior, runtime geometry parity, or general geometry support
- G16 exact local results are recorded in `PLAN_VALIDATION_IOS_MEDIA.md`. Device/Simulator checks, strict Clippy, and the import/layout script passed. Both arm64 probes import exactly CoreMedia.framework and `/usr/lib/libSystem.B.dylib`; `dyld_info -imports` maps `_CMTimeMake` to CoreMedia and all C/runtime symbols to libSystem. C/Rust `CMTime` layout assertions pass. Probes were built but not run; no runtime media behavior is established
- G17 exact local results are recorded in `PLAN_VALIDATION_IOS_TEXT_METRICS.md`. The shared `ios-ui` device/Simulator checks and strict Clippy passed; the Release probes import exactly CoreText.framework, CoreFoundation.framework, and `/usr/lib/libSystem.B.dylib`. The C fixture confirms the public CoreText function signatures and LP64 `CGFloat`/enum layouts. Probes were built but not run; no runtime font output or `UILabel` parity is established
- G18 exact local results are recorded in `PLAN_VALIDATION_IOS_CONNECTION.md`. Eleven portable `framework-connection` tests, Rust 1.94.1 device/Simulator checks, strict all-target Clippy, and `sh platform/ios/ios-connection/check-link-imports.sh` passed on Xcode 26.6 / SDK 26.5. Both arm64 probes import exactly Network.framework and `/usr/lib/libSystem.B.dylib`; `vtool` reports minimum iOS 12.0 for device and 14.0 for Simulator. Probes were not executed; checks do not establish DNS, peer identity, live reachability, TLS behavior, byte delivery, cancellation timing, or performance
- G19 exact local results are recorded in `PLAN_VALIDATION_IOS_BACKGROUND_TASKS.md`. Rust 1.94.1 no-default portable check/Clippy, device/Simulator `ios-background-tasks` checks, strict all-target Clippy, and `sh platform/ios/ios-background-tasks/check-link-imports.sh` passed on Xcode 26.6 / SDK 26.5. Both arm64 probes import exactly BackgroundTasks, Foundation, `/usr/lib/libSystem.B.dylib`, and `/usr/lib/libobjc.A.dylib`; minos is device 13.0 and Simulator 14.0. Probes were not executed; no BGTask launch, scheduling, expiry, relaunch, or delivery behavior is claimed
- The focused plan files record local checks, not a passing CI workflow run. The recorded host is Xcode 26.6 / iOS SDK 26.5, below the Xcode 27.x plan baseline; the manifest warns about this mismatch
- Remaining evidence limits: full-application no-std linkage, arm64 simulator app execution, physical-device behavior/performance, real parity suites, signed archive/export, provisioning, and entitlement-specific runtime behavior are not passed. The bounded probe dylibs do not establish simulator/device execution or full framework coverage

- G131 records aggregate validation for the integrated checkpoint. `cargo +1.94.1 fmt --all -- --check`, `cargo +1.94.1 check --locked --offline --workspace`, `cargo +1.94.1 clippy --locked --offline --workspace -- -D warnings`, `cargo +1.94.1 doc --locked --offline --workspace --no-deps`, `target/debug/xtask docs-check`, `target/debug/xtask zero-swift-source`, manifest consistency (`113` rows, `B=94`, `X=19`), CI YAML parsing, and `git diff --check` passed. B266 and accessibility B271/B273/B276 focused checks also passed in their scoped workstreams. No tests, app launches, live filesystem/accessibility/capability queries, or CI workflow run are claimed. The host remains Xcode 26.6 / iOS SDK 26.5, below the Xcode 27.x baseline.
- G132 records aggregate validation after B274/B280/B291 changed matrix coverage to 114 rows (`B=97`, `X=17`; 85.1%). `cargo +1.94.1 fmt --all -- --check`, locked/offline workspace `cargo check`, strict workspace Clippy, workspace rustdoc, `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`, manifest consistency, CI YAML parsing, and `git diff --check` passed. Focused gates also passed for AccessorySetupKit B274, AlarmKit B280, Photogrammetry B291/B298, accessibility B305/B309/B313/B317, and app-data B296/B301. B303/B304/B306/B307/B310 record no-go audits; none changes matrix coverage. No tests, app launches, probe execution, live filesystem/accessibility/capability queries, or CI workflow run are claimed. Static link/import probes built in focused workstreams were not executed. The host remains Xcode 26.6 / iOS SDK 26.5, below the repository's Xcode 27.x baseline.
- G133 records aggregate validation after the later accessibility B325/B330 and app-data integrations with the matrix unchanged at 114 rows (`B=97`, `X=17`; 85.1%). `cargo +1.94.1 fmt --all -- --check`, locked/offline workspace check, strict workspace Clippy, workspace rustdoc, `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`, manifest consistency (unique IDs, 16 families, `B=97`, `X=17`), CI YAML parsing, and `git diff --check` passed. Focused accessibility device/Simulator checks, strict Clippy, rustdoc, format, docs-check, and scoped diff checks passed for B305/B309/B313/B317/B325/B330; app-data B296/B301 and the previously recorded B274/B280/B291/B298 focused gates remain green. DeviceActivity B338, accessibility B318/B331/B335/B340/B343/B346/B349/B352/B355/B358/B361 and app-data B324/B328/B329/B332/B333/B334/B336/B337/B342/B345/B351 audit details are recorded as deferred/no-go and leave coverage unchanged. No tests, app launches, probe execution, live filesystem/accessibility/capability queries, or CI workflow run are claimed. Static link/import probes were not executed. The host remains Xcode 26.6 / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

- G134 records aggregate validation after B339/B394 HomeKit and B359/B365 app-data slices, accessibility B379/B382/B410, and the B432/B431 audits with the matrix unchanged at 114 rows (`B=98`, `X=16`; 86.0%). `cargo +1.94.1 fmt --all -- --check`, locked/offline workspace `cargo check`, strict workspace Clippy, workspace rustdoc, `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`, manifest consistency (unique row IDs, 16 families, `B=98`, `X=16`), CI YAML parsing, and `git diff --check` passed. Focused B394 device/Simulator checks, strict Clippy, host/device rustdoc, and package format/docs gates passed; B410 device/Simulator checks, strict Clippy, device rustdoc, and package format/docs gates passed. B431 remains a feasibility candidate with no API pending confirmation of Darwin return semantics. No tests, app launches, probe executions, live filesystem/accessibility/HomeKit queries, or CI workflow run are claimed. The host remains Xcode 26.6 / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

- G135 records `sh examples/ios-minimal/build.sh simulator` and `sh examples/ios-minimal/build.sh device` passing as Release build/package gates. `file` reports arm64 Mach-O executables; `xcrun vtool -show-build` reports `IOSSIMULATOR` and `IOS` with minos 17.0 / SDK 26.5. No app launch, archive creation, signing, physical-device execution, or UI interaction is claimed.

- G136 records `cargo +1.94.1 xtask archive-smoke` passing on Xcode 26.6 / iOS SDK 26.5. Xcode created and validated an unsigned `.xcarchive`; `codesign` confirmed no signature, CodeSignature directory, or provisioning profile. The archived app imports exactly UIKit, Foundation, CoreFoundation, `libobjc.A.dylib`, and `libSystem.B.dylib`. Xcode emitted interface-orientation and launch-configuration/storyboard warnings; the archive still passed. This does not validate signing, provisioning, export, installation, launch, or physical-device behavior.

- G137 records aggregate validation after app-data B431/B439/B446, accessibility B434/B443/B444/B447/B450/B451/B453, HomeKit B445/B449/B452, VPN B448, and removal of the redundant `UIDeviceFamily` plist key. The matrix remains 114 rows (`B=98`, `X=16`; 86.0% row coverage). `cargo +1.94.1 fmt --all -- --check`, locked/offline workspace check, strict workspace Clippy, workspace rustdoc with warnings denied, `xtask docs-check`, `xtask zero-swift-source`, capability-manifest consistency (unique IDs, 16 families, `B=98`, `X=16`), CI YAML parsing, plist lint, and `git diff --check` passed. Focused device and Simulator check/strict-Clippy passed for accessibility and HomeKit setup-result count; device rustdoc with warnings denied passed for both. The updated unsigned archive smoke passed; its only Xcode warnings were that all interface orientations must be supported unless full-screen is required and that a launch configuration/storyboard/xib must be provided unless full-screen is required. Imports remain exactly UIKit, Foundation, CoreFoundation, `libobjc.A.dylib`, and `libSystem.B.dylib`; no duplicate `UIDeviceFamily` warning remains. No tests, app launches, live filesystem/accessibility/HomeKit queries, signing, provisioning, or CI workflow run are claimed. Host: Xcode 26.6 / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

## Documentation infrastructure

Generate/index:
- support matrix;
- capability availability;
- benchmark decisions;
- dependency inventory;
- ABI version manifest.

Do not overwrite hand-written semantic docs with generated noise.

Require each workstream to update:
- rustdoc;
- capability guide;
- architecture note if invariant changed;
- parity/performance decision where relevant.

## Final independent audit

Before V1 signoff:
1. verify current `main` against PLAN.md;
2. compare every workstream output to its plan;
3. inspect all dependency additions;
4. inspect unsafe/assembly inventory;
5. inspect `std` leaks;
6. inspect all linked Apple frameworks;
7. verify unsupported/deferred capability list;
8. rerun cross-workstream tests;
9. review final diff for accidental architectural drift;
10. ensure PLAN files are deleted before final implementation completion.

## Handoff

Report:
- tool/CI commands;
- generated support matrix;
- parity coverage summary;
- benchmark decision summary;
- ABI/linkage/dependency audit results;
- unresolved environment-only validation (for example entitlement or physical-device access).

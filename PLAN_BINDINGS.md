# PLAN_BINDINGS.md — Workstream F: Stable C/C++/Python Binding Surfaces

## Objective

Expose the completed Rust framework to foreign languages without making the foreign ABI the Rust implementation path.

## Dependencies

Requires A foundation.
Capability-specific bindings start only after the corresponding D public contract is stable.

## Execution decomposition

Start with `PLAN_BINDINGS_CORE.md` (F1), which freezes and validates the foundational C ABI from A without waiting for capability crates. F2 is `PLAN_BINDINGS_SECURE_STORAGE.md`; it adds a capability-scoped C API only after D2 and B2 are integrated. `PLAN_BINDINGS_CPP.md` (F3) owns a small header-only C++ convenience layer over the existing core ABI. `PLAN_BINDINGS_NOTIFICATION_RESPONSES.md` (F4) exposes D9 response values without delivery or delegate behavior. `PLAN_BINDINGS_TRANSFER.md` (F5) adds an opt-in iOS transfer C ABI over B13's concrete backend, with a host-app event-forward seam. `PLAN_BINDINGS_CLIPBOARD.md` (F6) adds an opt-in iOS C ABI for D5/B6 plain-text clipboard operations, with `ios-sharing` source features that isolate the C clipboard path from share UI. `PLAN_BINDINGS_SHARE.md` (F7) adds an opt-in iOS C ABI over B7's retained `IosShareSession`. `PLAN_BINDINGS_ASYNC.md` (F8) records the capability-scoped async policy; no universal C operation registry is required. F9 is the opt-in D1/B1 preferences API in `PLAN_BINDINGS_PREFERENCES.md`; it exposes only the standard `NSUserDefaults` byte-value facade and does not replace or broaden the portable contract. F10 is the opt-in D65/B71 MediaPlayer authorization-status API in `PLAN_BINDINGS_MEDIA_LIBRARY_STATUS.md`; it preserves the signed native status without adding a portable or non-iOS capability. F11 is the opt-in D64/B70 detached-SpriteKit-position C ABI in `PLAN_BINDINGS_SPRITEKIT.md`; it retains the main-thread and opaque-handle contract. F12 adds an opt-in D76/B72 CallKit count/state snapshot C ABI in `PLAN_BINDINGS_CALL_OBSERVER.md`; it exposes no call object or identifier. F13 adds an opt-in D77/B73 MapKit geometry C ABI in `PLAN_BINDINGS_MAPS.md`; it exposes only finite point conversion and distance values. F14 adds an opt-in D81/B74 ClassKit marker C ABI in `PLAN_BINDINGS_CLASSKIT.md`; it accepts a caller-borrowed opaque `NSUserActivity` pointer and returns one Boolean without enabling assignment data access. F15 adds an opt-in B5 Location C ABI in `PLAN_BINDINGS_LOCATION.md`; each opaque handle owns one operation future, and its callback is a main-thread readiness hint that requires a later poll for results. F16 adds an opt-in B75 FileProvider registered-domain query in `PLAN_BINDINGS_FILEPROVIDER.md`; it copies borrowed identifiers and returns caller-owned result buffers without exposing provider operations. F17 adds an opt-in B57 Vision text-recognition revision-membership query in `PLAN_BINDINGS_F17.md`; it writes one caller-owned Boolean and does not create requests or read images. F18 adds an opt-in B76 ProximityReader device-model query in `PLAN_BINDINGS_F18.md`; it does not claim payment readiness. F19 exposes B66's CommonCrypto SHA-256 operation through one caller-buffer C function in `PLAN_BINDINGS_F19.md`. F20 exposes B67's ModelIO extension-support Boolean in `PLAN_BINDINGS_F20.md`. F21 exposes B78's prior Sign in with Apple credential-state query with one caller completion in `PLAN_BINDINGS_F21.md`, without general readiness or authorization UI claims. F22 exposes B65's single-precision vector addition through the opt-in C ABI in `PLAN_BINDINGS_F22.md`. F23 exposes B69's Security query for caller-supplied P-256 public-key suitability in `PLAN_BINDINGS_F23.md`, without signature verification or private-key operations. F24 exposes B68's MPS preferred-device presence Boolean in `PLAN_BINDINGS_F24.md`, without GPU work or workload support claims. F25 exposes B50's VideoToolbox hardware-decode predicate in `PLAN_BINDINGS_F25.md`, without session or media operations. F26 exposes B62's default video-capture-device presence Boolean in `PLAN_BINDINGS_F26.md`, without authorization or capture-readiness claims. F27 exposes B56's Core ML available-compute-device-list Boolean in `PLAN_BINDINGS_F27.md`, without model or inference claims. Python remains optional and must not block the C ABI.

F28 adds the opt-in `ios-speech-status` query over B58's `SFSpeechRecognizer.authorizationStatus`. It writes the signed native code unchanged, including unknown values; below iOS 10.0 it returns `FRAMEWORK_STATUS_UNAVAILABLE`. It does not request permission or process audio. See `PLAN_BINDINGS_F28.md`.

F29 adds the opt-in `ios-natural-language-status` query over B59's English contextual-model asset state. It maps the five backend cases to fixed `uint32_t` codes and does not load a model, process text, compute vectors, or request assets. See `PLAN_BINDINGS_F29.md`.

F30 adds an opt-in `ios-extension-support` C function that reads one caller-selected `.appex`
`NSExtension.NSExtensionPointIdentifier` value into a caller-owned UTF-8 buffer and maps B77's
eight metadata outcomes. It does not load or launch extensions. The backend API floor is iOS 4.0;
measured link minos are device 12.0 and Simulator 14.0. See `PLAN_BINDINGS_F30.md`.

F31 exposes B61's `RoomCaptureSession.isSupported` value as one opt-in C Boolean output. It does not
create a session, access camera/LiDAR frames, request permission, present UI, or start a scan. The
API and required deployment floor are iOS 16.0. Host C/C++ imports only libSystem; device/Simulator
C/C++ imports RoomPlan and libSystem with minos 16.0. See `PLAN_BINDINGS_F31.md`.

F32 exposes B63's `AppStore.canMakePayments` as one opt-in C Boolean; its API/symbol floor is iOS 15.0,
separate from the device/Simulator link minos 10.0/14.0. The StoreKit framework and getter are weak
imports, with no runtime fallback check below iOS 15.0. F33 exposes only B55's local-player
authentication snapshot; the SDK API floor is iOS 4.1, distinct from link minos 10.0/14.0, and the
signed app needs `com.apple.developer.game-center`. Neither adds capability-row coverage. See
`PLAN_BINDINGS_F32.md` and `PLAN_BINDINGS_F33.md`.

## Status and evidence

F1–F33 source, public headers, and named validation paths are present in the current tree; `bindings/c/abi-manifest.json` records the core and capability C exports. Their plan records state:

- F1: `sh bindings/c/check.sh` passed ABI layout and symbol checks, linked the minimal C consumer, and ran it; the consumer printed `framework ABI 1.0`
- F2: `sh bindings/c/check-secure-storage.sh` passed, including 10 package tests (eight secure-storage-specific), C11/C++17 header compilation, the host unsupported-stub run, and arm64 iOS device/simulator Clippy, archive, and C-probe link/import checks; the linked iOS probes did not run
- F3: `sh bindings/cpp/check.sh` passed formatting and compiled, linked, and ran `consumer.cpp` and
  `owner-consumer.cpp` against the real C API archive. The real-archive owner consumer proves
  symbol resolution and process exit only because F1 has no core buffer creator; the
  `ownership.cpp` test stub checks move-only traits and exactly-once destroy behavior. Object
  `nm -u` checks found only the expected C ABI symbols. Links use `-nostdlib++`; no C++ runtime
  symbol is required, and no final-binary import audit is claimed
- F4: `sh bindings/c/check-notification-responses.sh` passed manifest-derived tag/layout and symbol checks plus C11/C++17 host links; current ownership prose was manually checked against the implementation, header, and guide; linked consumer binaries were not executed, and no Rust tests ran
- F5: `sh bindings/c/check-ios-transfer.sh` passed feature, layout, symbol, device/simulator compile, and linked-probe import checks; probes did not run
- F6: `sh bindings/c/check-ios-clipboard.sh` passed host/device/simulator feature, compile/link, and import checks; consumers did not run
- F7: `sh bindings/c/check-ios-share.sh` passed again on 2026-10-08 after the minimal example formatting correction; it checks feature, layout, symbol, direct-import, and forbidden-symbol constraints. The linked probes and tests do not run
- F8: closed by design disposition after source and contract audit; F5's client plus durable `TransferId` and F7's single-operation `IosShareSession` meet the async ABI gate with capability-scoped lifetimes; no shared async API, operation registry, or executor is required. See `PLAN_BINDINGS_ASYNC.md`
- F9: `sh bindings/c/check-ios-preferences.sh` validates the opt-in feature graph, manifest tags/symbols, C11/C++17 consumers, and arm64 device/Simulator check, Clippy, archive, linked-import, and deployment-minimum gates. Linked consumers are build-only; no live defaults behavior is claimed. See `PLAN_BINDINGS_PREFERENCES.md` for exact evidence and runtime limits
- F10: `sh bindings/c/check-ios-media-library-status.sh` passed in the integrated checkout. It checks opt-in feature isolation, host/device/Simulator checks and strict Clippy, C11/C++17 consumers, exact MediaPlayer/Foundation imports and F10 symbols, selector strings, and probe minimums. Consumers/probes are build-only; no live authorization result is claimed. See `PLAN_BINDINGS_MEDIA_LIBRARY_STATUS.md`
- F11: `sh bindings/c/check-ios-spritekit.sh` passed in the integrated checkout. It checks opt-in feature isolation, locked host/device/Simulator checks and strict Clippy, C11/C++17 links, four exported symbols, exact SpriteKit/Foundation imports, and deployment minimums. Probes were not executed; B70’s iOS 7.0 API floor remains distinct from the 12.0/14.0 probe floors. See `PLAN_BINDINGS_SPRITEKIT.md`
- F12: `sh bindings/c/check-ios-call-observer.sh` passed locked host/device/Simulator checks and strict Clippy, Release archives, feature isolation, C11/C++17 compile/link, one exported function, exact CallKit/Foundation/libSystem/libobjc imports, and device/Simulator minos 12.0/14.0. Probes and linked consumers were not executed; B72’s API floor remains iOS 10.0. See `PLAN_BINDINGS_CALL_OBSERVER.md`
- F13: `sh bindings/c/check-ios-maps.sh` passed locked host/device/Simulator checks and strict Clippy, Release archives, feature isolation, C11/C++17 compile/link, three exported functions, exact MapKit/libSystem imports (plus libc++ for C++), and minos 12.0/14.0. Probes and linked consumers were not executed; B73's API floor remains iOS 4.0. See `PLAN_BINDINGS_MAPS.md`
- F14: `sh bindings/c/check-ios-classkit-deep-link.sh` passed feature isolation, locked host/device/Simulator checks and strict Clippy, Release archives, C11/C++17 compile/link, one export, exact ClassKit/Foundation/libSystem/libobjc imports (plus libc++ for C++), selector and forbidden-ClassKit-symbol guards, and minos 11.3/14.0. Probes and linked consumers were not executed; B74's API floor is iOS 11.3. See `PLAN_BINDINGS_CLASSKIT.md`
- F15: `sh bindings/c/check-ios-location.sh` passed after root lock refresh. Each opaque handle owns one operation future; its main-thread readiness callback is only a hint and the caller polls later for the result. Host/device/Simulator checks, strict Clippy, Release builds, C11/C++17 links, feature isolation, imports, and deployment gates passed; probes were inspected, not executed. The device gate retains an unexplained iOS 9.0 deployment-target warning while final probe minos is 10.0. See `PLAN_BINDINGS_LOCATION.md`
- F16: `sh bindings/c/check-ios-file-provider.sh` passed after root refreshed the Cargo lockfile. Host and iOS device/Simulator checks, strict Clippy, Release archives, feature isolation, C11/C++17 links, exact imports, and deployment gates passed. Device minos is 11.0; Simulator minos is 14.0. Each call copies the caller's borrowed domain identifier and returns an owned result buffer; only start/poll/destroy are exported. Probes were not executed. See `PLAN_BINDINGS_FILEPROVIDER.md`
- F17: `sh bindings/c/check-ios-vision.sh` passed after root wiring. Host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, symbols, imports, and minos 13.0/14.0 passed; its static gate also asserts aligned writable output storage through the synchronous call, no unsynchronized access, and no pointer retention. Probes were not executed. The initial manifest assertion used the wrong status key and was corrected from `valid_result` to `success` before the passing run. See `PLAN_BINDINGS_F17.md`
- F18: `sh bindings/c/check-ios-proximity-reader.sh` passed after root wiring and lock refresh. Host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact ProximityReader/libSystem imports, Swift symbol checks, and minos 15.4 passed; its static gate also asserts aligned writable output storage through the synchronous call, no unsynchronized access, and no pointer retention. Probes were not executed. See `PLAN_BINDINGS_F18.md`
- F19: `sh bindings/c/check-ios-crypto.sh` passed host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact `libSystem.B.dylib`/`_CC_SHA256` imports, export parity, and minos 10.0/14.0. Static assertions cover null/span validation, zero-before-platform/slice order, length bounds, full-call input/output ownership, and synchronization terms. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F19.md`
- F20: `sh bindings/c/check-ios-modelio-status.sh` passed host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact Foundation/ModelIO/libSystem/libobjc imports (+libc++ for C++), selector/message-send checks, and minos 10.0/14.0. Its static gate asserts aligned writable output storage, full-call lifetime, caller concurrency protection, range/overlap checks, and the wrapper's inability to prove memory validity. No tests or consumers/probes were executed. See `PLAN_BINDINGS_F20.md`
- F21: `sh bindings/c/check-ios-sign-in-with-apple-status.sh` passed static C11/C++17 syntax, symbol-parity, formatting, and whitespace checks. `sh bindings/c/check-ios-sign-in-with-apple-status-link.sh` passed arm64 device/Simulator C11/C++17 links, exact imports, Objective-C/block symbols, the single export, forbidden-symbol checks, and minos 13.0/14.0. The native gate is wired in macOS CI; consumers/probes were not executed. See `PLAN_BINDINGS_F21.md`
- F22: `sh bindings/c/check-ios-accelerate.sh` passed manifest/symbol/feature checks, host/device/Simulator compilation and strict Clippy, Release builds, C11/C++17 links, exact Accelerate/libSystem imports (+libc++ for C++), `_vDSP_vadd`, exports, and minos 10.0/14.0. Static assertions cover array byte bounds, alignment, alias policy, overlap rejection before slices, full-call pointer ownership, and synchronization terms. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F22.md`
- F23: `sh bindings/c/check-ios-key-support.sh` and `sh bindings/c/check-ios-key-support-link.sh` passed. The static gate asserts writable output bounds, full-call lifetime, no unsynchronized access, input/output non-overlap, and no output-pointer retain. Host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact CoreFoundation/Security/libSystem imports (+libc++ for C++), Security symbols, forbidden-symbol checks, export parity, and minos 10.0/14.0 passed. The gates are wired in macOS CI; no passing workflow run is recorded. No consumers or probes were executed. See `PLAN_BINDINGS_F23.md`
- F24: `sh bindings/c/check-ios-mps-status.sh` and `sh bindings/c/check-ios-mps-status-link.sh` passed host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact Foundation/Metal/MetalPerformanceShaders/libSystem/libobjc imports (+libc++ for C++), `_MPSGetPreferredDevice` and `_objc_release`, forbidden-symbol checks, export parity, and minos 12.2/14.0. The static gate asserts valid aligned writable output memory, full-call lifetime, nullness-only validation, no unsynchronized access, and no output-pointer retain. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F24.md`
- F25: `sh bindings/c/check-ios-videotoolbox.sh` and `sh bindings/c/check-ios-videotoolbox-link.sh` passed in the integrated checkout. Host/device/Simulator feature isolation, strict Clippy, Release archives, C11/C++17 links, exact VideoToolbox/libSystem device and Simulator imports, export parity, and minos 11.0/14.0 passed. Host C imported libSystem only; host C++ also imported libc++. A minos-10.0 device link showed a strong `_VTIsHardwareDecodeSupported` import, so no pre-iOS-11 link/load claim is made. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F25.md`
- F26: `sh bindings/c/check-ios-camera-device-status.sh` and `sh bindings/c/check-ios-camera-device-status-link.sh` passed after root integration. Host/device/Simulator feature isolation, strict Clippy, Release archives, C11/C++17 links, exact AVFoundation/libSystem/libobjc device and Simulator imports, export parity, and probe minos 10.0/14.0 passed; Foundation was linked but dead-stripped. The API floor remains iOS 4.0. Its static gate asserts source/header/guide/plan/manifest full-call output validity, alignment, writability, zeroing, synchronization, and non-retention wording. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F26.md`
- F27: `sh bindings/c/check-ios-core-ml-status.sh` and `sh bindings/c/check-ios-core-ml-status-link.sh` passed after root integration. Host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 links, exact CoreML/Foundation/libSystem/libobjc device and Simulator imports, export parity, and minos 11.0/14.0 passed. Host C and C++ imported only libSystem; C++ used `-nostdlib++`. The API floor is iOS 17.0. Its static gate asserts source/header/guide/plan/manifest full-call output validity, alignment, writability, zeroing, synchronization, and non-retention wording. Both gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F27.md`
- F28: `sh bindings/c/check-ios-speech-status.sh` and `sh bindings/c/check-ios-speech-status-link.sh` passed after root integration. Host/device/Simulator feature isolation, strict Clippy, rustdoc, C11/C++17 headers and links, exact imports, export parity, and minos 10.0/14.0 passed. Host imports are libSystem only; device/Simulator imports are Foundation, Speech, libSystem, and libobjc. C++ used `-nostdlib++`. The API floor is iOS 10.0. Its static gate asserts source/header/guide/plan output validity, aligned writable `int64_t` lifetime, zeroing, synchronization, and non-retention wording. Gates are wired in macOS CI; no passing workflow run is recorded. No tests, consumers, or probes were executed. See `PLAN_BINDINGS_F28.md`
- F29: `sh bindings/c/check-ios-natural-language-status.sh` and `sh bindings/c/check-ios-natural-language-status-link.sh` passed in the integrated checkout. Host C/C++ import only `libSystem.B.dylib`; device/Simulator C/C++ import Foundation, NaturalLanguage, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos is 17.0/17.0. The C ABI exposes B59's five English contextual-model asset states as fixed `uint32_t` codes and creates/drops one temporary model object, without model load, text, vector, or asset request. No tests, consumers, or probes ran; no passing CI workflow run is recorded. See `PLAN_BINDINGS_F29.md`
- F30: `sh bindings/c/check-ios-extension-support.sh` and `sh bindings/c/check-ios-extension-support-link.sh` passed after root integration. Host C/C++ import only `libSystem.B.dylib`; device/Simulator C/C++ import exactly Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; minos is 12.0/14.0. The API returns only B77's caller-selected `.appex` extension-point metadata and eight exact metadata outcomes. No tests, consumers, or probes were executed; no passing CI workflow run is recorded. See `PLAN_BINDINGS_F30.md`
- F31: `sh bindings/c/check-ios-roomplan-status.sh` and `sh bindings/c/check-ios-roomplan-status-link.sh` passed after root integration. Host/device/Simulator feature isolation, Rust checks, strict Clippy, rustdoc, C11/C++17 header syntax and links, exact RoomPlan/libSystem target imports, required RoomPlan symbols, and minos 16.0 passed. Consumers and probes were not executed; no RoomPlan call ran. See `PLAN_BINDINGS_F31.md`
- F32: `sh bindings/c/check-ios-storekit2-status.sh` and `sh bindings/c/check-ios-storekit2-status-link.sh` passed after root integration. Host imports were only `libSystem.B.dylib`; device/Simulator C/C++ imported weak StoreKit plus `libSystem.B.dylib`, with the getter symbol weak and minos 10.0/14.0. API/symbol floor is iOS 15.0; no runtime check below that floor, tests, or consumer/probe execution occurred. No passing CI run is recorded. See `PLAN_BINDINGS_F32.md`
- F33: `sh bindings/c/check-ios-game-status.sh` and `sh bindings/c/check-ios-game-status-link.sh` passed after root integration. Host imports were only `libSystem.B.dylib`; device/Simulator C/C++ imports were Foundation, GameKit, `libSystem.B.dylib`, and `libobjc.A.dylib`, with minos 10.0/14.0. SDK API floor is iOS 4.1; the signed app needs `com.apple.developer.game-center`. No tests, auth flow, player query, or consumer/probe execution occurred; no passing CI run is recorded. See `PLAN_BINDINGS_F33.md`

F25 adds an opt-in `ios-videotoolbox` query over B50's `VTIsHardwareDecodeSupported` predicate. It accepts one big-endian FourCC `uint32_t` and writes one Boolean byte; it does not create a decoder session, process media, or reserve resources. See `PLAN_BINDINGS_F25.md`

F26 adds an opt-in `ios-camera-device-status` query over B62's current default video-device predicate. It writes one Boolean byte and does not query authorization, configure capture, or establish readiness. The direct C surface needs no C++ RAII wrapper; its API floor is iOS 4.0 while the linked probe minima are device 10.0 and Simulator 14.0. See `PLAN_BINDINGS_F26.md`

F27 adds an opt-in `ios-core-ml-status` query over B56's `MLModel.availableComputeDevices` nonempty-list predicate. It writes one Boolean byte and does not load a model, run inference, or guarantee a device can execute a particular operation. The API floor is iOS 17.0; measured link minos are device 11.0 and Simulator 14.0. Its C and C++ link gates passed; see `PLAN_BINDINGS_F27.md`

F28 adds an opt-in `ios-speech-status` query over B58's `SFSpeechRecognizer.authorizationStatus`. It writes the signed native code unchanged, including unknown values; below iOS 10.0 it returns `FRAMEWORK_STATUS_UNAVAILABLE`. It does not request permission or process audio. See `PLAN_BINDINGS_F28.md`

F29 adds an opt-in `ios-natural-language-status` query over B59's English contextual-model asset state. It maps the five backend cases to fixed `uint32_t` codes and does not load a model, process text, compute vectors, or request assets. See `PLAN_BINDINGS_F29.md`

F30 adds an opt-in `ios-extension-support` C function that reads one caller-selected `.appex`
`NSExtension.NSExtensionPointIdentifier` value into a caller-owned UTF-8 buffer and maps B77's
eight metadata outcomes. It does not load or launch extensions. See `PLAN_BINDINGS_F30.md`

F31 exposes B61's `RoomCaptureSession.isSupported` value as one opt-in C Boolean output. It does not
create a session, access camera/LiDAR frames, request permission, present UI, or start a scan. Its
API and required deployment floor are iOS 16.0; host C/C++ imports only libSystem, while device and
Simulator C/C++ imports RoomPlan and libSystem at minos 16.0. See `PLAN_BINDINGS_F31.md`

F32 exposes B63's StoreKit 2 purchase-ability Boolean through a weak-imported StoreKit getter and has
an iOS 15.0 API/symbol floor; device/Simulator C/C++ link gates pass at minos 10.0/14.0. The absent
symbol fallback was not runtime-checked below the API floor. F33 exposes one Game Center
local-player authentication snapshot with an iOS 4.1 SDK API floor and a signed entitlement
requirement; device/Simulator C/C++ link gates pass at minos 10.0/14.0. Consumers and probes were not
executed, and neither adds capability-row coverage. See `PLAN_BINDINGS_F32.md` and
`PLAN_BINDINGS_F33.md`

`.github/workflows/ci.yml` invokes the F1–F33 named C ABI checks on macOS. This records workflow scope, not a claim that every CI job has a passing run on every supported toolchain

## Open limits

- Future async C scopes must define their own handle or durable ID, acceptance point, result path, cancel effect, destroy/drop rule, callback lifetime, thread, and error map; F8 does not require a cross-capability handle shape or executor
- Larger same-major record compatibility is not proven: `FrameworkOptionsV1` has a measured layout but no exported C API takes it, while F5 `FrameworkTransferRequestV1` and F7 request/anchor inputs require exact V1 sizes. A later API that takes an extensible record must state if a larger size is valid and check `struct_size` and `abi_version` before field reads
- F2, F4–F7, F9–F16 ownership semantics were manually reconciled against source, headers, guides, and contracts; their automated checks validate ownership-record shape/presence and selected fields, not every prose semantic, so future ownership edits require another manual comparison
- C and C++ probes establish link shape only; they do not prove live Keychain, URLSession, pasteboard, privacy UI, notification response delivery, or share recipient behavior
- F9 link probes validate import shape and deployment metadata only; they do not prove live defaults access, consuming-app privacy-manifest compliance, cross-process visibility, persistence flush, or crash durability
- F10 link probes validate import shape and deployment metadata only; they do not prove the live authorization value, access to media items, or service/playback availability
- F11 link probes validate import shape and deployment metadata only; they do not prove live SpriteKit scene/rendering behavior or presentation
- F12 link probes validate import shape and deployment metadata only; they do not prove live call state, call lifecycle, or call-control behavior
- F13 link probes validate import shape and deployment metadata only; they do not prove coordinate-conversion or distance parity with native MapKit results
- F14 link probes validate import shape and deployment metadata only; they do not prove a live incoming activity, ClassKit availability, or Schoolwork assignment behavior
- The recorded F6/F7 Xcode 26.6 and iPhoneOS/iPhoneSimulator SDK 26.5 evidence is below the planned Xcode 27.x baseline
- Python source is not present; Python remains optional and does not block C/C++ completion

## Write scope

- `bindings/c/**`
- `bindings/cpp/**`
- `bindings/python/**`
- `examples/c-minimal/**`
- ABI tests and binding docs

## C ABI

Create a stable modular C ABI using `framework-abi`.

Requirements:
- fixed-width scalars;
- pointer+length strings/bytes;
- versioned structs;
- opaque handles;
- explicit ownership/destruction;
- explicit callbacks;
- a capability-owned opaque handle or durable ID, with documented cancel and destroy/drop rules for async;
- stable status/error codes;
- capability-scoped headers/libraries so one small API does not force unrelated linkage.

Do not expose:
- Rust references;
- `Vec`, `String`, `Box`, `Arc`;
- trait objects;
- Rust enum/layout without specified repr;
- `objc2::Retained`;
- Swift metadata/types;
- platform-native object types in portable contracts.

Platform-specific C escape APIs may expose opaque native handles where explicitly documented.

## C header generation

Choose the smallest dependable strategy:
- hand-maintained header for small stable core if safer;
- or cbindgen-like generation if dependency/tooling value justifies it.

Generated header output must be diff/tested for ABI changes.

## C++ layer

Optional V1 convenience layer:
- header-only RAII where possible;
- typed wrappers over C ABI;
- no runtime registry;
- no second implementation;
- no hidden allocation beyond semantics.

Do not confuse Swift-generated C++ ABI oracle headers with the framework's public C++ API.

## Python

Python is optional and should not block V1 native completion.

If implemented in V1:
- top-level binding only;
- idiomatic Python objects/exceptions/awaitables;
- heavy loops remain Rust;
- avoid chatty per-element crossings;
- no Python dependency for Rust/C builds;
- Python-specific ownership/GIL stops at binding edge.

Select PyO3 or alternative only after dependency/runtime/build-cost review. Keep Python binding machinery replaceable.

## Async ABI

Each capability defines its own async boundary. Use an opaque handle when the backend owns a live session or client, or a stable ID when the backend owns durable work. A callback is optional when the capability offers a queryable result path

Document start rejection and acceptance, result source, callback count and trigger, handle or ID lifetime, cancellation effect, destroy/drop result, callback context lifetime, thread rule, and native/error map. Do not promise a terminal callback if the backend does not guarantee one. A successful cancel may detach a callback, request native cancellation, or prove cancellation only as the capability contract states

F5 uses a per-client B13 handle and host-assigned durable `TransferId`. Task state comes from `framework_ios_transfer_status`; `framework_ios_transfer_cancel` records a durable stop request. Its background-event callback is a separate one-shot UIKit event-drain signal, not a per-task terminal callback

F7 uses one `FrameworkIosShareSession` handle for at most one active operation. Start rejection does not take callback/context ownership. After accepted start, B7 calls the one-shot main-thread callback if UIKit reports a terminal result; cancel or destroy may detach it, and UIKit does not guarantee a terminal result

Do not add a universal operation registry, executor, callback table, or cross-capability handle type. C++ and Python wrappers must preserve each native capability's rules

## ABI compatibility tests

Test:
- `sizeof`/`alignof` known structs;
- symbol exports;
- header compile from C11 and C++;
- ownership create/destroy;
- callback signatures;
- error mapping;
- backward-compatible struct extension for an API that allows a larger same-major record; current C inputs do not make that promise (see Open limits);
- no panic across boundary.

Use an ABI manifest/version record.

## Linkage tests

C minimal consumers for:
- core only;
- one storage capability;
- one networking capability;
- one UI capability.

Verify no unrelated frameworks/bindings.

## Non-goals

- Rust API never calls the C ABI;
- no universal serialized message protocol;
- no COM-like object model;
- no Python runtime in core;
- no freezing internal packed layout into public ABI.

## Handoff

Report:
- exported ABI version;
- header modules;
- ownership table;
- async rules;
- compatibility tests;
- optional bindings actually completed.

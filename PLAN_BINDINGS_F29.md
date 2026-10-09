# PLAN_BINDINGS_F29.md — F29: Natural Language asset-status C ABI

## Objective

Expose the existing B59 English contextual-model asset-state snapshot through one opt-in C
function. Add no portable Natural Language API, model load, text/vector call, asset request, or
general language-model support claim

## Candidate and bounds

D54/B59 is selected because its Rust API is already a five-case owned status enum and the backend
uses typed `objc2-natural-language` 0.3.2 with only `NLContextualEmbedding` and `NLLanguage`
features. It checks the iOS 17.0 floor, obtains `NLLanguageEnglish`, asks the public factory for a
temporary `NLContextualEmbedding`, reads only `hasAvailableAssets`, and drops the object before
return. The inspected SDK has no main-thread annotation for these calls. This does not call
`load`, `unload`, `requestEmbeddingAssetsWithCompletionHandler`, or `embeddingResultForString`,
and does not accept text or start a download

F29 maps B59's five Rust cases to fixed `uint32_t` codes 0–4. These are framework-owned C values,
not raw native enum values. B59 does not provide an unknown native status case. `FRAMEWORK_STATUS_OK`
means the output contains a B59 state, including code `UNAVAILABLE` below iOS 17.0. Non-iOS
returns `FRAMEWORK_STATUS_UNSUPPORTED` and zero output; null output returns
`FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write; panic returns `FRAMEWORK_STATUS_PANIC` and
zero output. The output is initialized to zero before platform handling and is not retained

Other partials remain outside F29: B53 SafetyKit has an unresolved getter-specific entitlement
prerequisite; B55 reads signed Game Center local-player state; B60 is deprecated StoreKit 1; B61
RoomPlan and B63 StoreKit 2 need Swift ABI thunks. They are not part of the B59 C contract

The remaining `X` capability rows remain out of F29. FamilyControls lacks a supported Rust route
and documented query-only entitlement semantics; Foundation Models, ContactProvider,
ExtensionFoundation, MarketplaceKit, MatterSupport, and several service APIs are Swift-only.
DeviceActivity getter meaning and entitlement semantics remain unclear. PushToTalk, CarPlay,
browser/extension, and locked-camera slices need entitlement, APNs, host lifecycle, UI, or session
state. HomeKit's first manager use can prompt. See the scoped capability plans. F29 adds no
capability row and changes no matrix count

## Exact ABI

- Cargo feature: `ios-natural-language-status`; default remains empty
- Header: `bindings/c/include/framework_ios_natural_language_status.h`
- Export: `FrameworkStatus framework_ios_natural_language_english_contextual_embedding_assets(FrameworkIosNaturalLanguageAssetStatus *out_status)`
- `FrameworkIosNaturalLanguageAssetStatus` is fixed `uint32_t`: `UNAVAILABLE` 0,
  `LANGUAGE_UNAVAILABLE` 1, `NO_MODEL` 2, `ASSETS_NOT_AVAILABLE` 3, `ASSETS_AVAILABLE` 4
- iOS 17.0 and later return `FRAMEWORK_STATUS_OK` with a code from B59; below iOS 17.0 returns
  `FRAMEWORK_STATUS_OK` with `UNAVAILABLE` 0
- A valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` and output zero
- A non-null `out_status` must point to valid, properly aligned, writable `uint32_t` storage for the
  full synchronous call. The caller must prevent unsynchronized concurrent access. The API checks
  only nullness and does not retain the output pointer. The output is initialized to zero before
  platform handling
- A null output pointer returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write; panic returns
  `FRAMEWORK_STATUS_PANIC` with zero output and no unwind across C
- Calls are synchronous on the caller's thread; no main-thread rule or broad thread-safety promise
- No Natural Language object, model handle, text, vector, callback, or native pointer crosses C

## Source evidence

- Apple, [`NLContextualEmbedding`](https://developer.apple.com/documentation/naturallanguage/nlcontextualembedding): contextual-model factory and asset APIs
- Apple, [`hasAvailableAssets`](https://developer.apple.com/documentation/naturallanguage/nlcontextualembedding/hasavailableassets): on-device asset predicate
- Installed SDK: `$(xcrun --sdk iphoneos --show-sdk-path)/System/Library/Frameworks/NaturalLanguage.framework/Headers/NLContextualEmbedding.h`, which declares `NLContextualEmbedding` from iOS 17.0 and distinguishes asset presence from model load/request
- Installed Rust binding: `objc2-natural-language` 0.3.2 exposes typed `contextualEmbeddingWithLanguage`, `hasAvailableAssets`, and `NLLanguageEnglish`; B59 enables only the two used feature groups

## Validation and evidence

`sh bindings/c/check-ios-natural-language-status.sh` and
`sh bindings/c/check-ios-natural-language-status-link.sh` are the non-test gates. They inspect
status codes, feature closure, host/device/Simulator builds, strict Clippy, rustdoc, C11/C++17
headers and links, exact imports, exports, forbidden model operations, deployment metadata, and the
valid aligned output-memory, full-call lifetime, zero-before-platform, nullness-only check,
no-concurrent-access, and no-pointer-retain contract across source/header/guide/plan/manifest where
applicable. The static gate asserts null rejection before the zero write and the zero write before
the panic/platform boundary, plus the nullness-only error branch. After these order checks were
added, root reran both gates; host/device/Simulator feature isolation, strict Clippy, rustdoc,
C11/C++17 links, exact Foundation/NaturalLanguage/libSystem/libobjc imports, export parity,
forbidden model-operation checks, and minos 17.0/17.0 passed. The gates do not execute consumers or
probes. The offline
`cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features
ios-natural-language-status` also passed and refreshed the root `Cargo.lock` feature edge

The Release link gate measured host C and C++ imports as only `libSystem.B.dylib`. Device and
Simulator C and C++ imports were exactly Foundation, NaturalLanguage, `libSystem.B.dylib`, and
`libobjc.A.dylib`; the C++ links used `-nostdlib++` and imported no `libc++`. `vtool` measured
minos 17.0 for both device and Simulator, matching the iOS 17.0 API floor. C11/C++17 host,
device, and Simulator consumers linked, archive exports matched the manifest, and required
Objective-C/NaturalLanguage symbols and selectors were present; forbidden model load, text,
asset-request, and Swift surfaces were absent

Validation used Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the repository's
Xcode 27.x baseline. No tests or linked consumers/probes were executed, and no passing CI workflow
run is recorded

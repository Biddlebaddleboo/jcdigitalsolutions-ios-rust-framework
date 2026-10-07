# Swift-First Residuals — Pass 1

Research date: 2026-10-07

This document begins the residual analysis after two native-capability elimination passes.

The question here is not "is this framework written in Swift?" The question is:

> Does this capability justify adding Swift ABI/tooling support to a Rust-first framework because no equally capable public C/Objective-C/Rust route exists?

## Priority model

- **P0:** common capability with no supported lower-level substitute; blocks normal app functionality.
- **P1:** valuable system capability with no clean lower-level substitute; worth dedicated ABI work.
- **P2:** useful but narrower, entitlement-limited, UI-heavy, or replaceable for many apps.
- **P3:** niche or mostly Swift-specific convenience; defer.

## Ranked residual table

| Framework/API | Why it survives native elimination | ABI/tooling shape | Priority | Current recommendation |
|---|---|---|---|---|
| StoreKit 2 | Modern supported IAP API uses Swift structs, async and AsyncSequence; old ObjC payment queue is deprecated | structs, enums, generics, async, AsyncSequence, verification values | P0 | First serious Swift-ABI target |
| App Intents | System discovery is defined through Swift protocol-conforming types and macros/parameters | protocols, metadata, macros, async, compiler-generated discovery | P0/P1 | First compiler/tooling target; research separately from runtime ABI |
| Foundation Models | New on-device model API is strongly Swift-native; structured generation uses macros | Swift classes + async + protocols + macros + generated schema/conformance | P1 | High-value modern target; plain text generation may be easier than @Generable |
| Translation | Direct non-UI TranslationSession exists but methods are Swift async | Swift class/value types/async | P1 | Good contained Swift-ABI target after StoreKit |
| FinanceKit | Financial APIs use Swift async/value types and managed entitlement | async, structs/enums/history sequences; entitlement-gated | P1/P2 | Valuable but access-limited; research after broadly available APIs |
| ActivityKit | Generic `Activity<Attributes>`, Codable attributes, async updates and AsyncSequence | generics, protocols, Codable values, async/AsyncSequence | P2 | Non-UI lifecycle may merit ABI support; rendering is WidgetKit/SwiftUI-heavy |
| GroupActivities / SharePlay | App type conforms to `GroupActivity`; sessions delivered asynchronously | protocols, Codable, generics, async sessions | P2 | Real capability gap; later than StoreKit/App Intents |
| FamilyControls / DeviceActivity / ManagedSettings | Core values are Swift structs/tokens; privacy picker is SwiftUI; reporting uses result builders/AsyncSequence | structs, opaque tokens, protocols, SwiftUI, result builders, AsyncSequence | P2 | Important for parental-control apps, but entitlement/specialized |
| WidgetKit | Widget/provider protocols and generic timelines; rendering is SwiftUI | protocols, generics, result builders, App Intents, SwiftUI | P2/P3 | Low core priority because UI-heavy; study system-facing provider metadata separately |
| Journaling Suggestions | Primary picker is generic SwiftUI view; selected data uses Swift structs/protocols | SwiftUI generics, async closure, structs/protocols; entitlement | P3 | Defer; UI-heavy and niche |
| TipKit | Tip content/rules use Swift protocols/value types; UIKit presentation classes exist | protocols, async sequences/Observation; some ObjC/UIKit display classes | P3 | Low priority; can build Rust-native hints or use UIKit pieces |
| SwiftData | `@Model` macro generates persistence/Observation conformances | macros, protocols, generated metadata, actors | D/P3 | Do not use as framework persistence foundation; Core Data/Rust storage already solve capability |
| Observation | `@Observable` macro generates state-tracking machinery | macro/compiler transformation | D | Rust-owned state should remain Rust |
| Combine | generic Publisher/Subscriber abstraction | generics, associated types, closures | D | Use native callbacks/Futures/channels; not a capability gap |
| Core Transferable | Swift-focused protocol/composed representations | protocols, opaque result types/generics | P3/D | Native pasteboard/share/document APIs already cover most capability needs |
| WeatherKit Swift API | Swift service/structs/async | Swift values/async | D for ABI | Apple provides REST API; use REST unless local Swift API gives unique required behavior |
| CryptoKit | Swift generic/value API for crypto | protocols/generics/value types | D for most | Prefer RustCrypto/direct Rust and Security/SecKey for system key material; research Secure Enclave gaps separately |
| SwiftUI | declarative UI generic graph | protocols, generics, result builders, opaque types, runtime graph | D/P3 | UIKit/objc2 is the framework's UI path |

## 1. StoreKit 2 — highest-priority runtime ABI target

Apple's current StoreKit `Product` and `Transaction` are Swift structures. Transaction monitoring exposes `Transaction.Transactions`, which conforms to `AsyncSequence`.

The legacy `SKPaymentQueue` Objective-C API is deprecated/no longer supported, so unlike many other frameworks there is a real long-term capability reason to support the Swift surface.

### ABI features likely required

At minimum research:

- calling static Swift methods;
- returning/borrowing resilient or frozen Swift structs;
- Swift `String`;
- Swift arrays/collections used by product lookup;
- enums and optionals;
- `VerificationResult<T>` generic values;
- async function calling convention;
- task/continuation completion;
- `AsyncSequence` iteration;
- errors;
- ownership/destruction of returned values.

### Important optimization question

Do not automatically implement a general Swift async executor. Determine whether a narrow thunk can invoke StoreKit async entrypoints and resume into Rust callbacks/Futures while Apple's Swift concurrency runtime performs the actual StoreKit async work.

### References

https://developer.apple.com/documentation/storekit/product
https://developer.apple.com/documentation/storekit/transaction
https://developer.apple.com/documentation/storekit/transaction/transactions
https://developer.apple.com/documentation/storekit/skpaymentqueue?language=objc

## 2. App Intents — runtime ABI is only part of the problem

Apple defines app actions as types conforming to `AppIntent`. Parameters use the `@Parameter` macro.

This strongly suggests that "call a Swift method" is insufficient. The system must discover intent types and their parameter/result metadata.

### Research must separate

**Runtime problem**
- protocol witness/conformance;
- Swift values;
- async `perform`;
- errors/results.

**Build/discovery problem**
- macro expansion output;
- generated metadata;
- app/extension registration artifacts;
- any build-tool extraction step used by Xcode;
- whether equivalent metadata can be emitted without compiling Swift source.

### Framework rule

Do not fake private registration mechanisms. Any Rust-produced metadata must correspond to supported public build/runtime behavior.

### References

https://developer.apple.com/documentation/appintents/appintent
https://developer.apple.com/documentation/appintents/app-intents

## 3. Foundation Models

Foundation Models is a genuine modern Swift-native capability.

`LanguageModelSession` is a Swift class representing model context. Structured generation uses the `Generable` protocol and `@Generable` / `@Guide` macros.

### Split the problem

**Phase A candidate:** plain text prompting/session response without custom generated model types.

This may require:
- constructing Swift session/configuration values;
- Swift String;
- async calls;
- returned response/value ownership.

**Phase B:** structured guided generation.

This adds:
- protocol conformances;
- generated schemas;
- macro-equivalent code;
- potentially substantial generic metadata.

### Recommendation

Research whether the non-`Generable` text/session surface can be reached with a much smaller Swift ABI subset before considering macro-compatible structured generation.

### References

https://developer.apple.com/documentation/foundationmodels/languagemodelsession
https://developer.apple.com/documentation/foundationmodels/generable

## 4. Translation

`TranslationSession` can now be initialized directly in non-UI contexts, which is important: the capability is not inherently tied to SwiftUI.

However, preparation/translation methods are Swift async APIs and use Swift Foundation value types such as `Locale.Language`.

### Why this is attractive

Translation may be a cleaner ABI test than App Intents because it appears to be mostly runtime calling/values/async rather than compiler registration metadata.

### Recommendation

High-priority post-StoreKit proof target.

Reference:
https://developer.apple.com/documentation/translation/translationsession

## 5. FinanceKit

`FinanceStore` is a Swift class whose authorization and query APIs are async and return Swift model values/history types.

FinanceKit requires a managed entitlement and organization-level eligibility for financial-data access.

### Recommendation

Technically meaningful Swift-ABI target, but lower deployment priority because entitlement availability limits general framework usage.

References:
https://developer.apple.com/documentation/financekit
https://developer.apple.com/documentation/financekit/financestore

## 6. ActivityKit

ActivityKit is genuinely Swift-centric:

- app-defined attributes conform to `ActivityAttributes`;
- `Activity<Attributes>` is generic;
- activity content uses associated `ContentState`;
- updates are async;
- observation uses async sequences.

The Objective-C-language documentation still displays the generic Swift-style API rather than an `@interface`, which is a useful warning that this is not merely a renamed ObjC class.

### Architectural split

The framework could potentially support **Live Activity lifecycle/data operations** before solving rendering.

Actual Live Activity UI is tied strongly to WidgetKit/SwiftUI and can remain a later problem.

References:
https://developer.apple.com/documentation/activitykit/activity
https://developer.apple.com/documentation/activitykit/activityattributes

## 7. GroupActivities / SharePlay

Apps adopt the `GroupActivity` protocol on custom Codable data types. Group sessions are delivered asynchronously.

This is a real feature with no obvious Objective-C-equivalent route discovered in the native passes.

### Likely ABI needs

- protocol conformance/witnessing;
- Codable-compatible Swift representation or a supported way to bridge serialized activity data;
- async session acquisition;
- generic group-session types.

Reference:
https://developer.apple.com/documentation/groupactivities/groupactivity

## 8. FamilyControls / DeviceActivity / ManagedSettings

These APIs deserve their own later vertical research because they mix several Swift-native mechanisms:

- `FamilyActivitySelection` is a Swift struct containing opaque application/category/domain tokens;
- `FamilyActivityPicker` is a SwiftUI view;
- `DeviceActivityCenter` is a Swift value type;
- report APIs use result builders and asynchronous result sequences;
- `ManagedSettingsStore` uses Swift value/settings types and exposes Combine-published state;
- access is privacy/entitlement-sensitive.

### Recommendation

Do not make this part of the initial general ABI layer. Treat it as a specialized capability package after the foundational ABI work proves itself.

References:
https://developer.apple.com/documentation/familycontrols/familyactivityselection
https://developer.apple.com/documentation/familycontrols/familyactivitypicker
https://developer.apple.com/documentation/deviceactivity
https://developer.apple.com/documentation/managedsettings/managedsettingsstore

## 9. WidgetKit

Apple describes widget creation around the `Widget` protocol, generic timeline/provider types, App Intents configuration, and SwiftUI presentation.

Because this framework intentionally treats UI as lower priority, full WidgetKit support should not drive the first Swift ABI implementation.

### Later research split

- Can WidgetCenter/reload management be called independently?
- What build metadata identifies a Rust-authored widget extension?
- Can provider/timeline functionality be implemented without arbitrary SwiftUI?
- Is rendering unavoidably SwiftUI for supported third-party widgets?

Reference:
https://developer.apple.com/documentation/widgetkit

## 10. Journaling Suggestions

Apple's primary API requires declaring `JournalingSuggestionsPicker` using SwiftUI. The picker is a generic SwiftUI `View` and its completion is async.

The framework also has selected-value structs/protocols and notification-related configuration.

### Recommendation

Defer. This is exactly the kind of niche, UI-heavy Swift surface that should not shape the general ABI design.

References:
https://developer.apple.com/documentation/journalingsuggestions
https://developer.apple.com/documentation/journalingsuggestions/journalingsuggestionspicker

## 11. TipKit

TipKit's logical model uses the Swift `Tip` protocol, rules/value types, async update streams, and Observation. However, Apple also provides UIKit presentation classes such as `TipUICollectionViewCell` and popover/view-controller integration.

### Recommendation

Low priority. If an app merely needs contextual hints, a Rust-native eligibility/state model plus UIKit presentation is often sufficient. Only implement TipKit ABI if actual system TipKit behavior is valuable.

Reference:
https://developer.apple.com/documentation/tipkit

## 12. SwiftData and Observation

The SwiftData `@Model` macro expands model classes to provide `PersistentModel`, `Observable`, backing storage, schema metadata and generated access/mutation machinery.

Observation similarly depends on macro-generated behavior; adopting the protocol alone does not implement observation.

These are not fundamental iOS capabilities. Persistence and state management are already available through Rust, SQLite, Core Data and other native routes.

### Recommendation

**Do not spend early ABI/tooling effort here.** Interoperability can be revisited only if a consuming app specifically needs to share SwiftData models with Swift code.

References:
https://developer.apple.com/documentation/swiftdata/model()
https://developer.apple.com/documentation/observation/observable

## 13. Combine

Combine's `Publisher<Output, Failure>` and Subscriber ecosystem is a Swift generic reactive abstraction.

Underlying Apple capability sources frequently already expose delegates, callbacks or Blocks.

### Recommendation

Do not reproduce Combine for framework internals. Rust callbacks/channels/Futures can provide equivalent application architecture without crossing Swift ABI.

Reference:
https://developer.apple.com/documentation/combine

## 14. Core Transferable

Apple explicitly describes Core Transferable as a "modern, Swift-focused" API. `Transferable` is a protocol whose representation uses generic/opaque composition.

The underlying user capabilities—copy/paste, drag/drop, share sheets, documents—already have older native framework routes.

### Recommendation

Defer unless interoperability with an API specifically requiring `Transferable` becomes necessary.

Reference:
https://developer.apple.com/documentation/coretransferable

## 15. WeatherKit

Apple explicitly provides:
- a Swift API on Apple platforms;
- a REST API usable on any platform.

### Recommendation

Classify WeatherKit Swift ABI as **D** initially. A Rust HTTP client can use the public REST service. Revisit only if the in-process API provides a required capability that REST cannot.

Reference:
https://developer.apple.com/weatherkit/

## 16. CryptoKit

CryptoKit is heavily Swift-generic/value oriented (`HashFunction`, `SHA256`, key types, etc.).

But cryptography is unusually easy to keep Rust-native:
- RustCrypto and other audited Rust libraries can cover general hashing/ciphers/signatures;
- Keychain/Security is already direct C ABI;
- Security/SecKey should be investigated first for system/secure-hardware key operations.

### Secure Enclave caveat

CryptoKit exposes `SecureEnclave` and modern algorithms. Before declaring any Secure Enclave operation a Swift-ABI requirement, research the corresponding Security/SecKey public APIs and algorithm coverage.

References:
https://developer.apple.com/documentation/cryptokit
https://developer.apple.com/documentation/cryptokit/secureenclave

## Revised Swift-ABI research order

Based on actual capability gaps rather than API popularity:

1. **StoreKit 2** — strongest combination of broad usefulness and lack of supported ObjC replacement.
2. **Swift ABI fundamentals needed by StoreKit** — strings, values, enums, optionals, generic results, async, AsyncSequence.
3. **Translation** — contained non-UI async Swift API; useful second proof.
4. **App Intents discovery/build metadata** — study compiler/tooling independently of runtime ABI.
5. **Foundation Models plain-session API** — then structured `Generable` only if necessary.
6. **ActivityKit non-UI lifecycle/data**.
7. **GroupActivities**.
8. **FinanceKit** subject to entitlement availability.
9. **FamilyControls/DeviceActivity/ManagedSettings**.
10. **WidgetKit system-facing pieces**.
11. **TipKit/Core Transferable/Journaling Suggestions**.
12. **SwiftData/Observation/Combine/SwiftUI** only for explicit interoperability requirements.

## Important design consequence

The Swift ABI layer should **not** be designed by trying to cover Swift generically.

It should grow from concrete API requirements in this order:

```text
StoreKit requirement
  -> implement minimum ABI primitive
  -> test it
  -> keep primitive reusable

Translation requirement
  -> add only missing primitive
  -> test it

App Intents requirement
  -> add build/discovery support separately
```

This keeps the framework from accidentally becoming a partial Swift compiler/runtime project.

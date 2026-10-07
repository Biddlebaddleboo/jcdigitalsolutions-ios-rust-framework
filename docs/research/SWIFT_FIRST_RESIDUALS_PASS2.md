# Modern Swift-First Capability Residuals — Pass 2

Research date: 2026-10-07

This pass distinguishes modern Swift-only frameworks by the **kind of interoperability they require**, rather than simply labeling them all "Swift ABI."

## Executive classification

There are now four materially different residual categories:

### A. Call existing Apple Swift types/functions

Examples:
- StoreKit 2
- Translation
- basic Foundation Models text sessions
- FinanceKit queries

These primarily require:
- Swift calling convention;
- Swift value ownership/layout;
- async/throws;
- standard-library values.

This is the most tractable category.

### B. Rust must define a Swift protocol-conforming domain type

Examples:
- ActivityKit `ActivityAttributes`
- GroupActivities `GroupActivity`

These additionally require:
- Rust-emitted Swift type metadata;
- conformance descriptors;
- witness tables;
- associated types;
- Codable/Hashable or equivalent conformances.

This is substantially harder than simply calling Apple Swift code.

### C. Compiler-generated static metadata is part of the system contract

Primary example:
- App Intents

These need category B plus a build/discovery pipeline.

### D. Macro/result-builder-heavy convenience layers

Examples:
- Foundation Models `@Generable`
- SwiftUI
- some WidgetKit surfaces

These should usually be decomposed so the framework first targets a simpler runtime API.

---

# Foundation Models

## Finding

Foundation Models is worth supporting, but its API has a **simple first layer and a much harder structured-generation layer**.

Apple exposes:

```swift
class LanguageModelSession
```

and supports ordinary text responses.

Apple's documentation shows:

```swift
let response = try await session.respond(to: prompt)
```

The session records transcript/context and performs on-device generation.

Reference:
https://developer.apple.com/documentation/foundationmodels/languagemodelsession

## Plain text generation first

The framework should prioritize the API shape that ultimately yields:

```swift
LanguageModelSession.Response<String>
```

This still involves:
- Swift class construction;
- Swift String/Prompt;
- generic `Response<String>`;
- async/throws;
- GenerationOptions;
- ownership of transcript/result values.

But it does **not inherently require a Rust-defined custom Generable type**.

Classification: **S2 / category A**.

## Avoid PromptBuilder first

Some overloads use:

```swift
@PromptBuilder prompt: () throws -> Prompt
```

Result-builder closure construction adds unnecessary ABI/tooling complexity for the first implementation.

Apple also provides `respond(to:...)`-style entrypoints that accept prompt values directly.

Research should identify the simplest public concrete overload in the installed SDK and prefer it.

## Structured generation is a separate phase

Apple exposes:

```swift
func respond<Content>(
    ...,
    generating: Content.Type,
    ...
) async throws -> Response<Content>
where Content: Generable
```

and recommends macro-generated Swift model types for guided generation.

This introduces:
- Rust-defined Swift type metadata;
- `Generable` conformance/witnesses;
- generated schema representation;
- macro-equivalent compile-time functionality;
- generic response values.

Classification: **S2/S3-like tooling complexity**, but unlike App Intents this is not needed merely for system discovery.

### Better intermediate target

Apple also exposes schema-driven generation returning `GeneratedContent`.

Research this before recreating `@Generable`. If Rust can construct a public `GenerationSchema` and read `GeneratedContent`, the framework may support structured generation without pretending Rust structs are Swift macro-generated Generable types.

References:
https://developer.apple.com/documentation/foundationmodels/languagemodelsession
https://developer.apple.com/documentation/foundationmodels/languagemodelsession/response

## Concurrency constraint

Apple documents that a session handles only one request at a time and can produce a runtime error for concurrent requests on the same session.

The Rust wrapper should encode or enforce this rather than adding a hidden global lock.

Potential approaches:
- `&mut self`-style exclusive request API where practical;
- explicit session-busy error/state;
- one in-flight future/token per session.

Do not hide concurrency behind `Arc<Mutex<LanguageModelSession>>` by default.

## Performance

On-device model inference dominates ABI-call overhead.

The important performance concerns are therefore:
- avoiding duplicate prompt/string copies;
- avoiding unnecessary transcript conversion;
- not serializing result values through JSON merely because Rust is the caller;
- preserving direct Apple session reuse.

## Priority

**P1 after StoreKit/Translation ABI primitives.**

Foundation Models text generation is more broadly valuable than reproducing SwiftUI or SwiftData machinery.

---

# ActivityKit

## Finding

ActivityKit's non-UI lifecycle is Swift-only enough to deserve eventual support, but it requires **Rust-defined Swift-compatible domain types**.

Apple defines:

```swift
protocol ActivityAttributes : Decodable, Encodable
```

with:

```swift
associatedtype ContentState :
    Decodable,
    Encodable,
    Hashable
```

Then Live Activities are represented as:

```swift
Activity<Attributes>
```

References:
https://developer.apple.com/documentation/activitykit/activityattributes
https://developer.apple.com/documentation/activitykit/activity

## Starting a Live Activity

The modern API includes:

```swift
static func request(
    attributes: Attributes,
    content: ActivityContent<Attributes.ContentState>,
    pushType: PushType?
) throws -> Activity<Attributes>
```

This requires the application to provide a concrete `Attributes` type satisfying the Swift protocol and associated-type requirements.

Reference:
https://developer.apple.com/documentation/activitykit/activity/request(attributes:content:pushType:)

## Why this is harder than StoreKit

StoreKit mainly asks Rust to **consume Apple-defined Swift types**.

ActivityKit asks Rust to define an application-specific type that Swift generic code can use.

A pure-Rust implementation likely needs:
- nominal Swift type descriptor;
- type metadata;
- value witness table;
- `ActivityAttributes` conformance descriptor/witness table;
- associated `ContentState` metadata;
- Codable conformances;
- Hashable conformance for ContentState;
- field layout/ownership;
- generic metadata instantiation for `Activity<RustAttributes>`.

Classification: **S2, category B**.

## Serialization insight

ActivityKit requires attributes/content state to be Codable because the system needs serializable activity data.

A possible implementation strategy is to generate Swift-compatible Codable witness machinery for a constrained set of Rust data shapes.

However, before building full Codable interoperability, inspect whether ActivityKit's generic ABI eventually calls public encoder/decoder witnesses directly or whether there is a supported lower-level representation that can accept pre-encoded data.

Do not assume such a bypass exists.

## UI can remain separate

Live Activity presentation is supplied through WidgetKit/SwiftUI.

That is a separate problem from:
- request;
- update;
- end;
- push token handling;
- activity state observation.

Because this project deprioritizes UI-heavy Swift surfaces, research should first determine whether the **lifecycle/data half** can be supported independently.

A consuming app may initially need a separate presentation solution until a zero-Swift widget path is proven.

## Updating

ActivityKit exposes async update operations on `Activity<Attributes>`.

Modern APIs use `ActivityContent<ContentState>`.

This reuses the ABI primitives needed for:
- async methods;
- generic values;
- Rust-defined Swift-compatible values.

## Observation

Activity state/content/push-token streams use async sequences.

As with StoreKit, do not implement universal AsyncSequence support first. Bridge concrete ActivityKit iterator types, then generalize only proven common machinery.

## Content-size invariant

Apple documents that encoded dynamic ContentState data cannot exceed 4 KB.

A future Rust wrapper should validate/propagate this domain limitation and must not confuse it with a framework memory/transport limit.

## Priority

**P2.**

ActivityKit is valuable, but the Rust-defined-protocol-type requirement means it should follow the foundational Swift ABI work.

---

# GroupActivities / SharePlay

## Finding

SharePlay has the same key escalation as ActivityKit: applications define a custom Swift-compatible activity type.

Apple's `GroupActivity` protocol inherits Codable requirements and exposes activity metadata, activation, and asynchronous session discovery.

Reference:
https://developer.apple.com/documentation/groupactivities/groupactivity

## Session model

Apple exposes:

```swift
final class GroupSession<ActivityType>
where ActivityType : GroupActivity
```

and:

```swift
static func sessions() -> Self.Sessions
```

where `Sessions` is an asynchronous sequence of `GroupSession<Self>`.

References:
https://developer.apple.com/documentation/groupactivities/groupsession
https://developer.apple.com/documentation/groupactivities/groupactivity/sessions()

## ABI requirements

Likely:
- Rust-defined Swift nominal type;
- `GroupActivity` conformance;
- Codable conformance;
- static/activity metadata requirements;
- generic `GroupSession<RustActivity>`;
- async activation;
- AsyncSequence session delivery;
- eventual messenger/message Codable types.

Classification: **S2, category B**.

## Why it should not precede ActivityKit

It adds:
- custom type conformance;
- async session discovery;
- networking/session state;
- transferred Codable messages.

ActivityKit is probably a cleaner first experiment for Rust-defined Swift generic-domain types.

## Priority

**P2 after ActivityKit.**

---

# FinanceKit

## Finding

FinanceKit is technically closer to Translation than to ActivityKit.

Apple centers data access on:

```swift
class FinanceStore
```

with async methods including:

```swift
func authorizationStatus() async throws -> AuthorizationStatus
func requestAuthorization() async throws -> AuthorizationStatus
```

Reference:
https://developer.apple.com/documentation/financekit
https://developer.apple.com/documentation/financekit/financestore

## ABI shape

Likely category A:
- Swift class;
- async throws;
- Swift structs/enums representing accounts/transactions/history;
- collection/history iteration.

No application-defined Swift protocol type is obviously required for basic querying.

## Deployment limitation

Apple requires a managed FinanceKit entitlement and organization-level eligibility for financial-data access.

Therefore the framework may technically support it but most applications cannot simply enable it.

This reduces priority despite manageable ABI shape.

## Priority

**P2 due entitlement gating**, even though ABI difficulty may be closer to P1.

---

# CryptoKit / Secure Enclave check

## Finding

CryptoKit itself is not a priority Swift-ABI target because Rust has strong native cryptographic options and Security provides C APIs.

Apple's Security framework exposes `SecKey` and Keychain APIs.

Reference:
https://developer.apple.com/documentation/security/seckey

## Research rule

Before using Swift ABI for any CryptoKit operation:

1. determine whether audited Rust crypto covers it;
2. determine whether Security/SecKey covers the required hardware/system-key operation;
3. only classify the remaining Secure Enclave/key type operation as Swift-only if no equivalent public Security API exists.

## Why

Crypto should not be routed through Swift merely to mirror Apple's preferred Swift syntax.

Rust implementations can often provide:
- lower abstraction cost;
- stronger portable reuse;
- easier test vectors;
- no Swift ABI dependency.

System-protected key material remains a Security-framework concern when available.

Classification: **D by default; gap-driven research only.**

---

# Revised interoperability layers

The research now supports an explicit capability ladder:

## Layer 1 — Consume public Swift types

Target first.

Required by:
- StoreKit 2;
- Translation;
- Foundation Models simple text;
- FinanceKit.

Needed primitives:
- symbol resolution/linking;
- Swift calling convention;
- strings/arrays/sets/value metadata;
- error ABI;
- async;
- concrete AsyncSequence adapters where needed.

## Layer 2 — Define Rust-backed Swift-compatible types

Target later.

Required by:
- ActivityKit;
- GroupActivities;
- potentially structured Foundation Models.

Adds:
- nominal descriptors;
- value witness tables;
- protocol conformance descriptors;
- witness tables;
- associated types;
- possibly Codable/Hashable generation.

## Layer 3 — Emit compiler/build metadata

Target only when necessary.

Required by:
- App Intents;
- potentially other system-discovery frameworks.

Adds:
- build-time extraction inputs;
- Xcode processor integration;
- static metadata packaging;
- distribution validation.

## Layer 4 — Declarative UI/compiler DSL emulation

Lowest priority.

Includes:
- arbitrary SwiftUI;
- WidgetKit rendering graphs;
- result builders where no simpler public API exists.

Do not allow Layer 4 requirements to contaminate Layers 1–2.

# Recommended order after this pass

1. Prove Layer 1 calling convention and ownership.
2. StoreKit 2 product lookup/purchase.
3. Translation single-string translation.
4. Foundation Models plain text response.
5. Extract reusable async/value primitives.
6. Prove one minimal Layer 2 Rust-defined Swift value + protocol conformance.
7. ActivityKit lifecycle/data prototype.
8. GroupActivities.
9. App Intents Layer 3 metadata research/prototype.
10. FinanceKit when an entitlement-bearing test environment is available.
11. UI/result-builder-heavy surfaces only on demand.

This order maximizes useful iOS capability while minimizing speculative Swift reimplementation.

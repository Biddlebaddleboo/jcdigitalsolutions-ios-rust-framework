# Swift Language and Library Abstractions — Framework Disposition

Research date: 2026-10-07

This document separates **Swift application-programming abstractions** from **Apple platform capabilities**.

The framework should not implement Swift abstractions merely because Apple's Swift examples use them.

## Summary

| Swift abstraction | Is it a unique iOS capability? | Rust-framework disposition |
|---|---|---|
| `Task` / structured concurrency | No | Use Rust async/callbacks; interop only at Swift-only API boundaries |
| `AsyncStream` / `AsyncSequence` | No | Bridge concrete Apple streams into Rust streams/callbacks; don't reproduce generally |
| Combine | No | Use Rust channels/callbacks/Futures/operators where needed |
| Observation / `@Observable` | No | Keep Rust state Rust-native |
| Codable | No | Use serde / Rust codecs for Rust models; implement Swift conformance only when an Apple generic API requires it |
| JSONEncoder/Decoder | No | Use Rust JSON or native JSONSerialization as appropriate |
| Result | No | Map to Rust `Result`; only decode Swift Result ABI when a public Apple API physically returns it |
| Swift collections | Usually no | Keep Rust collections internally; construct Swift collection only at Swift ABI boundary |
| Swift property wrappers/macros | No | Do not reproduce unless a system framework's compiler metadata requires equivalent output |
| Swift actors | No | Use Rust ownership/concurrency; interop with Apple-defined actors only when required |
| Sendable | No | Use Rust Send/Sync semantics internally; emit Swift conformance only for Rust-defined Swift types when required |

# Task / Swift structured concurrency

Apple defines:

```swift
@frozen
struct Task<Success, Failure>
```

A Task represents asynchronous work and cancellation/result interaction.

Reference:
https://developer.apple.com/documentation/swift/task/

## Decision

Do not model ordinary Rust application async work as Swift Task.

Use:
- Rust Futures;
- callbacks;
- platform dispatch/executor primitives where appropriate;
- a minimal Rust executor only if the application already needs one.

At a Swift-only API boundary:
- let Apple's Swift concurrency runtime execute the public async API;
- bridge result/cancellation into Rust.

The framework does not need to expose Task to normal Rust app code.

# AsyncStream / AsyncSequence

Apple describes `AsyncStream` as a way to adapt callbacks/delegates into async-await iteration.

Reference:
https://developer.apple.com/documentation/swift/asyncstream

Rust already has equivalent architectural options:
- callbacks;
- channels;
- streams;
- async iterators from ecosystem crates;
- custom polling state.

## Decision

Do not reproduce AsyncStream as an application abstraction.

For Apple APIs exposing AsyncSequence:
1. bridge the specific concrete sequence;
2. deliver items as a Rust callback or Stream;
3. preserve cancellation/backpressure semantics that actually matter;
4. generalize only if several APIs prove the same ABI primitive reusable.

# Combine

Apple explicitly describes Combine as a declarative Swift API for asynchronous values over time.

The core abstraction is:

```swift
protocol Publisher<Output, Failure>
```

with generic associated types and subscriber demand.

References:
https://developer.apple.com/documentation/combine
https://developer.apple.com/documentation/combine/publisher/

## Decision

Combine is not a framework dependency.

Underlying Apple sources often already expose:
- delegate callbacks;
- Blocks;
- NotificationCenter;
- timers;
- URLSession.

Rust can provide equivalent transformation/composition using Rust-native abstractions when useful.

Only add Combine ABI interoperability for an Apple API that **only** exposes a required value through a Publisher and offers no ordinary accessor/callback/stream alternative.

Do not implement Combine generics/operators for parity.

# Observation

Apple explicitly states that merely conforming to `Observable` does not add observation behavior; the `@Observable` macro generates the functionality.

References:
https://developer.apple.com/documentation/observation/observable
https://developer.apple.com/documentation/observation/observable()

## Decision

Rust application state should not adopt Observation.

Use:
- direct mutable state;
- callbacks;
- subscriptions;
- channels;
- ECS/store-specific mechanisms as appropriate.

For SwiftUI interop, Observation may eventually be relevant, but SwiftUI itself is not a core UI target.

# Codable

Apple defines:

```swift
typealias Codable = Decodable & Encodable
```

Reference:
https://developer.apple.com/documentation/swift/codable

## Decision for Rust-owned data

Use Rust-native serialization:
- serde where appropriate;
- hand-written binary protocols where performance demands it;
- direct database encoding;
- domain-specific codecs.

Do not route Rust models through Swift Codable just because Apple sample code does.

## Exception

Some Swift-only Apple generic APIs require an application-defined type to conform to Codable, for example:
- ActivityKit attributes/content state;
- GroupActivity data.

In those cases, Codable becomes part of the **Swift ABI conformance contract**, not the application's general serialization architecture.

Implement only the constrained witness behavior needed by those APIs.

# JSONEncoder / JSONDecoder

Apple's `JSONEncoder` is Swift generic over Encodable values.

Reference:
https://developer.apple.com/documentation/foundation/jsonencoder

Foundation separately exposes Objective-C `NSJSONSerialization`.

Reference:
https://developer.apple.com/documentation/foundation/jsonserialization?language=objc

Rust also has mature JSON libraries.

## Decision

For Rust data:
- prefer Rust serialization directly.

For Foundation object graphs:
- NSJSONSerialization is available through Objective-C.

Do not introduce Swift JSONEncoder ABI into the core.

# Result

Apple defines:

```swift
@frozen enum Result<Success, Failure>
```

Reference:
https://developer.apple.com/documentation/swift/result

Rust has a direct language-native conceptual equivalent:

```rust
Result<T, E>
```

## Decision

Framework public APIs should expose Rust `Result`.

If an Apple Swift API physically returns a Swift Result-like enum:
- decode that concrete result at the ABI boundary;
- immediately map it into Rust Result;
- do not expose Swift Result semantics beyond the boundary.

# Swift collections

Normal Rust code should use:
- Vec;
- slices;
- HashMap/BTreeMap;
- HashSet/BTreeSet;
- arrays;
- domain-specific compact containers.

Only construct Swift:
- Array;
- Set;
- Dictionary;

when a concrete Swift-only public Apple API requires them.

For Foundation-bridgeable element types, investigate NSArray/NSSet/NSDictionary bridging before implementing native collection layouts.

# Property wrappers

Examples:
- `@Published`;
- App Intents `@Parameter`;
- SwiftUI state wrappers;
- SwiftData property/model wrappers.

## Decision

Property wrappers are source-language/compiler conveniences, not runtime requirements by themselves.

Do not emulate their syntax/semantics generically.

When a framework requires generated metadata:
- determine the actual ABI/build artifact produced;
- generate only that required artifact from Rust tooling if supported.

# Macros

Examples:
- `@Observable`;
- `@Model`;
- `@Generable`;
- App Intents macros.

## Decision

A Swift macro is never automatically a framework requirement.

Ask:
1. what code/metadata does the macro generate?
2. does the system need the generated runtime conformance?
3. does the system need compiler-extracted static metadata?
4. can Rust generate the supported final ABI artifact directly?

Do not implement a Swift macro engine.

# Actors

Apple-defined actors may appear in Swift-only frameworks, for example SecureElementCredential.

Rust application code does not need Swift actor semantics.

## Decision

At an Apple actor boundary:
- respect actor isolation/calling requirements;
- let Apple's concurrency runtime schedule actor work;
- expose a Rust async/callback facade.

Do not build a general Swift actor runtime.

# Sendable

Swift's Sendable captures concurrency-safety constraints within Swift.

Rust has stronger native type-system concepts:
- Send;
- Sync;
- borrowing/ownership.

## Decision

Use Rust Send/Sync internally.

Only emit/consume Swift Sendable-related metadata where a concrete Rust-defined Swift-compatible type must satisfy an Apple protocol constraint.

# Dispatch / queues

Grand Central Dispatch is a public C/Objective-C system capability and can be used directly where it is the correct Apple primitive.

Swift Task is not required merely to schedule work.

A Rust app can choose:
- dispatch queues;
- threads;
- Rust async executor;
- direct callback execution;

based on semantics and performance.

# Framework public API rule

The Rust framework should expose Rust-native idioms:

```rust
Result<T, E>
Option<T>
Future
Iterator/Stream where useful
Vec<T>
&[T]
String/&str
callbacks
```

and translate only at Apple boundaries.

Do **not** expose public Rust APIs shaped around:
- Combine Publisher;
- Swift Task;
- AsyncStream;
- Observable;
- Codable;
- Swift Result;
- Swift property wrappers.

# Why this matters for maintenance

If the framework copied Swift application architecture, every Swift language/framework evolution would become a maintenance obligation.

By keeping the boundary narrow:

```text
Rust application architecture
     |
native boundary adapters
     |
Apple public API
```

the framework tracks only:
- Apple capability ABI;
- not Apple's preferred Swift coding style.

This significantly reduces long-term maintenance while also preserving performance.

# Runtime-overhead consequence

Avoiding Swift application abstractions also avoids potential:
- extra task creation;
- publisher subscription objects;
- boxed closures;
- observation registrars;
- generic adapter layers;
- duplicate serialization;
- extra reference-counted state.

This does not mean those Swift abstractions are inherently inefficient. It means their cost and complexity are unnecessary when Rust already has equivalent native mechanisms.

# Final classification

## Reimplement in Rust conceptually, not ABI-for-ABI

- Result
- Codable-like serialization
- reactive/event composition
- observation
- task/future management
- streams
- collection operations

## Interoperate only when an Apple API requires them physically

- Swift String
- Array/Set/Dictionary
- Optional/enums
- Swift Error
- async calling convention
- concrete AsyncSequence
- Apple-defined Swift classes/structs
- protocol conformances for Rust-defined types

This distinction should be treated as a permanent architecture rule.

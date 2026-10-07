# Documentation Policy

## Purpose

Documentation is part of the framework implementation.

Codex and other contributors must develop documentation continuously as the codebase grows. Do not defer documentation until after implementation.

## Documentation audiences

The project needs documentation for three different readers:

1. **Application developer**
   - how to use the API;
   - examples;
   - platform support;
   - errors;
   - async/cancellation;
   - performance-relevant behavior.

2. **Framework maintainer**
   - architecture;
   - internal invariants;
   - dependency seams;
   - packed-data layouts;
   - ownership/threading;
   - unsafe contracts;
   - backend design.

3. **Executor/reviewer**
   - why a design exists;
   - what was benchmarked;
   - Apple parity evidence;
   - known gaps;
   - validation commands.

Do not mix these audiences into one giant document when separate focused docs are clearer.

## Public API documentation

Every meaningful public Rust item should have useful rustdoc.

Document:
- purpose;
- semantics;
- errors;
- ownership/lifetime if non-obvious;
- thread affinity;
- cancellation;
- platform availability;
- complexity/cost when important;
- native escape behavior;
- examples.

Prefer examples that compile in CI where practical.

Do not expose internal packing details in ordinary public docs unless developers need them for ABI/serialization compatibility.

## Capability documentation

Each capability should eventually have a focused guide describing:

- what is portable;
- what is platform-specific;
- supported backends;
- availability/permissions/entitlements;
- high-level usage;
- lifecycle;
- errors;
- async/cancellation;
- native escape hatch;
- performance model;
- dependency/backend implementation choices;
- parity status if Rust replaces an Apple implementation.

## Architecture documentation

Update architecture docs in the same change when modifying:
- crate/module boundaries;
- portable contracts;
- backend selection;
- stable C ABI;
- ownership;
- async model;
- no_std guarantees;
- dependency substitution;
- Swift ABI approach;
- unsafe assumptions;
- public compatibility guarantees.

## Internal representation documentation

Compact internal representations should document enough for safe maintenance without leaking them into the public abstraction.

For packed words/tables include:
- field/bit allocation;
- range/capacity;
- sentinel/reserved values;
- overflow/wrap behavior;
- atomicity;
- alignment;
- endian/serialization assumptions if applicable;
- why the layout is performance-motivated.

## Dependency documentation

For each major third-party dependency record:
- why it is used;
- exact surface relied upon;
- types that cross boundaries, ideally none;
- optional features enabled/disabled;
- no_std implications;
- replacement seam;
- runtime/binary cost where material.

The goal is that a future maintainer can replace the dependency without rediscovering the architecture.

## Apple parity documentation

When Rust replaces Apple library functionality, document:
- exact Apple API used as reference;
- supported semantic subset;
- intentional differences;
- OS-version differences;
- parity test location;
- benchmark location;
- why Rust is the default;
- conditions where Apple remains preferred.

## Performance documentation

Performance-sensitive changes should record:
- benchmark scenario;
- input sizes;
- hardware/OS/toolchain;
- build profile;
- comparison baseline;
- measured result;
- allocations/copies where relevant;
- limitations.

Do not preserve marketing-style claims without reproducible evidence.

## Examples

Examples should emphasize the ergonomic facade.

A developer should see:

```rust
let location = Location::current().await?;
```

not internal:
- bit masks;
- Objective-C selectors;
- JNI objects;
- Swift metadata;
- callback registries.

Advanced/native escape examples belong in separate sections.

## Documentation freshness

A change is incomplete when it changes behavior but leaves docs describing the old behavior.

Review documentation during final diff review.

At minimum check:
- rustdoc;
- README/guide;
- architecture docs;
- parity/testing docs;
- performance docs;
- platform availability.

## Research versus normative documentation

Keep exploratory findings in `docs/research/`.

Once a decision becomes an architectural rule, promote it into normative docs such as:
- `AGENTS.md`;
- `docs/ARCHITECTURE.md`;
- `docs/API_DESIGN.md`;
- `docs/PERFORMANCE.md`;
- `docs/TESTING_AND_PARITY.md`.

Research notes may contain uncertainty. Normative docs must state the adopted rule clearly.

## Documentation quality bar

Good documentation should let a smaller implementation model or a new maintainer:
- understand the design without broad repository exploration;
- know which files/modules own an invariant;
- know what must not change;
- reproduce validation;
- understand why a dependency or Apple backend remains;
- distinguish public semantics from internal representation.

Documentation is successful when it reduces rediscovery and makes architectural drift harder.

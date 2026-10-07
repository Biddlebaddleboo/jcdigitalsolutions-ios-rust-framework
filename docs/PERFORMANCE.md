# Performance Policy

## Objective

Using Rust must not impose a meaningful general runtime penalty over the best equivalent native iOS implementation.

The project separates unavoidable Apple cost from avoidable framework cost.

## Unavoidable Apple cost

Examples include:

- `objc_msgSend` for Objective-C methods;
- required retain/release;
- UIKit layout and rendering;
- Core Animation;
- text shaping;
- main-thread serialization;
- Apple-required Block copying;
- system calls;
- network I/O;
- representation conversions required by Apple API contracts.

These are not Rust-framework regressions when an equivalent native implementation pays the same cost.

## Avoidable framework cost

Treat these as optimization bugs unless justified:

- additional retain/release;
- wrapper allocation;
- dynamic dispatch added only by the framework;
- duplicate string/data conversion;
- unnecessary copies;
- callback registry locking;
- global locks;
- duplicate object models;
- unnecessary Objective-C objects;
- serialization;
- repeated selector lookup;
- task/thread creation per trivial operation.

## Baselines

For UIKit work, compare:

1. Objective-C/UIKit;
2. Swift/UIKit;
3. Rust/UIKit through this framework.

SwiftUI may be measured separately, but it is not the zero-overhead UIKit baseline.

## Required benchmark dimensions

Measure, where relevant:

- process launch;
- time to application delegate;
- time to root UI ready/first frame;
- RSS;
- allocations;
- live allocation count;
- retain/release traffic;
- target/action callback latency;
- direct delegate callback latency;
- label/property mutation;
- navigation push/pop;
- list population and scrolling;
- text conversion;
- data conversion;
- request construction;
- network completion path;
- response-body copy count;
- binary size.

## Generated-code inspection

For important hot paths, inspect optimized assembly or LLVM output.

A direct ABI implementation may replace objc2 only when it demonstrably removes cost or solves an ABI limitation.

## Claims

Do not claim "zero overhead" without qualifying the measured surface.

Preferred language:

- "no measured incremental overhead in scenario X";
- "same call topology as Objective-C for operation Y";
- "one extra handle lookup costing Z";
- "allocation-free after setup";
- "copy-free for borrowed path."

Keep raw benchmark methodology and environment reproducible.

## CI

Wall-clock microbenchmarks in CI are advisory only.

Deterministic regressions are better CI gates:

- allocation count;
- copy count;
- presence of hot-path locks;
- callback-storage allocation;
- retain/release surplus;
- code-size deltas where stable.

Physical-device Release measurements are authoritative for performance claims.

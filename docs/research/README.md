# Apple API Research

This directory is the durable research record for deciding how much Swift interoperability this framework actually needs.

## Research rule

For each iOS capability, investigate in this order:

1. Pure Rust implementation.
2. Public C/CoreFoundation/Darwin API.
3. Public Objective-C API, preferably through an existing objc2 framework crate.
4. Thin Rust reconstruction of a Swift convenience/overlay on top of the public native API.
5. Only then classify the remaining surface as genuinely Swift-only.

The goal is to avoid building Swift ABI machinery for capabilities Rust can already access at native or near-native cost.

## Cost classes

- **R0 — Pure Rust:** no Apple language ABI needed.
- **R1 — Direct C ABI:** public C/CoreFoundation/Darwin calls; native function-call boundary.
- **R2 — Objective-C / objc2:** native Objective-C message boundary; expected to match ordinary Objective-C call topology.
- **R3 — Rust overlay:** ergonomic Rust layer over R1/R2; must be allocation/copy/dispatch transparent where possible.
- **S1 — Simple Swift ABI:** genuinely Swift-only, but reachable with narrow stable ABI work.
- **S2 — Complex Swift ABI:** generics, resilient values, protocols/existentials, closures, async/AsyncSequence, actor isolation, etc.
- **S3 — Compiler/tooling integration:** macros, generated registration/discovery metadata, result builders, or other compile-time artifacts are part of the contract.
- **D — Defer/avoid:** a lower-level public route exists or the Swift-specific abstraction adds little value to a Rust-first framework.

## Current documents

- `NATIVE_CAPABILITY_MATRIX.md` — common capabilities that can already avoid Swift.
- Future passes will add focused files for genuine Swift-only residuals and lower-frequency framework families.

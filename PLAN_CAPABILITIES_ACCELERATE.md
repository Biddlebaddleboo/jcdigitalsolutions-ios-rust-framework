# PLAN_CAPABILITIES_ACCELERATE.md — Workstream D59: Bounded Accelerate Vector Addition

## Status

D59 defines no portable Accelerate contract. B65 is an iOS-only partial slice of capability row
060: equal-length single-precision vector addition through the public C `vDSP_vadd` function

## Objective

Expose one safe Rust operation over borrowed `f32` slices that delegates element-wise addition to
Accelerate vDSP. Do not add a portable math facade or claim broad Accelerate coverage

## API and behavior

- `ios-accelerate` exposes `vector_add(&[f32], &[f32], &mut [f32]) -> Result<(), VectorAddError>`
- `VectorAddError::LengthMismatch` reports unequal input/output lengths and
  `VectorAddError::LengthTooLarge` reports a count that cannot fit `vDSP_Length`
- Return a length-mismatch error before the native call if any slice lengths differ; do not modify
  output on that error
- For equal empty slices, return success without calling native code
- For nonempty equal slices, call `vDSP_vadd` with unit stride and the slice length
- The Rust wrapper allocates no buffer, retains no pointers, exposes no Apple types, and requires
  no main-thread proof
- The public API is available only on iOS; no non-iOS result is defined as an Accelerate operation

## Boundaries

- No portable contract is added
- Do not add double-precision, strided, matrix, FFT, vImage, vForce, BNNS, or MPS operations
- Do not claim bitwise equivalence with a Rust scalar loop, Apple parity, numerical certification,
  performance improvement, or a benchmark result
- No permission, Info.plist key, or entitlement is required for this memory-only vector operation

## Validation and handoff

- Compile and strict-Clippy the iOS device and Simulator targets
- Inspect the Release consumer for the exact Accelerate/libSystem imports and `_vDSP_vadd` symbol
- Run rustdoc, formatting, documentation-index, zero-Swift-source, and diff gates
- Do not execute Apple link probes or claim a live-device result; no tests are required for this
  integration slice

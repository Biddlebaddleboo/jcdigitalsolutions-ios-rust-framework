# PLAN_IOS_ACCELERATE.md — Workstream B65: Accelerate vDSP Vector Addition

## Status

The iOS backend adds one safe `f32` vector addition operation through the public Accelerate C API;
the Rust wrapper creates no output buffer of its own. Focused device/Simulator compile, strict
Clippy, and Release link/import gates pass in the root checkout. It is a partial slice of row 060
and adds no portable contract

## SDK and API evidence

- The inspected Xcode 26.6 / iOS SDK 26.5 public `vDSP.h` declares `vDSP_vadd` with two
  `const float *` inputs, `vDSP_Stride` input/output strides, a `float *` output, and
  `vDSP_Length` element count. The header marks it available from iOS 4.0
- Apple's public [`vDSP_vadd` documentation](https://developer.apple.com/documentation/accelerate/vdsp_vadd)
  describes single-precision element-wise vector addition with caller-specified strides
- Rust 1.94.1's arm64 device and Simulator link probes use their supported minimums, iOS 10.0 and
  iOS 14.0. These probe floors are distinct from the API's iOS 4.0 availability declaration; the
  crate sets no deployment target
- `vDSP_Stride` maps to the target C `long` width and `vDSP_Length` maps to target `unsigned long`;
  the iOS arm64 Rust `isize`/`usize` types match those widths
- The backend links `Accelerate.framework` directly and calls the C symbol; it adds no Swift,
  Objective-C, Foundation, or runtime dependency

## Public surface and safety

- `vector_add` borrows two input slices and one output slice; it returns `LengthMismatch` before
  FFI if the three lengths differ and `LengthTooLarge` if the element count cannot fit
  `vDSP_Length`
- `LengthTooLarge` is a defensive conversion result; the supported arm64 targets use equal-width
  `usize` and `vDSP_Length`, so the guard does not reject a representable arm64 slice length
- Equal empty slices return success without an FFI call; nonempty slices call `vDSP_vadd` with all
  strides set to 1 and the exact slice length
- Rust shared and exclusive borrows ensure the output does not alias either input in safe code; no
  raw pointer or native handle escapes the function
- The operation has no retained state and is not main-thread-bound. It makes no guarantee about
  bitwise parity, application performance, or observed runtime results

## Scope and non-goals

- No portable API or non-iOS fallback is added
- No double-precision, strided, matrix, convolution, FFT, vImage, vForce, BNNS, or MPS operation is
  included
- The Rust wrapper allocates no buffer and converts no input data; no image/audio access,
  permission, Info.plist key, or entitlement is included
- No numerical test, live device execution, parity result, or benchmark is claimed

## Validation

See [PLAN_VALIDATION_IOS_ACCELERATE.md](PLAN_VALIDATION_IOS_ACCELERATE.md). Build/link probes are
compiled and inspected but are not executed

# iOS Accelerate vDSP

`ios-accelerate` implements B65 through an iOS-only Rust wrapper. It creates no Rust-owned output
buffer and exposes `vector_add(&[f32], &[f32], &mut [f32]) -> Result<(), VectorAddError>` over
borrowed slices. It calls Apple's public C `vDSP_vadd` with unit strides. All three slice lengths
must match; a mismatch returns
`VectorAddError::LengthMismatch` before the native call and leaves output untouched. A count that
cannot fit `vDSP_Length` returns `VectorAddError::LengthTooLarge`. Equal empty slices return
success without an FFI call

The wrapper exposes no Apple type, pointer, or native handle and needs no main-thread proof. It
adds no portable contract or non-iOS fallback. No other Accelerate operation is included

Apple documents [`vDSP_vadd`](https://developer.apple.com/documentation/accelerate/vdsp_vadd) as
single-precision element-wise vector addition with caller-specified strides. The inspected iOS
26.5 SDK header marks the function available from iOS 4.0. The Rust 1.94.1 arm64 device link probe
uses iOS 10.0 and the Simulator probe uses iOS 14.0; those are toolchain link floors, not the API
availability declaration. The crate sets no deployment target

Compile/link checks do not establish numerical parity, bitwise equality, performance, or live
device behavior. Link probes are inspected, not executed

# iOS Accelerate Vector-Addition C ABI

F22 proposes the opt-in `ios-accelerate` feature and one synchronous C function, `framework_ios_accelerate_vector_add`, over B65's `ios_accelerate::vector_add` API

The C function accepts three `float` arrays and their element counts. Counts must match. For a nonzero equal count on iOS, each input span must name readable, aligned elements and stay valid for the full call; output must name writable, aligned elements for the full call and remain disjoint from both inputs. Input spans may overlap each other. On success, every output element is overwritten; invalid arguments and valid non-iOS calls leave output unchanged. Equal empty arrays are an iOS no-op that returns `FRAMEWORK_STATUS_OK`; valid non-iOS calls return `FRAMEWORK_STATUS_UNSUPPORTED`

The app must keep both inputs immutable and prevent unsynchronized output access for the full call. The wrapper checks count equality, byte-length bounds, non-nullness, alignment, address-range overflow, and output/input overlap, but cannot prove that memory is valid, readable, writable, or live. No pointer is retained after return. The non-iOS stub checks metadata, then returns `FRAMEWORK_STATUS_UNSUPPORTED` without a data-byte read or write

The backend calls Apple's public `vDSP_vadd` with unit stride and is available from iOS 4.0. Focused link probes use iOS deployment minima 10.0 for device and 14.0 for Simulator; these are probe minima, not the API floor. Expected direct C imports are `Accelerate` and `libSystem.B.dylib`; C++ consumers may also import `libc++.1.dylib`, and the native symbol is `_vDSP_vadd`

This is one memory-only single-precision vector operation. It does not add a portable math contract or claim numerical parity, certification, or performance. It does not expose double precision, strided vectors, matrix operations, FFT, vImage, vForce, BNNS, or MPS. See [D59](../../PLAN_CAPABILITIES_ACCELERATE.md), [B65](../../PLAN_IOS_ACCELERATE.md), and [F22](../../PLAN_BINDINGS_COMPLETED_C_ABI.md)

# PLAN_VALIDATION_C_ABI_CALL_OBSERVER.md — G69 CallKit C ABI

## Scope

G69 validates F12's opt-in C ABI for the D76/B72 CallKit snapshot. The API writes an initialized
call count and aggregate state flags to caller-provided outputs; it does not expose native objects,
identifiers, caller data, callbacks, or call control.

## Gate

`sh bindings/c/check-ios-call-observer.sh` passed. It validates feature isolation, locked host,
device, and Simulator checks, strict Clippy, Release archives, C11/C++17 header compile/link,
exported symbol shape, exact imports, and deployment metadata.

The device and Simulator link probes import CallKit, Foundation, `libSystem.B.dylib`, and
`libobjc.A.dylib`; the C++ link additionally uses libc++. Probe minimums are iOS 12.0 on device and
iOS 14.0 on Simulator, distinct from B72's iOS 10.0 API floor. Neither probes nor linked consumers
were executed. Local evidence used Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the
plan's Xcode 27.x baseline.

## Limits

These checks do not establish live call state, CallKit lifecycle behavior, synchronous-call timing,
or a consumer app's runtime behavior.

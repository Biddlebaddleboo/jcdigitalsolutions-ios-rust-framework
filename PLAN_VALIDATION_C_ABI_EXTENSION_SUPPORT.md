# PLAN_VALIDATION_C_ABI_EXTENSION_SUPPORT.md — G114: extension metadata C ABI

## Scope

G114 validates F30's opt-in C wrapper over D96/B77's one-key .appex runtime metadata reader.
It checks fixed error-code mapping, caller-borrowed UTF-8 input, caller-owned output-buffer
lifetime, feature isolation, exact imports, and separate API/link deployment floors

## Gate

`sh bindings/c/check-ios-extension-support.sh` checks the manifest, Rust format, host/device/
Simulator feature graph, strict Clippy, rustdoc, C11/C++17 header syntax, owned-buffer layout,
and error-code values. `sh bindings/c/check-ios-extension-support-link.sh` links Release C11/C++17
host/device/Simulator consumers and audits import sets, exports, required selectors, forbidden
extension-loading surfaces, and minos. Consumers and probes are build-only

## Evidence and limits

On Xcode 26.6 (build 17F113) with iOS SDK 26.5, both focused gates passed in the integrated
checkout. Host C and C++ consumers imported only `libSystem.B.dylib`; device and Simulator C/C++
consumers imported exactly Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. `vtool` reported
minos 12.0 for device and 14.0 for Simulator. These are link-probe settings, distinct from the
backend's iOS 4.0 API floor. The probes and consumers were linked and inspected, not executed; no
passing CI workflow run is recorded

No tests, live bundle read, extension loading, registration, signing, installation, approval,
entitlement, launch, or host compatibility is claimed

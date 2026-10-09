# PLAN_VALIDATION_C_ABI_NATURAL_LANGUAGE.md — G112: Natural Language C ABI

## Scope

G112 validates F29's opt-in wrapper over D54/B59's English contextual-model asset status. The
C API copies one of B59's five Rust status cases to a fixed `uint32_t`; it exposes no object, text,
vector, model handle, or asset request

## Gate

`sh bindings/c/check-ios-natural-language-status.sh` checks feature closure, manifest codes,
Rust format, host/device/Simulator checks, strict Clippy, rustdoc, and C11/C++17 header syntax.
`sh bindings/c/check-ios-natural-language-status-link.sh` builds Release archives and links C11/
C++17 host/device/Simulator consumers, then audits imports, exports, required selectors, forbidden
model operations, minos, and the output-pointer preconditions across Rust, header, and manifest.
No consumer or probe is executed

## Evidence and limits

`sh bindings/c/check-ios-natural-language-status.sh` and
`sh bindings/c/check-ios-natural-language-status-link.sh` passed in the integrated checkout. The
offline feature-scoped check passed and refreshed the root lock. Host C and C++ import
only `libSystem.B.dylib`; device and Simulator C and C++ imports are exactly Foundation,
NaturalLanguage, `libSystem.B.dylib`, and `libobjc.A.dylib`. C++ links use `-nostdlib++` and import
no `libc++`. Measured minos is 17.0 for both device and Simulator, matching the API floor. Export
parity, required symbols/selectors, forbidden-operation checks, host/device/Simulator builds,
strict Clippy, rustdoc, and C11/C++17 links passed

Validation used Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the repository's
Xcode 27.x baseline. No tests or linked consumers/probes were executed; no passing CI workflow run
is recorded. No live Natural Language query, asset download/request, model load, text input, vector
result, or Swift runtime behavior is claimed

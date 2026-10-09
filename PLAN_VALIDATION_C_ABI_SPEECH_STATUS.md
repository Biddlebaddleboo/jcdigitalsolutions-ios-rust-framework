# PLAN_VALIDATION_C_ABI_SPEECH_STATUS.md — G110: Speech status C ABI

## Scope

G110 validates F28's opt-in wrapper over D53/B58's saved Speech authorization-status query. The
surface returns one signed raw status value; it has no permission, audio, recognizer, callback, or
service-availability operation

## Gate

`sh bindings/c/check-ios-speech-status.sh` passed feature isolation, ABI manifest, status mapping,
Rust format, host/device/Simulator checks, strict Clippy, rustdoc, and C11/C++17 header syntax.
`sh bindings/c/check-ios-speech-status-link.sh` passed Release archives and host/device/Simulator
C11/C++17 links, import/export checks, selector and forbidden-API audits, and minos inspection.
Host C/C++ imports are only `libSystem.B.dylib`. Device and Simulator C/C++ imports are exactly
Foundation, Speech, `libSystem.B.dylib`, and `libobjc.A.dylib`. Device minos is 10.0 and Simulator
minos is 14.0; the API floor is iOS 10.0. C++ links use `-nostdlib++`; no `libc++` import is
present

## Evidence and limits

No tests, linked consumers, probes, live authorization read, prompt, audio capture, recognition,
service availability, or recognition-success behavior is claimed. Local evidence used Rust
1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the repository's Xcode 27.x baseline.
No passing CI workflow run is recorded

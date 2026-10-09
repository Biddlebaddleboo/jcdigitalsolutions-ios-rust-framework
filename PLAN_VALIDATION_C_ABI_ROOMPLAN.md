# PLAN_VALIDATION_C_ABI_ROOMPLAN.md — G115: RoomPlan support C ABI

## Scope

G115 gates F31's opt-in scalar wrapper over B61's `RoomCaptureSession.isSupported` result. It
checks the feature edge, output initialization/status mapping, source/header/manifest pointer
preconditions, Rust host/device/Simulator compile, strict Clippy, rustdoc, C11/C++17 header syntax
and links, direct imports, required RoomPlan symbols, and deployment metadata

## Gate

`sh bindings/c/check-ios-roomplan-status.sh` is a non-test compile/static gate. The
`sh bindings/c/check-ios-roomplan-status-link.sh` gate links host, device, and Simulator C/C++
consumers; it checks that host consumers import only libSystem, target consumers import exactly
RoomPlan and libSystem, the public RoomCaptureSession metadata accessor and `isSupported` getter are
present, selected Swift/Objective-C runtime symbols are absent, and minos is 16.0. No consumer or
probe binary is executed and no RoomPlan query is called

## Evidence and limits

Both focused G115 gates passed in the integrated checkout after root refreshed the shared lock.
Host/device/Simulator feature isolation, Rust checks, strict Clippy, rustdoc, C11/C++17 syntax and
links passed on Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5. Host consumers imported
only `libSystem.B.dylib`; device/Simulator consumers imported `RoomPlan` and `libSystem.B.dylib`
with minos 16.0 and both required RoomPlan symbols. No tests, consumers, probes, or RoomPlan API
calls ran; no passing CI workflow run is recorded. The repository Xcode 27.x baseline has not been
met

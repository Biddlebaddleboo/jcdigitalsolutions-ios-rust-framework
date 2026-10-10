# PLAN_BINDINGS_F24.md — F24: iOS MPS preferred-device C ABI

## Objective

Expose one opt-in C query over B68's preferred-device-presence Boolean. Do not expose a device, submit GPU work, or claim support for an MPS operation, model, or workload

## Status

F24 is integrated in the root checkout. The optional target-iOS dependency and feature edge, cfg-gated module/export, ABI manifest entry, Cargo.lock edge, macOS CI gates, guide, and documentation links are present. On the `c55410d14c57c3ffdc1982cf62ae3180dd18c468` root baseline, `sh bindings/c/check-ios-mps-status.sh` passed its format, shell syntax, manifest/source/header contract, whitespace, and standalone C11/C++17 header checks. `sh bindings/c/check-ios-mps-status-link.sh` passed feature isolation, host/device/Simulator checks and strict Clippy, Release archive builds, C11/C++17 links, exact imports/symbols, and device/Simulator minima of iOS 12.2/14.0. Linked C/C++ probes were inspected but not executed; no tests or live MPS query ran. The gates establish build and link/import shape only, not runtime behavior or MPS workload support

## Backend contract and availability

B68's `ios_mps_status::preferred_mps_device_available() -> bool` calls `MPSGetPreferredDevice(MPSDeviceOptions::Default)`, maps non-null to `true`, and drops the retained device before return. It exposes no handle, operation, or error result. The imported API is available from iOS 12.2, implemented with `objc2-metal-performance-shaders` 0.3.2 and only its `MPSCore` feature. B68 has no runtime availability guard; callers must run on iOS 12.2 or later. No permission, Info.plist key, or entitlement applies. B68 makes no runtime-cost or thread-affinity claim

G62 records exact imports Foundation, Metal, MetalPerformanceShaders, `libSystem.B.dylib`, and `libobjc.A.dylib`; required symbol `_MPSGetPreferredDevice`; device minos 12.2; Simulator minos 14.0. The API floor is 12.2; the Simulator minimum is only a link setting. Probes were inspected but not executed

## Exact C contract

- Optional feature: `ios-mps-status`, backed by an optional target-iOS dependency on `ios-mps-status`
- Header: `framework_ios_mps_status.h`
- Export: `FrameworkStatus framework_ios_mps_status_preferred_device_available(uint8_t *out_available)`
- A non-null `out_available` must address valid, properly aligned writable memory for one byte during the synchronous call; the API checks nullness only and does not retain the output address. The client must prevent unsynchronized concurrent access. The API writes zero before platform handling
- iOS success returns `OK` and writes exactly 0 or 1 according to preferred-device presence for default options
- A valid non-iOS call returns `UNSUPPORTED` with output zero; null output returns `INVALID_ARGUMENT`; a caught Rust panic returns `PANIC` with zero output
- No pointer or native device crosses C; the retained device is dropped in B68 before its Boolean return
- The query is synchronous. No main-thread rule or thread-safety promise is added; runtime cost and thread affinity are unspecified
- No GPU work, specific operation/model/workload support, parity, or performance claim
- API floor iOS 12.2; link-probe minima iOS 12.2 device and iOS 14.0 Simulator

## F24-owned files

- `bindings/c/src/ios_mps_status.rs`
- `bindings/c/include/framework_ios_mps_status.h`
- `bindings/c/check-ios-mps-status.sh`
- `bindings/c/check-ios-mps-status-link.sh`
- `docs/bindings/ios-mps-status.md`
- this plan

Root-owned integration files were already wired on the `c55410d14c57c3ffdc1982cf62ae3180dd18c468` baseline: `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, `Cargo.lock`, `.github/workflows/ci.yml`, `docs/DOCUMENTATION.md`, and `docs/bindings/cpp.md`

## Static gate and root integration

`sh bindings/c/check-ios-mps-status.sh` checks Rust formatting, shell syntax for both F24 gates, ABI-manifest JSON syntax and F24 shape, source/header symbol agreement, output valid-memory/lifetime/concurrent-access preconditions in source, header, and manifest, whitespace, and standalone C11/C++17 header syntax. It performs no Cargo command, native link, test, consumer execution, or probe

`sh bindings/c/check-ios-mps-status-link.sh` passed. It checks host/device/Simulator feature isolation, default-off MPS bindings and `MPSCore`-only binding features; host/device/Simulator check and strict Clippy; Release archives; C11/C++17 host/device/Simulator links; exact import sets; `_MPSGetPreferredDevice` and `_objc_release`; forbidden Swift, Objective-C messaging, and unrelated GPU/MPS imports; export parity; and final minos 12.2/device and 14.0/Simulator. Probes were linked and inspected, never executed

Root integration is complete: the Cargo feature/dependency, module/re-export, ABI manifest and lock entries, both macOS CI gates, and links from `docs/DOCUMENTATION.md` and `docs/bindings/cpp.md` are present. The link gate establishes import and deployment shape only, not a live preferred-device result or MPS workload support

Mainline run
[38066495166](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38066495166)
later passed the F24 C ABI gates as part of steps 235–275 on macOS 15 and Xcode 27. Linked
probes were not executed; no live MPS query or device behavior is claimed.

# PLAN_BINDINGS_F27.md — F27: Core ML compute-device C status

## Objective

Expose one opt-in C Boolean over D51/B56's complete iOS backend query `ios_core_ml_status::has_available_compute_device()`. Add no portable capability, model API, inference API, or compute-device object

## Candidate decision

D51/B56 is the selected partial (`B`) capability because its public Rust backend already returns one Boolean from a point-in-time Core ML device-list count, guards the iOS 17.0 getter, and defines false below that API floor. No input, handle, callback, object lifetime, permission, UI, network, or capability-specific C error conversion is needed

Other existing partial candidates were not selected for this slice: B53 SafetyKit has an unresolved getter-specific entitlement prerequisite; B55 GameKit reads signed local-player state; B58 Speech needs raw native authorization-state mapping; B60 is deprecated StoreKit 1 purchase ability; B61 RoomPlan uses a Swift ABI thunk; B63 StoreKit 2 uses a Swift ABI thunk. These remain separate bounded C contracts, not gaps in F27

The remaining documented `X` rows remain outside F27: FamilyControls lacks a supported Rust route and documented query-only entitlement semantics; Foundation Models and several service/framework status APIs are Swift-only; DeviceActivity's getter meaning and entitlement semantics are undocumented; PTT/CarPlay need entitlement, APNs, scene, permission, or lifecycle state; HomeKit's first manager use can prompt; extension and browser rows need app/extension lifecycle or host data. See their focused capability plans for exact limits. F27 does not alter the support matrix or counts

## Backend contract and availability

B56 calls only `MLModel.availableComputeDevices` through `objc2-core-ml` 0.3.2 with `MLModel`, `MLModel_MLComputeDevice`, and `MLComputeDeviceProtocol`. The iOS API floor is 17.0. B56 returns false below that floor. A true value means only that the listed array is nonempty at query time; it does not prove that any model or operation can run. No model load, configuration, inference, input, device-object return, permission, or UI is in scope

The local iPhoneOS26.5 SDK places the `MLModel (MLComputeDevice)` category and `availableComputeDevices` at iOS 17.0, while the base `MLModel` class and CoreML framework begin at iOS 11.0. The planned device link minos is 11.0 and Simulator minos is 14.0. The B56 runtime guard protects the category getter below iOS 17.0; target minima are link settings, not the API floor

## Exact C contract

- Optional feature: `ios-core-ml-status`
- Header: `framework_ios_core_ml_status.h`
- Export: `FrameworkStatus framework_ios_core_ml_status_has_available_compute_device(uint8_t *out_available)`
- Required output is one caller-owned writable byte; it is zero before platform handling and never retained. The API checks only nullness, so a non-null pointer must address valid, properly aligned writable memory during the synchronous call, and the caller must prevent unsynchronized concurrent access
- On iOS, return `FRAMEWORK_STATUS_OK` and write exactly `0` or `1` from B56. Below iOS 17.0 B56 returns false, so this API returns OK with zero
- Valid non-iOS input returns `FRAMEWORK_STATUS_UNSUPPORTED` with output zero
- Null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write; panic returns `FRAMEWORK_STATUS_PANIC` with output zero and does not unwind across C
- No Core ML object or device list crosses C; calls are synchronous on the caller's thread with no added queue guarantee

## Scope and root wiring

F27 owns `bindings/c/src/ios_core_ml_status.rs`, `bindings/c/include/framework_ios_core_ml_status.h`, `bindings/c/check-ios-core-ml-status.sh`, `bindings/c/check-ios-core-ml-status-link.sh`, `docs/bindings/ios-core-ml-status.md`, and this plan. The isolated implementation also adds the optional target-iOS dependency/feature, module/re-export, ABI manifest entry, Cargo.lock dependency resolution, named macOS CI checks, F27 aggregate status, and documentation index link. No global capability matrix, count, or backend code changes

## Validation and evidence

- `cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features ios-core-ml-status` passed and refreshed only the isolated worktree lock resolution for the new optional dependency
- `sh bindings/c/check-ios-core-ml-status.sh` passed: feature graph isolation, Rust format check, locked host/device/Simulator Cargo check and strict Clippy, host/iOS rustdoc, JSON/source/header contract checks, and standalone C11/C++17 header syntax
- The integrated static/build gate asserts source, header, guide, plan, and manifest output-pointer
  preconditions: valid aligned writable storage for the full synchronous call, caller protection
  from unsynchronized access, zero initialization before platform handling, nullness-only
  validation, and no pointer retention. It does not prove arbitrary C memory validity
- `sh -n bindings/c/check-ios-core-ml-status.sh` and `sh -n bindings/c/check-ios-core-ml-status-link.sh` passed as part of the static/build gate
- `sh bindings/c/check-ios-core-ml-status-link.sh` passed the host/device/Simulator C11/C++17 Release link/import gate and minos inspection after root integration. Consumers and probes were not executed
- No tests, consumers, linked probes, Core ML model loads, inference, live device-list calls, parity, or performance checks were run
- Exact device/Simulator C and C++ direct imports are CoreML, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; host C and C++ import only `libSystem.B.dylib`. C++ links use `-nostdlib++`; device minos is 11.0 and Simulator minos is 14.0
- The installed host is Xcode 26.6 build 17F113 with iPhoneOS and Simulator SDK 26.5, below PLAN.md's Xcode 27.x baseline

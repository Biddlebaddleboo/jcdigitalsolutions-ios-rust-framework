# PLAN_BINDINGS_F31.md — F31: RoomPlan support C ABI

## Objective

Expose B61's existing RoomPlan device-support snapshot through one opt-in C function. Add no
portable capability, session lifecycle, permission, camera/LiDAR frame, UI, or scan API

## Candidate and bounds

B61 is selected because `ios-roomplan::device_support()` already returns one owned
`RoomPlanDeviceSupport` from only `RoomCaptureSession.isSupported` on iOS and returns `None` on
non-iOS. Its Swift ABI thunk is compiler-checked by the B61 gate and its repository-owned native
bridge contains no Swift source. F31 calls that backend; it does not add a new Swift ABI call or
touch the B61 backend

F31 exports `framework_ios_roomplan_status_is_supported(uint8_t *out_supported)`. A successful iOS
call writes exactly 0 or 1 and returns `FRAMEWORK_STATUS_OK`. A valid non-iOS call writes zero and
returns `FRAMEWORK_STATUS_UNSUPPORTED`; null output returns
`FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write; caught panic returns
`FRAMEWORK_STATUS_PANIC` with zero output. The output is caller-owned, valid, aligned, writable for
one byte for the synchronous call, not retained, and must not be accessed concurrently without
synchronization

The public API and required deployment floor are iOS 16.0. The B61 backend has a strong Swift symbol
import and its own device/Simulator probes use minos 16.0/16.0; F31 adds no weak-symbol fallback.
The F31 C/C++ link gate links host consumers without RoomPlan, then links device and Simulator
consumers against RoomPlan and checks exact direct imports, required RoomPlan symbols, selected
Swift/Objective-C runtime exclusions, and minos 16.0. The consumers are not executed

The query does not create, run, or stop a `RoomCaptureSession`, access camera or LiDAR frames,
request permission, present UI, start a scan, or establish scan readiness or success. No
thread/queue, parity, or performance claim is added

## Exact ABI

- Cargo feature: `ios-roomplan-status`; default remains empty
- Header: `bindings/c/include/framework_ios_roomplan_status.h`
- Export: `FrameworkStatus framework_ios_roomplan_status_is_supported(uint8_t *out_supported)`
- `out_supported` is one required caller-owned byte and is initialized to zero before platform
  handling; nullness is the only pointer check
- Non-iOS valid calls return `FRAMEWORK_STATUS_UNSUPPORTED`; no RoomPlan dependency is active on
  host targets
- No RoomPlan session, Swift object, callback, or native pointer crosses C

## Source evidence

- B61 contract and exact compiler-oracle/backend link evidence: `PLAN_IOS_ROOMPLAN.md`
- `crates/framework-roomplan/src/lib.rs` owns the portable result; `platform/ios/ios-roomplan/src/lib.rs`
  calls only B61's Rust backend API
- The B61 backend API floor is iOS 16.0, and its already-measured RoomPlan/libSystem probes use
  iOS 16.0 minos for device and Simulator

## Acceptance and validation

- [x] Add an opt-in target-iOS dependency and feature; keep default and host dependency graphs free
  of `ios-roomplan`
- [x] Add one output-only C symbol and fixed 0/1 result
- [x] Document and statically assert valid aligned output storage through the full call, caller protection from unsynchronized access, zero initialization, and no pointer retention
- [x] Preserve B61's strong iOS 16.0 API/deployment floor; add no weak-symbol fallback
- [x] Keep non-iOS behavior unsupported with zero output and catch panics at the C boundary
- [x] Add the focused source, header, manifest entry, guide, and plan
- [x] `sh bindings/c/check-ios-roomplan-status.sh` passed in the integrated checkout with host/device/Simulator feature isolation,
  Rust checks and strict Clippy, rustdoc, and C11/C++17 header syntax
- [x] `sh bindings/c/check-ios-roomplan-status-link.sh` passed C11/C++17 host/device/Simulator links,
  exact RoomPlan/libSystem target imports, required RoomPlan symbols, and minos 16.0

The gates ran with Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5 after root refreshed the
shared lock edge. Host/device/Simulator feature graphs, checks, Clippy, and rustdoc passed; the C11
and C++17 header syntax fixtures passed. The link gate passed host C/C++ imports of only
`libSystem.B.dylib` and device/Simulator C/C++ imports of `RoomPlan` plus `libSystem.B.dylib`; the
required public `RoomCaptureSession` metadata accessor and `isSupported` getter imports and iOS
16.0 minos passed. No tests, consumers, probes, or RoomPlan calls ran. No passing CI workflow run
had been recorded at this validation point, and Xcode 26.6 / SDK 26.5 is below the Xcode 27.x
baseline. Mainline run
[38066495166](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38066495166)
later passed the F31 C ABI gates as part of steps 235–275 on macOS 15 and Xcode 27. Linked probes
were not executed; this is not runtime or device evidence.

## Root integration

Root owns the shared lockfile and aggregate workflow wiring. Root refreshed the lock with
`cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features
ios-roomplan-status` and wired the static/build and link/import gates into macOS CI

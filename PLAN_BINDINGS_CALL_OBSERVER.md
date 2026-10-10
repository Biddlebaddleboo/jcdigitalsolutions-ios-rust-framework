# PLAN_BINDINGS_CALL_OBSERVER.md — Workstream F12: CallKit Snapshot C ABI

## D76 source contract

- `platform/ios/callkit/ios-call-observer::active_call_snapshot()` calls
  `CXCallObserver::new()` and `CXCallObserver.calls()`, then copies a `u64` list count and the
  OR of four `u32` state flags into `CallActivitySnapshot`.
- The four flags report whether any returned call is outgoing, connected, on hold, or ended;
  they do not correlate state to one call.
- The backend creates and releases CallKit objects within the synchronous call. It exposes no
  handle, delegate, callback, UUID, or caller data.
- CallKit's `calls` read may block while it retrieves initial state. F12 runs on the caller's
  thread, adds no main-thread rule, and requires the host to keep the call off UI-critical work.
- D76 API floor: iOS 10.0. F12 link probes use device 12.0 and Simulator 14.0; these are
  toolchain/validation floors, not a raised API floor.

## C API

- Cargo feature: `ios-call-observer`, off by default; only the iOS target enables the optional
  `ios-call-observer` package. Its own `objc2-call-kit` dependency has default features disabled
  and enables only `CXCallObserver` and `CXCall`.
- Header: `framework_ios_call_observer.h`.
- Export: `framework_ios_call_observer_active_call_snapshot(uint64_t *out_call_count,
  uint32_t *out_state_flags)`.
- Output tags are fixed-width: OUTGOING = 1, CONNECTED = 2, ON_HOLD = 4, ENDED = 8. Pass D76's
  raw flag mask through unchanged.
- Each non-null required output is initialized to zero. Null output returns
  `FRAMEWORK_STATUS_INVALID_ARGUMENT`; success writes both values and returns `FRAMEWORK_STATUS_OK`.
- On non-iOS, return `FRAMEWORK_STATUS_UNSUPPORTED` with both outputs zero. A caught panic returns
  `FRAMEWORK_STATUS_PANIC` with both outputs zero.
- No CallKit object, UUID, caller data, callback, permission API, delegate, history, provider,
  controller, audio API, PushKit path, or call-control API crosses C.
- The API is synchronous, may block, imposes no main-thread rule, and makes no live-call visibility
  or service-readiness claim.

## Scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs` and `bindings/c/src/ios_call_observer.rs`
- `bindings/c/include/framework_ios_call_observer.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-call-observer.sh`
- `docs/bindings/ios-call-observer.md`
- `PLAN_BINDINGS_CALL_OBSERVER.md`

Root owns `Cargo.lock`, CI, root `PLAN_BINDINGS.md`, aggregate plans, shared documentation indexes,
workspace membership, and capability counts. Do not edit those files in F12 or change D76 backend
files.

## Validation gate

The named gate must not add or run tests or execute consumer/probe binaries. It must check manifest
JSON and shell syntax, format the C ABI crate, compare header symbols/tags to the ABI manifest,
verify default and non-iOS feature isolation plus the exact `objc2-call-kit` feature closure, run
locked host/device/Simulator library checks and strict Clippy, build host and arm64 Apple Release
archives, compile/link C11/C++17 consumers without execution, inspect exact F12 exports/imports,
reject unrelated frameworks/symbols, and verify minos metadata.

## Status and validation record

F12 source, header, ABI manifest, guide, and check script are implemented. Root refreshed
`Cargo.lock` with the command below; the locked F12 gate then passed with Rust 1.94.1
(`e408947bf`, 2026-03-25), Xcode 26.6 (Build 17F113), and iOS SDKs 26.5.

```sh
sh bindings/c/check-ios-call-observer.sh
```

The gate passed manifest JSON and shell checks, `cargo fmt --check`, host/device/Simulator locked
`cargo check`, strict Clippy, Release static archive builds, and C11/C++17 host/device/Simulator
compile/link consumers. It verified the sole export
`framework_ios_call_observer_active_call_snapshot`; device C imports are `CallKit`, `Foundation`,
`libSystem.B.dylib`, and `libobjc.A.dylib`; device C++ adds `libc++.1.dylib`. Simulator C/C++
imports matched those same respective lists. The linked device consumers report `minos 12.0`, and
Simulator consumers report `minos 14.0`; both remain validation floors, while D76's API floor is
iOS 10.0. The consumers were not executed. No tests or live CallKit queries were run, so the
snapshot's live-call visibility and runtime latency remain unverified; the backend may block while
CallKit provides initial state.

Xcode 27 run `38048808184` on head `c78e8597fa1c497167105a6250825fd93d3cb2cc` passed the
preferences C ABI step 241, MediaPlayer step 242, and SpriteKit step 243, then failed CallKit C ABI
step 244 in Xcode-27 job `114203611456`. The device C11 probe linked with minos 12.0; the following
C++17 compile entered the Xcode 27 SDK libc++ `stddef.h` shim and promoted `availability.h:204`
warning `The selected platform is no longer supported by libc++` to an error under `-Werror`.
Since the fixtures use only C ABI declarations, `check-ios-call-observer.sh` now passes
`-nostdinc++` to its C++ host/device/Simulator compile commands while retaining C++17, the
`libc++.1.dylib` runtime import assertion, CallKit/Foundation and Objective-C import checks,
symbols/selectors, device 12.0 and Simulator 14.0 minos checks, and D76's iOS 10.0 API floor.
Xcode 27 CallKit requalification is pending; no consumer or probe execution/runtime behavior is
claimed

## Root integration steps

After F12 adds its optional path dependency, refresh the root lock with:

```sh
cargo +1.94.1 check --offline -p framework-c-api --no-default-features \
  --features ios-call-observer --target aarch64-apple-ios
```

Then add `sh bindings/c/check-ios-call-observer.sh` to the existing macOS binding-check step in
`.github/workflows/ci.yml`. That script performs the locked host/device/Simulator gates; linked
consumers are never executed. Root should add the F12 link to `PLAN_BINDINGS.md` and the shared docs
index separately.

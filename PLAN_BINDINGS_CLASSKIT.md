# PLAN_BINDINGS_CLASSKIT.md — Workstream F14: ClassKit Deep-Link Marker C ABI

## Objective

Expose only D81's synchronous ClassKit deep-link marker getter through an opt-in C feature. Accept
an opaque, caller-owned `NSUserActivity` pointer and return one fixed-width Boolean value. Do not
expose ClassKit contexts, assignment data, identifiers, or service access.

## Stable D81 contract

- `ios-system-services::is_classkit_deep_link(&NSUserActivity) -> bool` reads only the read-only
  `NSUserActivity.isClassKitDeepLink` property.
- The property API floor is iOS 11.3. F14 checks runtime availability before it casts or derefs the
  borrowed pointer; a lower runtime returns `FRAMEWORK_STATUS_UNAVAILABLE`.
- D81 borrows the activity for the synchronous call and does not retain or store it. The host owns
  the object and must preserve its lifetime and follow the host activity lifecycle/thread rules.
- The query does not read `contextIdentifierPath`, access `CLSDataStore`, create or inspect ClassKit
  contexts, validate assignment content, or identify a student.
- The non-iOS C stub checks only nullness and does not dereference a non-null opaque pointer.

## C contract

- Cargo feature `ios-classkit-deep-link` is off by default. On iOS it enables `ios-system-services`,
  `objc2`, and `objc2-foundation` with `NSUserActivity`; no ClassKit data-store feature is enabled.
- Header: `framework_ios_classkit_deep_link.h`. It forward-declares `NSUserActivity` as an opaque
  type outside Objective-C.
- Export: `framework_ios_classkit_is_deep_link(const NSUserActivity *activity,
  FrameworkIosClassKitBoolean *out_is_deep_link)`.
- `FrameworkIosClassKitBoolean` is `uint8_t`; success writes exactly `0` or `1`.
- The output is required and is initialized to zero first. Null activity or output returns
  `FRAMEWORK_STATUS_INVALID_ARGUMENT`; iOS runtime below 11.3 returns
  `FRAMEWORK_STATUS_UNAVAILABLE`; non-null non-iOS input returns `FRAMEWORK_STATUS_UNSUPPORTED`
  without pointer dereference. A caught panic returns `FRAMEWORK_STATUS_PANIC` with output zero.
- The activity pointer is borrowed only for the call; the wrapper performs no retain, store,
  release, consume, queue hop, or main-thread enforcement. The host must keep a live object and
  observe its own lifecycle/thread rules. The output must not overlap the activity object.
- This marker does not grant or expose ClassKit assignment-data access.

## Scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs` and `bindings/c/src/ios_classkit_deep_link.rs`
- `bindings/c/include/framework_ios_classkit_deep_link.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-classkit-deep-link.sh`
- `docs/bindings/ios-classkit-deep-link.md`
- `PLAN_BINDINGS_CLASSKIT.md`

Root owns `Cargo.lock`, CI, root `PLAN_BINDINGS.md`, aggregate plans, shared validation docs, and
documentation indexes. Do not edit those files or the D81 package in F14.

## Validation gate

`sh bindings/c/check-ios-classkit-deep-link.sh` must check manifest JSON and shell syntax, format
the C ABI crate, compare header symbols to the ABI manifest, verify default/host feature isolation
and the exact ClassKit/Foundation binding feature closure, run locked host/device/Simulator checks
and strict Clippy, build Release archives, compile/link C11/C++17 consumers without execution,
inspect the one F14 export, exact framework/runtime imports and deep-link selector, reject
ClassKit data-store/path symbols, and verify deployment metadata. It must not add or run tests or
execute linked consumers.

## Status and validation record

F14 source, header, ABI manifest, guide, and gate are implemented. Root refreshed `Cargo.lock` with:

```sh
cargo +1.94.1 check --offline -p framework-c-api --no-default-features \
  --features ios-classkit-deep-link --target aarch64-apple-ios
```

On Rust 1.94.1 (`e408947bf`, 2026-03-25), Xcode 26.6 (Build 17F113), and iOS SDKs 26.5,
`sh bindings/c/check-ios-classkit-deep-link.sh` passed. It checked manifest JSON, shell syntax,
formatting, header/manifest symbol parity, default and host dependency isolation, the exact
ClassKit/Foundation feature closure, locked host/device/Simulator checks and strict Clippy, Release
archives, and C11/C++17 host/device/Simulator compile/link consumers. The only F14 export is
`framework_ios_classkit_is_deep_link`; target probes reference `objc_msgSend` and the
`isClassKitDeepLink` selector. The probes contain no `CLSDataStore`, `CLSContext`, `CLSActivity`, or
`contextIdentifierPath` symbol/string.

Device C imports are `ClassKit`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ adds
`libc++.1.dylib`. Simulator imports match those respective lists. Device metadata reports minimum
iOS 11.3; Simulator metadata reports minimum iOS 14.0. The API/runtime floor is iOS 11.3. The host
archive has no ClassKit or Objective-C imports. Probe consumers were linked and inspected, not
executed. No tests, live `NSUserActivity`, Schoolwork flow, or ClassKit assignment-data access were
run or claimed.

Xcode 27 run `38051886481` on head `59555590b4fec231ea2f3686faff43df9e7220fb`, job
`114212506187`, passed the MapKit C ABI step 245 and failed ClassKit step 246. The ClassKit device
C11 fixture compiled and linked at minos 11.3; the following C++17 fixture compile entered the
Xcode 27 SDK libc++ `stddef.h` shim and failed at `availability.h:204` with
`The selected platform is no longer supported by libc++`, promoted to an error by `-Werror`. The
device/Simulator C++ fixture compiles now use `-nostdinc++` because they consume only C ABI
declarations. The C11 compile remains unchanged; the gate retains C++17, the `libc++.1.dylib`
import assertion, ClassKit/Foundation and Objective-C imports, symbol/selector checks, device
11.3 and Simulator 14.0 minos, and the iOS 11.3 API/runtime floor. Xcode 27 ClassKit
requalification is pending; no consumer execution or runtime behavior is claimed.

## Root integration steps

After the root lock refresh, add `sh bindings/c/check-ios-classkit-deep-link.sh` beside the other C
ABI gates in the macOS CI workflow. Root adds the F14 plan and guide links to aggregate plans and
documentation indexes separately.

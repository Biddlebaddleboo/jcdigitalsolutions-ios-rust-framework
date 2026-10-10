# PLAN_BINDINGS_MEDIA_LIBRARY_STATUS.md — Workstream F10: MediaPlayer Status C ABI

## Objective

Expose D65/B71's point-in-time iOS MediaPlayer library authorization-status query through a small
opt-in C API. Preserve the native signed 64-bit raw value, including values unknown to this crate.

## Stable source contract

- D65/B71 is implemented in `platform/ios/ios-media-library-status` and documented in
  `docs/ios/media-library-status.md`.
- `media_library_authorization_status()` synchronously calls `+[MPMediaLibrary authorizationStatus]`
  from iOS 9.3 and maps known values 0–3 to `NotDetermined`, `Denied`, `Restricted`, and
  `Authorized`; all other signed values remain `Unknown(i64)`.
- The query does not request authorization, present a prompt, create a media-library object, read
  library items, or contact Apple Music services. B71 makes no thread-safety promise and has no
  main-thread requirement.
- There is no portable D contract or non-iOS B71 function. The C feature's host stub is only an
  ABI/link surface: it initializes output and returns `FRAMEWORK_STATUS_UNSUPPORTED`.

## Public C contract

- Cargo feature: `ios-media-library-status`, isolated from every other capability feature.
- Header: `framework_ios_media_library_status.h`.
- Export: `framework_ios_media_library_authorization_status(int64_t *out_raw_status)`.
- Known raw values: `FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_NOT_DETERMINED` = 0,
  `FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_DENIED` = 1,
  `FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_RESTRICTED` = 2, and
  `FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_AUTHORIZED` = 3. Preserve any other native `int64_t`
  unchanged; do not collapse it to an `UNKNOWN` tag.
- Initialize a non-null output to zero before the query. Return `FRAMEWORK_STATUS_INVALID_ARGUMENT`
  for null output; return `FRAMEWORK_STATUS_OK` and the raw status on iOS; return
  `FRAMEWORK_STATUS_UNSUPPORTED` with zero output on non-iOS; contain a panic as
  `FRAMEWORK_STATUS_PANIC` with zero output.
- No handle, owned buffer, callback, error code, permission request, native object, or fabricated
  success state is part of this ABI.
- The host application must deploy to iOS 9.3 or later or guard this API at runtime. The C link
  checks use device 10.0 and Simulator 14.0 floors; those validation settings do not change B71's
  API floor.

## Scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/ios_media_library_status.rs` and the F10 module/export lines in `src/lib.rs`
- `bindings/c/include/framework_ios_media_library_status.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-media-library-status.sh`
- `docs/bindings/ios-media-library-status.md`
- `PLAN_BINDINGS_MEDIA_LIBRARY_STATUS.md`

Root owns `Cargo.lock`, root `PLAN_BINDINGS.md`, CI, shared documentation indexes, capability
counts, and canonical status summaries. Do not modify those files in F10.

## Validation gate

The named check must, without running tests or probe binaries:

- Check manifest JSON, shell syntax, Rust formatting, exact header/manifest symbols and raw-value
  constants, default-feature exclusion, the iOS `MPMediaLibrary` feature selection, and no
  MediaPlayer package in the host feature graph
- Run locked host, iOS device, and Simulator library checks and strict Clippy; build the C API
  release static library for host, arm64 iOS device, and arm64 Simulator
- Compile and link minimal C11 and C++17 host consumers plus arm64 device and Simulator probes
- Compare exact linked imports and exported F10 symbols, inspect Objective-C runtime/message-send
  and authorization-status selector strings, reject unrelated framework imports, and verify
  deployment metadata
- Record actual imports, symbols, API/probe floors, Xcode/SDK, exact commands, and the fact that no
  consumer or probe binary ran

## Status and blockers

F10 source, header, Cargo feature edge, ABI-manifest entry, focused guide, and named check script
are integrated. `cargo +1.94.1 check --offline -p framework-c-api --no-default-features
--features ios-media-library-status` refreshed the shared lock, and
`sh bindings/c/check-ios-media-library-status.sh` passed in the integrated checkout. Locked host,
device, and Simulator checks, strict Clippy, C11/C++17 consumer links, exact export/import audits,
feature isolation, selector strings, and deployment metadata all passed. C imports are Foundation,
MediaPlayer, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ also imports `libc++.1.dylib`. The
device probe minos is 10.0 and Simulator minos is 14.0. The iOS API floor remains 9.3. No consumer
or probe binary ran. CI and shared docs index links are integrated. No tests were added or run.

Xcode 27 run `38043226353` on head `3e2c7e66538df74e88a255de4da22920146c4b05` passed the
preferences C ABI step 241, then failed MediaPlayer C ABI step 242 in Xcode-27 job
`114187479698`. The C++17 fixture compile for the device probe entered the Xcode 27 SDK libc++
`stddef.h` shim and promoted `availability.h:204` warning `The selected platform is no longer
supported by libc++` to an error under `-Werror`; the Rust checks and release archive build had
passed. Since the fixture uses only C ABI declarations, the script now passes `-nostdinc++` to its
C++ compile commands while retaining C++17, the libc++ runtime link/import audit for
`libc++.1.dylib`, all framework/symbol assertions, and device 10.0 / Simulator 14.0 minos checks.
Xcode 27 MediaPlayer requalification is pending. The failure and source correction provide no
consumer execution or runtime-behavior evidence

macOS 15 CI run `38047131769` on head `a2c8dba55875aa4f7d85abe254f9c90e850cd8e4` passed preferences C ABI step 241, then MediaPlayer / Media Library Status C ABI step 242 failed in macOS job `114198818252` with exit code 1. The hosted log does not identify the failing assertion; the script redirects Rust archive `nm` diagnostics to `/dev/null`, and no exact Apple `nm` error is visible for this step. The script used Apple `nm` for host, iOS device, and Simulator Rust static archives, while the adjacent secure-storage and preferences gates use Rust's bundled LLVM symbol reader. Attribution to the known Xcode 16.4 Apple `nm` versus Rust LLVM 21 archive incompatibility is therefore an inference, not a directly observed diagnostic. The script now selects `llvm-nm` from the active rustc sysroot/host for Rust archive symbol and undefined-import inspections only; Apple `nm` remains in place for final linked Mach-O probes. All symbol/import assertions, link checks, API floors, and deployment metadata checks are unchanged. Static shell syntax and diff checks only; the check script was not rerun. The failed CI step and this correction do not establish successful C consumer linkage, probe execution, or runtime behavior.

Do not claim live authorization, media-library access, Apple Music catalog/subscription service,
playback availability, permission prompt behavior, or thread-safety.

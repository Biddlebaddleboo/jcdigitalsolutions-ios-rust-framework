# PLAN_BINDINGS_F30.md — F30: extension metadata C ABI

## Objective

Expose one already-implemented B77 runtime metadata read through an opt-in iOS C function. Add no
portable extension contract, extension loading, App Intents support, .appext compiler metadata,
build-host plist generation, or capability-count change

## Candidate and bounds

B77 is the selected backend because `ios-extension-support::read_extension_point_identifier`
accepts one caller-supplied path, reads only `NSExtension.NSExtensionPointIdentifier` through typed
`objc2-foundation` 0.3.2 calls, copies the string into Rust-owned UTF-8, and returns eight exact
bounded errors. The C layer adds no bundle search, path normalization, extension-point schema
validation, or Foundation object exposure

F30 accepts one borrowed `FrameworkStr` path. It returns `FRAMEWORK_STATUS_OK` with either
`FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE` and one caller-owned `FrameworkOwnedBuffer`, or one
of the eight exact B77 metadata error codes and an empty buffer. Malformed C spans, invalid UTF-8,
output nullness, or overlapping ranges return `FRAMEWORK_STATUS_INVALID_ARGUMENT`. A valid non-iOS
call returns `FRAMEWORK_STATUS_UNSUPPORTED` with both outputs initialized to zero. A caught panic
returns `FRAMEWORK_STATUS_PANIC`; buffer conversion overflow returns
`FRAMEWORK_STATUS_RESOURCE_EXHAUSTED`

The output descriptor must be empty of any live framework allocation on entry. Its returned bytes
are length-delimited UTF-8 and must be released once, unchanged, with
`framework_owned_buffer_destroy`. Input and outputs are caller-owned; the call is synchronous and
retains no pointer. The outputs must be writable, aligned, and mutually disjoint and disjoint from
the input span

The backend API floor is iOS 4.0 from `NSBundle.bundleWithURL:`. F30's Release link probe uses
device minos 12.0 and Simulator minos 14.0; these are probe settings, not the API floor. The
runtime metadata result does not prove that an extension is installed, registered, enabled,
signed, approved, entitled, launchable, or compatible with a host

## Exact ABI

- Cargo feature: `ios-extension-support`; default remains empty
- Header: `bindings/c/include/framework_ios_extension_support.h`
- Export: `FrameworkStatus framework_ios_extension_support_read_extension_point_identifier(FrameworkStr bundle_path, FrameworkIosExtensionMetadataError *out_error, FrameworkOwnedBuffer *out_identifier)`
- `FrameworkIosExtensionMetadataError` is fixed `uint32_t`; NONE is 0 and B77 errors map exactly to codes 1–8
- `FRAMEWORK_STATUS_OK` reports the query completion; `out_error` distinguishes B77 metadata outcomes from an identifier result
- `FrameworkOwnedBuffer` carries copied UTF-8 bytes; callers must destroy the original descriptor once with `framework_owned_buffer_destroy`
- No `NSBundle`, `NSDictionary`, `NSString`, native pointer, callback, extension code, or C++ object crosses the ABI

## Source evidence

- B77 contract: `PLAN_IOS_EXTENSION_SUPPORT.md`; its typed Foundation reader has no unsafe code and does not call `NSBundle.load`
- Apple docs: `NSBundle.bundleWithURL:`, `NSBundle.infoDictionary`, and `NSExtensionPointIdentifier` define the bounded bundle and metadata route
- Installed SDK: `Foundation.framework/Headers/NSBundle.h` marks `bundleWithURL:` available from iOS 4.0; `infoDictionary` has no higher availability annotation
- B77 package gate records exact imports Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; it records device minos 12.0 and Simulator minos 14.0 for its own probes

## Acceptance and validation

- [x] Add opt-in target-iOS dependency and feature edge; default and host feature graphs omit B77
- [x] Add one C symbol, one fixed error-code type, and one caller-owned UTF-8 result buffer
- [x] Preserve B77's eight metadata errors and caller-supplied path boundary
- [x] Require valid output storage, empty buffer output on entry, disjoint spans, and exactly-once buffer destruction
- [x] Keep non-iOS behavior unsupported and preserve the B77 API floor separately from link minos
- [x] Add manifest, C11/C++17 static and device/Simulator link-import gates, guide, and this plan
- [x] `sh bindings/c/check-ios-extension-support.sh` passed in the integrated checkout: host/device/Simulator feature
  isolation, strict Clippy, rustdoc, C11/C++17 header/layout/error checks, and manifest validation
- [x] `sh bindings/c/check-ios-extension-support-link.sh` passed in the integrated checkout: host/device/Simulator C11/C++17
  Release links, export parity, selector and forbidden-surface audits, exact import allowlists,
  and deployment-minimum checks

Host C/C++ imported only `libSystem.B.dylib`; device and Simulator C/C++ imported exactly
Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. Device and Simulator minos were 12.0 and
14.0. The link probes were inspected, not executed. The gates ran on Xcode 26.6 (build 17F113) / iOS
SDK 26.5; no passing CI workflow run is recorded

The gates may compile, link, and inspect C/C++ consumers and probes. They must not execute them.
No tests, extension load, live metadata read, install/approval query, or host launch is part of F30

## Root integration

Root owns the shared workspace lock refresh, CI, aggregate plan/index, and final manifest review.
The integrated lock edge was refreshed with
`cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features ios-extension-support`

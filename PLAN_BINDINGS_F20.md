# PLAN_BINDINGS_F20.md — F20: iOS ModelIO Extension-Status C ABI

## Objective

Expose only B67's synchronous `ios_modelio_status::can_import_file_extension(&str) -> bool` API through one opt-in C function. Preserve its exact extension-support meaning. Do not create/load assets or imply parsing, rendering, or GPU support

## Status

F20 is integrated in the root C ABI feature graph, public exports, ABI manifest, Cargo.lock, macOS CI, aggregate plan, and docs index. `sh bindings/c/check-ios-modelio-status.sh` passed host/device/Simulator feature isolation, strict Clippy, Release builds, C11/C++17 links, exact Foundation/ModelIO/libSystem/libobjc imports (+libc++ for C++), selector/message-send checks, and minos 10.0/14.0. Its static gate asserts valid aligned writable output memory, full-call lifetime, caller concurrency protection, range/overlap checks, and that the wrapper cannot prove memory validity. No tests or C/C++ consumer/probe binaries were executed

## Feasibility and backend contract

B67 provides an iOS-only `ios_modelio_status::can_import_file_extension(&str) -> bool`, implemented with `objc2-model-io` 0.3.2 (`MDLAsset` feature only) and Foundation's `NSString`. It sends `+[MDLAsset canImportFileExtension:]` and does not create an asset, load a URL, or read file data. The generated binding takes `&NSString` and returns `bool`; the method has no later availability annotation than `MDLAsset`'s iOS 9.0 class floor. Device and Simulator link-probe minima are 10.0 and 14.0, so no runtime unavailable branch is required

The C wrapper accepts `FrameworkStr`, validates the pointer/length representation, rejects overlap before creating a Rust string or writing output, validates UTF-8, then calls the existing API on iOS. The output is one caller-owned byte, initialized to zero after structural/disjoint validation. No native object, asset, URL, handle, callback, or owned buffer crosses C. Primary-source, SDK, binding, import, and floor evidence is recorded in [B67](PLAN_IOS_MODELIO_STATUS.md) and [D61](PLAN_CAPABILITIES_MODELIO_STATUS.md); see also Apple's [`canImportFileExtension(_:)` documentation](https://developer.apple.com/documentation/modelio/mdlasset/canimportfileextension%28_%3A%29) and [`MDLAsset` documentation](https://developer.apple.com/documentation/modelio/mdlasset)

## Exact C contract

- Optional Cargo feature: `ios-modelio-status`, backed only by an optional target-iOS dependency on `ios-modelio-status`
- Header: `framework_ios_modelio_status.h`
- Export: `FrameworkStatus framework_ios_modelio_can_import_file_extension(FrameworkStr extension, uint8_t *out_supported)`
- Input: borrowed valid UTF-8; null is valid only for zero length; empty text is passed through; no normalization is added. Input bytes remain readable and immutable through the synchronous call
- Output: required caller-owned valid, properly aligned writable byte for the full synchronous call, disjoint from nonempty input, never retained. The caller must prevent unsynchronized access to either region. The wrapper checks range arithmetic and input/output overlap but cannot prove that memory is valid, aligned, writable, or live. After output pointer, length representation, span, and non-overlap validation, initialize it to zero before UTF-8, platform, or native-result handling
- Status: `OK` writes exactly 0 or 1; unsupported/unknown extension is successful false; null output, malformed span, overlap, or invalid UTF-8 is `INVALID_ARGUMENT`; valid non-iOS call is `UNSUPPORTED` with zero output; caught Rust panic is `PANIC` with zero output. There is no native error or runtime-unavailable path
- Threading: synchronous on the caller's thread; adds no main-thread rule or thread-safety guarantee
- Ownership: no native pointer, object, asset, handle, callback, or `FrameworkOwnedBuffer` crosses C
- Limits: extension support only; no asset creation/load, URL/file-data access, parse/validate/render/GPU result, parity, or performance claim

## F20-owned files

- `bindings/c/src/ios_modelio_status.rs`
- `bindings/c/include/framework_ios_modelio_status.h`
- `bindings/c/check-ios-modelio-status.sh`
- `docs/bindings/ios-modelio-status.md`
- this plan

Root owns `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, Cargo.lock, CI, aggregate binding status, and documentation indexes

## Focused gate and root integration

`sh bindings/c/check-ios-modelio-status.sh` checks the integrated default/iOS/host feature trees and
binding-feature isolation, host C11/C++17 links and import isolation, host/device/Simulator
compilation and strict Clippy, Release archives, C11/C++17 device/Simulator links, exact
Foundation/ModelIO/libSystem/libobjc imports (plus libc++ for C++),
`canImportFileExtension:` and Objective-C message-send imports, one C ABI export, and deployment
minima 10.0/14.0. Its static assertions cover pointer validity limits, lifetime, synchronization,
input/output overlap, and null behavior. Consumers are build-only and must not run. No tests are in
scope

Root integration is complete: the target-iOS optional dependency/feature, module and export gate,
ABI manifest entry, Cargo.lock edge, macOS CI invocation, aggregate status, and documentation index
are present. The B67 backend remains unchanged

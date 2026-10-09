# PLAN_SWIFT_ABI_OWNERSHIP.md — Workstream C1: Swift Class Ownership Proof

## Status

C1 passes `sh interop/swift-abi-core/tests/check-retained-ownership.sh`: the host oracle completed
64 exact Swift class deinit cycles across retain, clone, drop, and re-adoption. Device/simulator
compiler checks passed but did not run. The default feature path has no `swiftCore` linkage or
retain/release imports. Xcode 26.6 / SDK 26.5 remains below the Xcode 27.x plan baseline.

## Objective

Establish executable evidence for the existing opt-in `SwiftRetained` wrapper: a Swift class instance created by a transient compiler-oracle fixture remains alive across Rust retain/clone/drop operations and is destroyed after the final release. Preserve the zero-shipping-Swift-source rule and the no-runtime-linkage default.

This slice verifies only class-reference ownership. It does not claim support for direct Swift method calls, metadata/value witnesses, resilient values, `String`, `Optional`, throwing or async calls, Translation, StoreKit, or App Intents.

## Dependencies

- `PLAN_FOUNDATION.md` is integrated
- the `swift-abi-core` and `swift-abi-generated` crates are integrated
- the scalar `swiftcall` compiler-oracle proof is integrated
- macOS with Xcode/Swift and the Rust host toolchain is available

Read first:
- `PLAN_SWIFT_ABI.md`
- `docs/SWIFT_ABI.md`
- `docs/swift-abi/SYNCHRONOUS_BOUNDARY.md`
- `interop/swift-abi-generated/fixtures/runtime-ownership.md`
- `interop/swift-abi-core/src/lib.rs`
- `interop/swift-abi-core/src/retained.rs`

## Write scope

- `interop/swift-abi-core/**`
- `interop/swift-abi-generated/**` only if compiler evidence requires a correction
- `docs/swift-abi/SYNCHRONOUS_BOUNDARY.md`
- `docs/swift-abi/OWNERSHIP_RUNTIME_PROOF.md`
- `.github/workflows/ci.yml` only to add the focused macOS proof gate

Do not edit public portable APIs, root workspace membership, capability crates, general Swift ABI value/async crates, or Apple API backends. Do not commit any `.swift` file or generated shipping Swift source.

## Required proof

- Generate all Swift oracle input under a temporary directory outside the checkout and remove it on exit.
- Create a minimal Swift class fixture whose lifetime can be observed without sleeps or external services.
- Exercise `SwiftRetained::from_owned_ptr`, `retain_borrowed`, `Clone`, `Drop`, and `into_raw`/re-adoption from a Rust host caller where those operations can be checked deterministically.
- Verify that the class remains alive while any owned wrapper exists and is destroyed exactly once after the final strong release.
- Preserve the compiler-derived runtime symbols and ABI declarations; do not infer them from a guessed signature.
- Retain device/simulator target checks for the runtime binding and report whether they are compile, link, or runtime evidence. Do not claim device/simulator execution without it.
- Confirm the default `swift-abi-core` feature path does not import or link `swiftCore`.
- Confirm no Swift source remains in the repository or the generated proof artifact path.

## Validation and handoff

- Run the new ownership proof on macOS.
- Run device and simulator checks available for `swift-abi-core` and `swift-abi-generated`.
- Run `cargo fmt --all -- --check`, focused Clippy/tests, and `git diff --check`.
- If the live class proof cannot be linked through a supported toolchain path, record the exact compiler/linker failure and keep the result as compile-only evidence; do not weaken ownership assertions or change runtime semantics to make the proof pass.
- Update the ABI boundary docs with exact commands, toolchain, runtime imports, ownership behavior, and all limits.
- Report changed files, commit SHA, commands/results, deviations, and unresolved assumptions. Do not push.

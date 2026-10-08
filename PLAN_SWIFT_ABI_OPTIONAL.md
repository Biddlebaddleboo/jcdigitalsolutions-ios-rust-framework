# PLAN_SWIFT_ABI_OPTIONAL.md — Workstream C3: Swift Optional String Boundary Proof

## Objective

Determine whether the active Swift compiler can expose `Optional<String>` through a generated C++ header, then reach it from Rust through a fixed-width C ABI shim

This is a compiler and toolchain proof, not a general Swift value runtime or a production ABI module

## Dependencies

- Foundation A and selected G tooling are integrated
- C1 Swift ownership proof and C2 `String` C++ proof are integrated
- Use the active Xcode/Swift compiler; do not guess an Optional layout or symbol

## Read first

- `PLAN.md`
- `PLAN_SWIFT_ABI.md`
- `PLAN_SWIFT_ABI_STRING.md`
- `docs/SWIFT_ABI.md`
- `docs/research/MINIMUM_SWIFT_ABI_PRIMITIVES.md`
- `docs/research/CLANG_SWIFT_ABI_BACKENDS.md`
- `docs/swift-abi/STRING_CXX_PROOF.md`
- `interop/swift-abi-core/tests/check-swift-string-cxx.sh`

## Write scope

- `interop/swift-abi-core/tests/check-swift-optional-cxx.sh`
- `docs/swift-abi/OPTIONAL_CXX_PROOF.md`
- `PLAN_SWIFT_ABI.md` only to link this C3 decomposition

Keep all Swift input, generated C++ headers/source, IR, objects, libraries, and binaries in a temporary path outside the checkout. Do not add `.swift` files, generated headers, shipping C++ wrappers, Swift value storage, runtime metadata parsing, Translation code, production ABI APIs, CI edits, or edits to common Swift ABI modules. The orchestrator owns CI

## Proof requirements

- Use temporary public Swift functions that accept and return `String?`, plus a byte-array helper if required to preserve embedded NUL
- Generate the public C++ header with documented compiler flags. If the active compiler cannot expose the optional value, record the exact diagnostic and stop; do not use a guessed layout or private compiler flag
- Build a temporary C++ shim that maps `none`/`some` to explicit fixed-width status, pointer, and length fields. No Swift or C++ value layout may cross into Rust
- On the host, verify `none`, `some` empty, ASCII, embedded NUL, BMP, combining marks, and multi-scalar emoji. Assert exact bytes, lengths, and optional state
- Inspect generated header/IR/object for compiler-authored Optional and String copy/destroy behavior; do not infer ownership from C++ syntax alone
- Compile Swift, C++, and no_std Rust caller objects for iOS device and simulator. Mark target results compile-only unless separately linked and run
- Inspect host runtime loads and symbols; confirm one Swift core runtime and that default `swift-abi-core` features remain free of Swift runtime linkage
- Record exact Xcode, Swift, Clang, Rust, SDK, target, generated API, and all failed probes
- Do not claim a stable cross-toolchain C++ ABI, production Swift Optional adapter, Translation readiness, binary-size benefit, or App Store packaging from this proof

## Validation and handoff

- Run the new proof script on macOS/Xcode, `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, shell syntax validation, and `git diff --check`
- Do not add a unit-test harness or production ABI API in this workstream
- Report changed files, commit SHA, exact compiler/toolchain data, generated API/signatures, fixture results, ownership evidence, target compile results, runtime/linkage evidence, exact failures, deviations, and open assumptions. Do not push

## Compiler reference

- [Swift to C++ interoperability vision](https://github.com/swiftlang/swift-evolution/blob/main/visions/using-swift-from-c%2B%2B.md)

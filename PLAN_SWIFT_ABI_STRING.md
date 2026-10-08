# PLAN_SWIFT_ABI_STRING.md — Workstream C2: Swift String Boundary Proof

## Objective

Determine whether the installed Swift compiler can generate a supported C++ interoperability wrapper for a concrete Swift `String` round-trip that a Rust caller can reach through a narrow C ABI. This is a compiler/toolchain proof, not a general Swift value runtime and not yet a Translation implementation.

## Dependencies

- Foundation A and selected G tooling are integrated.
- C1 `PLAN_SWIFT_ABI_OWNERSHIP.md` and the scalar `swiftcall` probe are integrated.
- Use the active Xcode/Swift compiler; do not copy a guessed ABI layout or symbol.

## Read first

- `PLAN.md`
- `PLAN_SWIFT_ABI.md`
- `docs/SWIFT_ABI.md`
- `docs/research/CLANG_SWIFT_ABI_BACKENDS.md`
- `docs/research/MINIMUM_SWIFT_ABI_PRIMITIVES.md`
- `docs/swift-abi/SYNCHRONOUS_THUNK_FEASIBILITY.md`
- `docs/swift-abi/OWNERSHIP_RUNTIME_PROOF.md`
- `interop/swift-abi-core/tests/check-scalar-swiftcall.sh`

## Write scope

- `interop/swift-abi-core/tests/check-swift-string-cxx.sh`
- `docs/swift-abi/STRING_CXX_PROOF.md`
- `PLAN_SWIFT_ABI.md` only to link this C2 decomposition

Keep all Swift source, generated C++ headers, generated C++ source, IR, objects, libraries, and test binaries in a temporary directory outside the checkout. Do not add `.swift` files, generated headers, shipping C++ wrappers, Swift value storage, runtime metadata parsing, Translation code, or edits to common Swift ABI modules. The orchestrator owns CI and global documentation indexes.

## Proof requirements

- Use ephemeral Swift input with a public function that accepts and returns `String`; compile it with the active compiler's C++ interoperability/header-generation mode.
- Record the exact supported compiler invocation and generated public C++ API. If the SDK/toolchain cannot generate a C++ header from a Swift test module or Apple `.swiftinterface`, record the exact error and stop; do not rely on private compiler flags or undocumented build metadata.
- Build a temporary C++ shim over the compiler-generated API. Expose only a fixed-width pointer-plus-length C boundary to a small Rust caller; do not pass C++ or Swift layouts to Rust.
- On the host, round-trip deterministic UTF-8 fixtures: empty text, ASCII, embedded NUL, non-ASCII BMP text, combining marks, and multi-scalar emoji. Check exact bytes and lengths; no lossy normalization.
- Exercise ownership/destruction of generated Swift `String` wrappers. Inspect the generated header/IR/object for compiler-authored retain/copy/destroy operations; do not infer ownership from C++ syntax alone.
- Compile the Swift oracle, C++ shim, and Rust caller for iOS device and simulator. State clearly whether evidence is compile-only or linked/runtime.
- Inspect host runtime loads and symbols; confirm no duplicate Swift runtime and no repository Swift source. The default Swift ABI feature path must remain free of Swift runtime linkage.
- Do not claim the C++ interop approach is a permanent production backend merely because the probe passes. Record toolchain stability, dependency, runtime, binary, and App Store packaging limitations for a later design decision.

## Validation and handoff

- Run the new script on macOS/Xcode, `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, shell syntax validation, and `git diff --check`.
- Do not add a unit-test harness or production ABI API in this workstream.
- Report exact Xcode, Swift, Clang, and Rust versions; generated API/signatures; fixture results; ownership evidence; target compile results; runtime/linkage evidence; exact failures; deviations; and unresolved assumptions. Do not push.

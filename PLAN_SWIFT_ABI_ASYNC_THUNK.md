# PLAN_SWIFT_ABI_ASYNC_THUNK.md — Workstream C5: Clang Swift Async Thunk Feasibility

## Status

C5 passes `sh interop/swift-abi-core/tests/check-swift-async-thunk.sh`: Clang's host, device, and
simulator lowering matches the compiler-derived `swifttailcc` entry. The probe does not invoke an
async entry or establish task-context, executor, resume, or error ownership; no Rust async adapter
is claimed. Xcode 26.6 / SDK 26.5 remains below the Xcode 27.x plan baseline.

## Objective

After C4 found that generated C++ headers omit public Swift `async throws` functions, determine whether Clang `swiftasynccall` can reproduce a Swift-compiler-derived async entry lowering for a minimal scalar API

This is a bounded compiler/ABI feasibility proof, not a runtime adapter or production API

## Dependencies

- Foundation A and shared G tooling are integrated
- C1 ownership, C2 `String`, C3 `Optional<String>`, and C4 async-header proofs are integrated
- Use the active Xcode/Swift/Clang toolchain; do not assume the Xcode 27.x baseline is available

## Read first

- `PLAN.md`
- `PLAN_SWIFT_ABI.md`
- `PLAN_SWIFT_ABI_ASYNC.md`
- `docs/SWIFT_ABI.md`
- `docs/research/CLANG_SWIFT_ABI_BACKENDS.md`
- `docs/research/MINIMUM_SWIFT_ABI_PRIMITIVES.md`
- `docs/swift-abi/ASYNC_CXX_FEASIBILITY.md`

## Write scope

- `interop/swift-abi-core/tests/check-swift-async-thunk.sh`
- `docs/swift-abi/ASYNC_THUNK_FEASIBILITY.md`
- `PLAN_SWIFT_ABI.md` only to link this C5 decomposition

Keep temporary Swift input, generated headers, SIL, LLVM IR, assembly, objects, libraries, and binaries outside the repository. Add no `.swift` files, generated shipping headers, production C/C++/Rust wrappers, async runtime state, capability implementation, or CI changes

## Proof requirements

- Use temporary public Swift declarations for `async -> Int32` and `async throws -> Int32`; include a compiler-generated Swift async caller as an oracle
- Record exact toolchain, SDK, targets, flags, mangled symbols, SIL/LLVM IR signatures, async-context/resume details, and compiler output
- Derive any Clang declaration only from the compiler output. Use `swiftasynccall` and `swift_async_context` only when the active public Clang surface can express the exact lowered signature
- Compare Clang-emitted LLVM IR and target assembly/object ABI details with the Swift compiler oracle; do not infer equivalence from matching symbol names
- Invoke a function only if compiler-emitted code establishes a valid task/context lifetime, resume path, executor behavior, and error delivery without guessed runtime internals. Otherwise stop before execution and record the precise blocker
- Compile device and simulator objects only when the derived declaration is representable for those targets; mark compile-only evidence as such
- Inspect symbols and runtime/linkage evidence, but distinguish observed loads/imports from actual caller requirements
- Do not use guessed signatures, private compiler flags, hidden metadata, handwritten assembly, or private Apple APIs
- Do not claim a stable cross-toolchain async ABI, task/executor integration, cancellation bridge, Swift error ownership contract, Rust `Future` adapter, Translation support, or StoreKit support from this proof

## Validation and handoff

- Run the proof script on macOS/Xcode, `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, shell syntax validation, and `git diff --check`
- Add no unit-test harness or production ABI API
- Report changed files, commit SHA, compiler/SDK facts, compiler-derived signatures, ABI comparison, execution or stop result, target evidence, runtime/linkage facts, deviations, and open assumptions. Do not push

## References

- [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html)
- [Swift ABI Calling Convention Summary](https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst)
- [Swift async/await proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md)

# PLAN_SWIFT_ABI_ASYNC.md — Workstream C4: Swift Async C++ Interop Feasibility

## Status

C4 passes as the specified feasibility stop: `sh interop/swift-abi-core/tests/check-swift-async-cxx.sh`
confirms the generated header exposes the synchronous control but marks both `async` entries
unavailable in C++. The script exits successfully after that expected omission; no C++/Rust async
caller or task runtime is claimed. Xcode 26.6 / SDK 26.5 remains below the Xcode 27.x plan baseline.

## Objective

Use the active Swift compiler to determine whether its generated C++ API can call a public `async throws` Swift function, and record a safe fixed-width C boundary if supported

This is a compiler feasibility proof, not a Swift task runtime, production async adapter, or Translation implementation

## Dependencies

- Foundation A and selected G tooling are integrated
- C1 ownership proof, C2 `String` proof, and C3 `Optional<String>` proof are integrated
- Use the active Xcode/Swift compiler and public flags; do not guess async ABI details

## Read first

- `PLAN.md`
- `PLAN_SWIFT_ABI.md`
- `PLAN_SWIFT_ABI_STRING.md`
- `PLAN_SWIFT_ABI_OPTIONAL.md`
- `docs/SWIFT_ABI.md`
- `docs/research/CLANG_SWIFT_ABI_BACKENDS.md`
- `docs/research/MINIMUM_SWIFT_ABI_PRIMITIVES.md`
- `docs/swift-abi/STRING_CXX_PROOF.md`
- `docs/swift-abi/OPTIONAL_CXX_PROOF.md`

## Write scope

- `interop/swift-abi-core/tests/check-swift-async-cxx.sh`
- `docs/swift-abi/ASYNC_CXX_FEASIBILITY.md`
- `PLAN_SWIFT_ABI.md` only to link this C4 decomposition

Keep all Swift input, generated C++ header/source, IR, objects, libraries, and binaries in an external temporary path. Do not add `.swift` files, generated headers, shipping C++ wrappers, production ABI APIs, async runtime state, Translation code, CI edits, or changes to common ABI modules. The orchestrator owns CI

## Proof requirements

- Use temporary public Swift declarations for a synchronous `String` control, `async -> String`, and `async throws -> String`
- Ask only public compiler modes to emit a C++ header. Record whether each declaration appears, its exact generated signature, or the exact compiler diagnostic/omission. If no supported async entry exists, stop the call-path probe and record the result
- If the compiler exposes an async API, build a temporary C++ caller with the public generated surface, then expose completion to Rust only through fixed-width C fields, callbacks, or opaque operation handles. Do not pass C++/Swift layouts to Rust
- If a complete caller path is representable, verify one success and one Swift error, exactly-once completion, and String ownership. Record cancellation support only if the generated public API exposes it
- Compile matching Swift/C++/no_std Rust objects for iOS device and simulator if the public async API can be represented. Mark target results compile-only unless linked and run
- Inspect the generated declarations, symbols, host runtime loads, executor/task requirements, and C++ runtime linkage. Do not infer a supported ABI from a mangled symbol alone
- Record exact Xcode, Swift, Clang, Rust, SDK, target, flags, generated API, runtime requirements, and all failed probes
- Do not use a guessed `swiftasynccall` signature, private compiler flag, hidden metadata, or handwritten assembly as a fallback in this proof
- Do not claim a stable cross-toolchain async ABI, Rust async adapter, cancellation bridge, Translation support, binary-size win, or App Store packaging from this proof

## Validation and handoff

- Run the proof script on macOS/Xcode, `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, shell syntax validation, and `git diff --check`
- Do not add a unit-test harness or production ABI API in this workstream
- Report changed files, commit SHA, compiler and SDK facts, generated declarations or exact errors, success/error results where supported, target evidence, runtime/linkage facts, deviations, and open assumptions. Do not push

## References

- [Swift async/await proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md)
- [Swift to C++ interoperability vision](https://github.com/swiftlang/swift-evolution/blob/main/visions/using-swift-from-c%2B%2B.md)

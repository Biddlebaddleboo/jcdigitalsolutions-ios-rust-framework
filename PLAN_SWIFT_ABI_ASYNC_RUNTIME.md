# PLAN_SWIFT_ABI_ASYNC_RUNTIME.md — Workstream C6: Public Swift Async Task Entry Feasibility

## Objective

Determine whether a supported public Apple/Swift C or C++ interface can create, own, and resume the Swift task context required to call a compiler-derived async entry from Rust without Swift source

This is a bounded interface audit and compiler-oracle study, not an async runtime or production adapter

## Dependencies

- Foundation A and shared G tooling are integrated
- C1 ownership, C2 `String`, C3 `Optional<String>`, C4 async-header, and C5 async-thunk proofs are integrated
- Use the installed Xcode SDK/runtime and public Apple/Swift documentation; do not assume the Xcode 27.x baseline is available

## Read first

- `PLAN.md`
- `PLAN_SWIFT_ABI.md`
- `PLAN_SWIFT_ABI_ASYNC.md`
- `PLAN_SWIFT_ABI_ASYNC_THUNK.md`
- `docs/SWIFT_ABI.md`
- `docs/research/CLANG_SWIFT_ABI_BACKENDS.md`
- `docs/research/MINIMUM_SWIFT_ABI_PRIMITIVES.md`
- `docs/swift-abi/ASYNC_THUNK_FEASIBILITY.md`
- `crates/framework-async/src/operation.rs`

## Write scope

- `interop/swift-abi-core/tests/check-swift-async-runtime-entry.sh`
- `docs/swift-abi/ASYNC_RUNTIME_ENTRY_FEASIBILITY.md`
- `PLAN_SWIFT_ABI.md` only to link this C6 decomposition

Keep temporary Swift input, generated SIL/LLVM IR/assembly, objects, binaries, symbol maps, and inspection output outside the repository. Add no `.swift` files, production C/C++/Rust wrappers, Swift runtime state, capability implementation, or CI changes

## Audit requirements

- Use compiler-generated Swift async callers as the oracle for task creation, context allocation/lifetime, resume setup, executor switching, success/error delivery, and task teardown
- Inspect public Swift runtime headers, installed SDK interfaces/module maps, supported Apple documentation, and exported symbols for a documented C/C++ task-entry or async-call API
- Distinguish a documented public contract from a symbol that is merely exported, a Swift compiler implementation detail, or an observation from one toolchain
- If a public entry facility exists, record its exact contract, availability, ownership, executor, cancellation, error, and deployment requirements; do not invoke it unless those semantics are complete and compiler-derived
- If no supported public facility is found, state the audit scope and exact blocker without claiming that no internal symbol exists
- Do not decode private descriptors, fabricate async context layouts, call undocumented `swift_task_*` symbols, use guessed executor rules, or use handwritten assembly
- Explicitly distinguish `framework-async::OperationState<T,E>` completion/waker semantics from Swift task/context creation and resume semantics
- Record Xcode, Swift, Clang, SDKs, targets, files/paths searched, exact symbols or declarations found, and all failed probes
- Do not claim a stable async ABI, Rust `Future` adapter, cancellation bridge, Translation support, StoreKit support, or App Store packaging from this audit

## Validation and handoff

- Run the audit script on macOS/Xcode, `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, shell syntax validation, and `git diff --check`
- Add no unit-test harness, production API, or live Apple API call
- Report changed files, commit SHA, toolchain and searched interfaces, public declarations or exact no-go result, evidence limits, deviations, and unresolved assumptions. Do not push

## References

- [Swift ABI Calling Convention Summary](https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst)
- [Swift async/await proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md)
- [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html)

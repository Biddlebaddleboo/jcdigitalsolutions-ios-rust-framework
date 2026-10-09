# PLAN_BINDINGS_SECURE_STORAGE.md — Workstream F2: Secure-Storage C ABI

## Status

F2's opt-in C ABI is implemented. The device and simulator static libraries link through a C probe
that calls read, store, and remove. Each probe Mach-O has the exact direct import set
`CoreFoundation`, `Security`, and `libSystem.B.dylib`; its undefined-symbol scan requires
Keychain/CoreFoundation APIs and rejects Swift, Objective-C, and network symbols. This evidence
applies to the probe, not arbitrary app links. The ABI manifest now records the invalid-input,
unsupported-policy, non-iOS, backend, platform-code, allocation, and panic status mappings; the
check script validates that record and exact header-to-manifest symbol parity. Deterministic Rust
mapping tests now pass for every known error-kind status, non-iOS output initialization, policy and
identifier validation, and panic containment. Eight targeted tests passed; this does not claim a
live Keychain or physical-device test

## Objective

Add a capability-scoped synchronous C ABI for the stable secure-storage contract, backed on iOS by `IosSecureStorage`. Keep the C layer as an opt-in foreign-language boundary; Rust callers continue to call `SecureStorage<IosSecureStorage>` directly.

## Dependencies

- Foundation A is integrated.
- F1 core C ABI is integrated.
- D2 `framework-secure-storage` contract is integrated.
- B2 `ios-secure-storage` Keychain backend is integrated.

## Write scope

- `bindings/c/**`
- `docs/bindings/secure-storage.md`

Do not edit `crates/framework-abi/**`, secure-storage contracts/backends, root workspace configuration, capability manifest, or unrelated bindings. The orchestrator owns dependency/lockfile reconciliation and shared manifest updates.

## API requirements

- Add a separate `framework_ios_secure_storage.h` capability header; do not put Keychain declarations in the core `framework.h` header.
- Make the C API an explicit opt-in Cargo feature and keep the iOS backend dependency optional and capability-scoped.
- Export synchronous read, store, and remove operations over the existing opaque bytes/service/item semantics.
- Use `FrameworkStr`, `FrameworkSlice`, `FrameworkOwnedBuffer`, `FrameworkStatus`, fixed-width policy flags, explicit output parameters, and optional native `OSStatus` output. Do not expose Rust-owned type layout beyond existing C ABI declarations.
- Read must distinguish missing from an existing zero-length secret. Store must report its effective policy. Remove must report whether an item existed.
- Reject invalid UTF-8, empty/NUL-containing identifiers, unknown policy flags, malformed pointer/length pairs, and missing required output pointers without calling Keychain.
- Preserve a nonzero native `OSStatus` when one exists. Set optional native-code outputs to zero for non-platform errors and before any fallible work.
- Initialize all required output parameters before work; return owned read bytes through `FrameworkOwnedBuffer` and require the F1 destroyer.
- Contain panics within each exported boundary; no panic may unwind across C.
- On non-iOS targets, compile the optional feature as explicit `UNSUPPORTED` stubs and do not link Apple frameworks.
- Avoid process-global state, runtime registries, callbacks, async runtime, and hidden initialization.

## ABI and consumer checks

- Add C11 and C++17 header compile coverage for the capability header.
- Test known policy flags/output semantics and validate the capability symbol list.
- Link and run a host C consumer against the explicit non-iOS `UNSUPPORTED` stubs when toolchain support permits.
- Build the iOS static library with the feature for `aarch64-apple-ios` and `aarch64-apple-ios-sim`; link a C probe that calls read, store, and remove; use `otool -L` to enforce the exact direct imports `CoreFoundation`, `Security`, and `libSystem.B.dylib`; use `nm -u` to require Keychain/CoreFoundation symbols and reject Swift, Objective-C, network, and unrelated framework imports for that probe
- Exercise the Rust mapping layer with deterministic tests; link checks do not satisfy this criterion. Do not claim a live Keychain or physical-device test unless one is run
- Update the ABI manifest and binding documentation with symbols, ownership, status/native-code mapping, policy bits, synchronous/blocking behavior, target support, and validation limits.

## Non-goals

- No asynchronous C API, callbacks, handle object, cancellation, or access-group configuration.
- No C++ convenience API or Python binding.
- No Swift, Objective-C, or custom Security ABI declarations.
- No changes to the portable D2 contract or B2 Keychain semantics.

## Handoff

Report changed files, commit SHA, exported symbols, ownership rules, exact host/device/simulator commands and results, deviations, and unresolved assumptions. The orchestrator integrates shared Cargo/manifest updates.

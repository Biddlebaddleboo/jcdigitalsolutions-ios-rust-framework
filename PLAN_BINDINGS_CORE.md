# PLAN_BINDINGS_CORE.md — Workstream F1: Core C ABI

## Status

F1's core C ABI, hand-maintained header, ABI manifest, and minimal consumer are integrated. A
follow-up audit corrected the C++ fixture's function-pointer representation check. ABI 1.0 layout,
symbol, and header checks pass; `sh bindings/c/check.sh` linked and ran the minimal C consumer. No
Rust test suite was run and no new test was added. `FrameworkOptionsV1` layout is checked, but no
public C function takes that record, so no test or promise for larger same-major `struct_size`
values exists yet. The focused C check now compares every public `FRAMEWORK_STATUS_*` macro value
with `status_codes` in `bindings/c/abi-manifest.json`

## Objective

Expose the stable foundational values in `framework-abi` through a small, linkable C library and a hand-maintained C header. This slice covers the ABI version and core value/ownership types only; it does not invent capability APIs before their D contracts are stable.

## Dependencies

- Foundation A is integrated
- Workstream G host tooling is integrated
- Capability crates are not required

## Write scope

- `bindings/c/**`
- `examples/c-minimal/**`
- C ABI documentation and focused ABI checks

Do not edit `crates/framework-abi/**`, capability crates, Swift ABI, iOS backends, root workspace files, or shared G tooling. The orchestrator adds the workspace glob after the package exists.

## Required surface

- A `framework-c-api` static library over the existing `framework-abi` crate; Rust callers continue to use the Rust-native crates directly.
- Export `framework_abi_version()` as a fixed-width major/minor value sourced from `ABI_VERSION_MAJOR` and `ABI_VERSION_MINOR`.
- Hand-maintained C11 header declarations for `FrameworkStatus`, `FrameworkSlice`, `FrameworkStr`, `FrameworkOwnedBuffer`, `FrameworkOperationHandle`, `FrameworkErrorHandle`, `FrameworkCompletionCallback`, and `FrameworkOptionsV1`.
- Declare `framework_owned_buffer_destroy` with exact pointer and ownership rules from `framework-abi`.
- Keep all scalar fields fixed-width; pointer fields are permitted only for actual addresses. Do not expose Rust layouts beyond the specified `#[repr(C)]` declarations.
- Record ABI version, struct size/alignment, field offsets, symbol names, and owned-buffer creator/destroyer rules in a machine-readable manifest or checked report.

## C consumer and compatibility checks

- Provide a minimal C example that checks the ABI version and creates/destroys no object whose creator is outside this slice.
- Compile the header from C11 and C++ translation units; C++ is only a header-compatibility check, not a C++ API.
- Link and run the C example against the built static library on the host where toolchain support permits.
- Check known struct size/alignment/offset values against Rust, and compare exported symbols with the header/ABI manifest.
- Record the V1 options layout, but do not claim that a future-size record is accepted: no public C function takes `FrameworkOptionsV1`. Any later function that takes an extensible versioned record must check `struct_size` and `abi_version` before field reads and state if a larger same-major record is valid. Do not promise forward compatibility for unversioned fields
- Preserve `catch_unwind`/panic-abort constraints: no panic may unwind across an exported C boundary.

If a required host linker or ABI probe fails, report the exact command and error; do not claim the failing check passed or hide the failure with a syntax-only substitute.

## Non-goals

- No capability-specific C functions before their D API contracts stabilize.
- No C++ convenience wrapper or Python runtime.
- No runtime registry, serialization protocol, or second implementation.
- No modification of Rust-native call paths.

## Handoff

Report exported ABI version, header modules, ownership table, struct layout results, symbol audit, C/C++ compile and link results, changed files, and unresolved host limitations.

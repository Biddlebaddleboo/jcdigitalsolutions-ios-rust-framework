# PLAN_BINDINGS_CORE.md — Workstream F1: Core C ABI

## Status

F1's core C ABI, hand-maintained header, ABI manifest, and minimal consumer are integrated. A
follow-up audit corrected the C++ fixture's function-pointer representation check. The ABI 1.0
baseline layout, symbol, and header checks passed; `sh bindings/c/check.sh` linked and ran the
minimal C consumer at that baseline. The additive `framework_options_v1_validate` export moves the
current ABI to 1.1 without changing existing layouts or status codes. It accepts the 16-byte
`FrameworkOptionsV1` prefix and larger records only for ABI major 1, requires `reserved == 0`, and
ignores flags and trailing bytes. The 1.1 follow-up received non-test format/build/symbol,
C11/C++17 syntax, and rustdoc checks; neither the minimal C consumer nor Rust test suite was run. The
focused C check compares public `FRAMEWORK_STATUS_*` macro values with `status_codes` in
`bindings/c/abi-manifest.json`

Follow-up evidence:

- `cargo fmt --package framework-abi -- --check` and `cargo fmt --package framework-c-api -- --check` passed
- `cargo check --locked -p framework-c-api --no-default-features` and `cargo clippy --locked -p framework-c-api --no-default-features -- -D warnings` passed
- `cargo build --locked --release -p framework-c-api --no-default-features` passed; `nm -gU` export names matched `c_symbols` in `bindings/c/abi-manifest.json`
- After checkpoint `0c83378`, `cargo +1.94.1 build --locked --release -p framework-c-api --no-default-features` completed. A clean isolated target then forced a fresh build: `CARGO_TARGET_DIR=target/abi11-fresh-target cargo +1.94.1 build --locked --release -p framework-c-api --no-default-features` compiled `framework-core`, `framework-abi`, and `framework-c-api`; its archive SHA-256 was `a78bdf35b8988245aec0a9d504943618c0c126730316f9e38d987b8c79fe983d`
- `sh -n bindings/c/check.sh`, manifest/source/header contract assertions, and `git diff --check` passed
- C11 header, `examples/c-minimal/main.c`, and C++17 header syntax checks passed; ABI 1.1 C and C++ link-only apps that call `framework_options_v1_validate` also compiled and linked against the fresh `target/abi11-fresh-target/release/libframework_c_api.a`
- `nm -u target/abi11-link-only/c_consumer.o` and `nm -u target/abi11-link-only/cpp_consumer.o` each showed only `_framework_options_v1_validate`; `nm -u target/abi11-link-only/c_consumer` and `nm -u target/abi11-link-only/cpp_consumer` showed no undefined symbols. `nm -gU target/abi11-fresh-target/release/libframework_c_api.a | rg 'framework_options_v1_validate|framework_abi_version'` found both archive exports. `otool -L target/abi11-link-only/c_consumer` and `otool -L target/abi11-link-only/cpp_consumer` listed only `/usr/lib/libSystem.B.dylib`; `otool -Iv` on both listed no import records
- The exact compile and link commands were `clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -c target/abi11-link-only/c_consumer.c -o target/abi11-link-only/c_consumer.o`, `clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -fno-exceptions -fno-rtti -I bindings/c/include -c target/abi11-link-only/cpp_consumer.cpp -o target/abi11-link-only/cpp_consumer.o`, `clang -std=c11 target/abi11-link-only/c_consumer.o target/abi11-fresh-target/release/libframework_c_api.a -o target/abi11-link-only/c_consumer`, and `clang++ -std=c++17 -fno-exceptions -fno-rtti -nostdlib++ target/abi11-link-only/cpp_consumer.o target/abi11-fresh-target/release/libframework_c_api.a -o target/abi11-link-only/cpp_consumer`; neither binary ran
- The initial `cargo doc --locked --no-deps -p framework-c-api --no-default-features` attempt stopped before rustdoc with `error: cannot update the lock file ... because --locked was passed to prevent this`; Cargo suggested removing `--locked` and using `--offline`. After root refreshed the shared lock, the same command passed and generated `target/doc/framework_c_api/index.html`; this work did not edit `Cargo.lock`
- `clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -fno-exceptions -fno-rtti -I bindings/c/include -I bindings/cpp/include -fsyntax-only bindings/cpp/tests/ownership.cpp` passed after its ABI fixture return value was updated to 1.1
- No Rust test, C app, C++ app, or probe was run for ABI 1.1, and no passing CI workflow run is recorded. Link resolution passed on this host; validator runtime behavior remains unverified

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

For the ABI 1.1 follow-up, the only `framework-abi` edit is `ABI_VERSION_MINOR`; do not change its types or behavior. Do not edit capability crates, Swift ABI, iOS backends, root workspace files, or shared G tooling. The orchestrator adds the workspace glob after the package exists.

## Required surface

- A `framework-c-api` static library over the existing `framework-abi` crate; Rust callers continue to use the Rust-native crates directly.
- Export `framework_abi_version()` as a fixed-width major/minor value sourced from `ABI_VERSION_MAJOR` and `ABI_VERSION_MINOR`.
- Hand-maintained C11 header declarations for `FrameworkStatus`, `FrameworkSlice`, `FrameworkStr`, `FrameworkOwnedBuffer`, `FrameworkOperationHandle`, `FrameworkErrorHandle`, `FrameworkCompletionCallback`, and `FrameworkOptionsV1`.
- Declare `framework_owned_buffer_destroy` with exact pointer and ownership rules from `framework-abi`.
- Export `framework_options_v1_validate` as the core validator for the 16-byte V1 options prefix; accept larger same-major records while ignoring unknown flags and trailing bytes.
- Keep all scalar fields fixed-width; pointer fields are permitted only for actual addresses. Do not expose Rust layouts beyond the specified `#[repr(C)]` declarations.
- Record ABI version, struct size/alignment, field offsets, symbol names, and owned-buffer creator/destroyer rules in a machine-readable manifest or checked report.

The options validator returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` for null input, a declared size
below 16, or nonzero `reserved`; it returns `FRAMEWORK_STATUS_UNSUPPORTED` for an ABI major other
than 1. Non-null input must be fully initialized, aligned, and readable for a complete
`FrameworkOptionsV1` through the call, and must not be mutated unsynchronized. The validator reads
no `flags` or trailing bytes and retains no pointer. This acceptance rule applies only to this
validator; other record-consuming functions must check their own supported prefix before later
field reads

## C consumer and compatibility checks

- Provide a minimal C example that checks the ABI version and creates/destroys no object whose creator is outside this slice.
- Compile the header from C11 and C++ translation units; C++ is only a header-compatibility check, not a C++ API.
- Link and run the C example against the built static library on the host where toolchain support permits.
- Check known struct size/alignment/offset values against Rust, and compare exported symbols with the header/ABI manifest.
- Record the V1 options layout and validate its extensible prefix before any later fields are read. This core validator accepts larger same-major records but does not authorize other functions to read beyond their own supported prefix. Do not promise forward compatibility for unversioned fields
- Preserve `catch_unwind`/panic-abort constraints: no panic may unwind across an exported C boundary.

If a required host linker or ABI probe fails, report the exact command and error; do not claim the failing check passed or hide the failure with a syntax-only substitute.

## Non-goals

- No capability-specific C functions before their D API contracts stabilize.
- No C++ convenience wrapper or Python runtime.
- No runtime registry, serialization protocol, or second implementation.
- No modification of Rust-native call paths.

## Handoff

Report exported ABI version, header modules, ownership table, struct layout results, symbol audit, C/C++ compile and link results, changed files, and unresolved host limitations.

# PLAN_BINDINGS_CPP.md — Workstream F3: Header-Only C++ Convenience Layer

## Objective

Add a small C++17 convenience header over the existing stable core C ABI without a second implementation, hidden allocation, or new runtime dependency.

## Dependencies

Requires `PLAN_BINDINGS_CORE.md` and the integrated core C ABI.

## Write scope

- `bindings/cpp/**`
- C++ consumer checks and binding docs

Do not change C ABI symbols/layouts, Rust capability APIs, workspace dependency policy, or platform backends.

## Required surface

- Add a header-only `framework.hpp` that includes the stable C header.
- Provide a small `AbiVersion` view over `framework_abi_version()`.
- Provide a move-only RAII owner for `FrameworkOwnedBuffer` that stores a pointer to the original descriptor and calls `framework_owned_buffer_destroy` exactly once.
- Preserve the C ABI rule: never copy or mutate an owned-buffer descriptor; the original descriptor must outlive its RAII view.
- Add no heap allocation, exception translation runtime, static registry, or C++ implementation source.
- Keep capability-specific wrappers out of this core slice.

## Validation and handoff

- Compile a C++17 consumer against the new header.
- Check move-only traits and single-destroy behavior with a test stub.
- Confirm the consumer links only the existing core C ABI symbols.
- Run the existing C ABI checks plus the focused C++ check, formatting, and `git diff --check`.
- Report changed files, commit SHA, checks, deviations, and unresolved assumptions.

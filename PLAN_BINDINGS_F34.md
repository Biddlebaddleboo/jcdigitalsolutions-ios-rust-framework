# PLAN_BINDINGS_F34.md — F34: transfer-aware C++ owned-buffer adoption

## Objective

Extend the existing opt-in, header-only C++17 C-ABI convenience layer with an explicit factory that adopts a `FrameworkOwnedBuffer` only when a C API's ownership/presence output says ownership transferred. The base `framework::OwnedBuffer` already supplies move-only RAII destruction; F34 makes the ownership signal explicit at call sites and leaves absent outputs unguarded

## Scope and invariants

- Product implementation: `bindings/cpp/include/framework.hpp`
- Consumer contract: `docs/bindings/cpp.md`
- This is a C++-only refinement; add no C symbols, Rust exports, Cargo features, dependencies, heap allocations, C++ source files, or runtime initialization
- Keep the existing direct `OwnedBuffer(FrameworkOwnedBuffer&)` constructor for APIs that unconditionally transfer ownership
- `OwnedBuffer::from_transfer(descriptor, ownership_transferred)` stores the original descriptor address only when the explicit Boolean is true; false creates an empty moved-from-equivalent guard that never calls the C destroyer
- A true ownership signal still permits exactly one `framework_owned_buffer_destroy` call over the original unchanged descriptor; move construction transfers that responsibility and move assignment releases the destination once before taking the source
- Never use buffer length to infer presence; present empty outputs are owned when the C API says so

## Existing C API examples

- `framework_ios_secure_storage_read`: `out_found == 1` transfers `out_secret`
- `framework_ios_preferences_get`: `out_found == 1` transfers `out_value`
- `framework_ios_clipboard_read`: `out_has_value == 1` transfers `out_text`
- Keep a containing result record alive at its original address until guard destruction; do not make a second guard or call the C destroy function while the guard is live

## Acceptance and validation

- [x] Add the inline `OwnedBuffer::from_transfer` factory with no new symbol or dependency
- [x] Preserve move-only semantics and direct-constructor compatibility
- [x] Document explicit ownership signals and present-empty behavior in `docs/bindings/cpp.md`
- [x] Route F34 from `PLAN_BINDINGS.md`
- [x] C++17 `-fsyntax-only` compilation of a consumer exercising both transferred and absent results with exceptions/RTTI disabled
- [x] `clang-format --dry-run --Werror` and `git diff --check`

No tests, test suites, linking, or consumer execution were requested or performed

The syntax-only compile is source-level evidence only. It does not establish link, runtime, API ownership, or platform behavior

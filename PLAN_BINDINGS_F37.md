# PLAN_BINDINGS_F37.md — F37: C++ owner for error-detail objects

## Scope

Add a small header-only C++17 owner and view for F35's `FrameworkErrorDetailHandle`. The owner is move-only, directly owns the original opaque pointer, calls `framework_error_detail_destroy` exactly once, and exposes the stored status plus a borrowed UTF-8 `std::string_view` through `framework_error_detail_view`. It adds no C/Rust exports, dependencies, allocations, exception/runtime use, shared Cargo changes, or capability API.

## Contract

- Construct `framework::ErrorDetail` only from a live handle whose ownership was returned by a successful `framework_error_detail_create` call.
- Copy construction/assignment is disabled; move transfers the handle and leaves the source empty. Destruction and move assignment release an existing handle exactly once.
- `ErrorDetail::view` returns the C view-call result separately from its `ErrorDetailView` output. The stored `FrameworkStatus` is copied unchanged, including unknown codes.
- On success, `ErrorDetailView::message` borrows the object's UTF-8 bytes, including embedded NUL, and remains valid only while the owner lives. A caller must synchronize destruction with all borrows.
- The wrapper performs no message allocation or status mapping. A moved-from/empty owner returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` with default output.

## Acceptance and evidence

- [x] Add `framework::ErrorDetail` and `framework::ErrorDetailView` in `bindings/cpp/include/framework.hpp`.
- [x] Document ownership, move, status pass-through, UTF-8 borrowing, and lifetime in `docs/bindings/cpp.md`.
- [x] Route this scoped workstream from `PLAN_BINDINGS.md`.
- [x] `xcrun clang++ -std=c++17 -fsyntax-only -Ibindings/c/include -Ibindings/cpp/include /tmp/f37_error_detail_consumer.cpp` passed; the consumer includes move-only static assertions and a moved-owner view call.
- [x] `xcrun clang-format --dry-run --Werror bindings/cpp/include/framework.hpp` and `git diff --check` passed.
- [x] No tests were added or run; no link or runtime evidence is claimed.

## Limits

Syntax-only compilation establishes header and consumer syntax, not ABI linkage, C++ runtime destruction, borrowed-view behavior, or platform execution. The view lifetime and unique ownership requirements remain caller obligations.

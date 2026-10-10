# PLAN_BINDINGS_F36.md — F36: optional Python owned-byte value

## Scope

F36 creates an independent `bindings/python` extension package exposing a small idiomatic Python
value over the existing framework-owned byte ABI. `framework_python.OwnedBytes` copies Python
`bytes` through `framework_owned_buffer_copy`, exposes Python `len()`/`bytes()` semantics, and
releases the unique descriptor through `FrameworkOwnedBuffer`'s framework destructor. The scope
does not add a new portable API, C symbol, Python capability surface, async behavior, or zero-copy
view.

## Architecture and dependency rationale

- Python and PyO3 stay inside an opt-in standalone Cargo package; they are absent from portable
  crates, `framework-c-api`, and Rust-only builds.
- PyO3 is selected as the binding library because it implements CPython reference, exception,
  type, and module machinery. Its matching `pyo3-build-config` helper adds extension linker flags;
  reimplementing those details over raw CPython calls would expand unsafe code and version-specific
  ABI maintenance. The helper is build-only and adds no runtime dependency. The offline build
  resolved 16 Rust packages total: this package, `framework-abi`, `framework-core`, and 13 external
  packages, including PyO3's procedural-macro and target-detection dependencies.
- Only PyO3's `macros` and stable `abi3-py39` features are enabled. Its proc-macro/build dependency
  graph and CPython runtime are opt-in costs paid by Python extension users.
- The API boundary remains framework-owned: `FrameworkSlice` is borrowed only during the copy,
  `FrameworkOwnedBuffer` owns one `Vec<u8>` allocation, and the Python class owns that descriptor.
  The Python class exposes no PyO3 type outside `bindings/python`.
- F36 adds carefully documented `Send` and `Sync` implementations because the contained immutable
  buffer is allocated from `Vec<u8>`, is read-only through the class, and may be destroyed on any
  thread; the class exposes no mutation or raw pointer access.
- Failure maps to `PyMemoryError` for resource exhaustion, `OverflowError` for ABI/Python length
  bounds, and `RuntimeError` for unexpected ABI statuses. No panic is intended to cross the C ABI.

## Acceptance and evidence

- [x] Add `bindings/python` as a separate opt-in PyO3 extension package with no root manifest,
  root lockfile, or shared-manifest edits.
- [x] Provide `OwnedBytes(bytes)`, `len(value)`, `value.length`, and `bytes(value)` with explicit
  copy/ownership behavior.
- [x] Document the dependency, feature/build opt-in, error mapping, and limits in
  `bindings/python/README.md`.
- [x] `cargo check --offline --manifest-path bindings/python/Cargo.toml` passed.
- [x] `cargo clippy --offline --manifest-path bindings/python/Cargo.toml -- -D warnings` passed.
- [x] `cargo doc --offline --no-deps --manifest-path bindings/python/Cargo.toml` passed.
- [x] `PYO3_BUILD_EXTENSION_MODULE=1 cargo build --offline --release --manifest-path bindings/python/Cargo.toml`
  linked successfully. `nm -gU` found `_PyInit_framework_python` and
  `_framework_owned_buffer_copy`; `otool -L` found only `libiconv` and `libSystem`, with no
  `libpython` dependency.
- [x] Preserve root integration in separate commit `c7ff494` by excluding `bindings/python` from
  the root `bindings/*` workspace glob. `cargo +1.94.1 metadata --no-deps --format-version 1`
  succeeds with no `framework-python` package in root metadata. `Cargo.lock` was not part of the
  integration commit, so no root lockfile update was required. The root exclusion is:

  ```toml
  exclude = ["examples/c-minimal", "bindings/cpp", "bindings/python", "platform/ios/callkit", "tools/validation", "tools/native-build", "tools/releases", "tools/native-build-support", "tools/ios-rust-validate"]
  ```

  No root lockfile update is needed while this package stays excluded; its isolated `Cargo.lock`
  is not committed by F36.

No tests, Python runtime execution, wheel build, or import check are part of this workstream.

## Open limits

F36 proves only compile-time integration with the existing owned-buffer API. Python allocation and
class behavior, wheel generation, interpreter import, cross-thread destruction, and end-to-end
capability APIs remain unverified. Returning `bytes(value)` necessarily copies into Python-owned
storage. The package intentionally offers no memoryview or buffer protocol because those require a
separate exported-memory lifetime contract.

# Optional Python binding

`framework-python` is an opt-in CPython extension crate. Its first value type is
`framework_python.OwnedBytes`, an immutable Python object backed by the framework-owned
`FrameworkOwnedBuffer` contract. This package is isolated from the root Cargo workspace so Python
and PyO3 are not pulled into Rust- or C-only workspace builds. The extension requires CPython 3.9
or newer through PyO3's stable `abi3` API.

## Build and install

From this directory, install into the active Python environment with Maturin:

```sh
python3 -m pip install .
```

For a local development install, use `maturin develop`. Maturin is a build/package tool and is not
a runtime dependency of the extension. The package's build script adds the platform-specific
extension linker arguments required by PyO3; a direct Cargo build is:

```sh
PYO3_BUILD_EXTENSION_MODULE=1 cargo build --release --manifest-path Cargo.toml
```

## Value and ownership

```python
from framework_python import OwnedBytes

payload = OwnedBytes(b"native payload")
assert payload.length == 14
assert len(payload) == 14
assert bytes(payload) == b"native payload"
```

Construction copies the input `bytes` through `framework_owned_buffer_copy`; it does not retain a
Python object or borrow Python memory. Dropping the Python object drops its unique
`FrameworkOwnedBuffer`, which releases the framework allocation exactly once. `bytes(payload)`
creates a normal Python-owned copy. No zero-copy Python buffer or borrowed memoryview is exposed.

`PyMemoryError` represents framework allocation failure, `OverflowError` represents a byte length
outside the framework or Python size range, and unexpected framework statuses become
`RuntimeError`. Python's own allocation failure from `bytes(payload)` also remains a
`MemoryError`. The ABI output starts empty and remains empty on its documented failures.

## Dependency and scope

PyO3 is the single Rust dependency added by this binding. It supplies CPython type/module glue that
cannot be expressed by `core`/`alloc`; a handwritten CPython C API layer would duplicate reference,
exception, and interpreter-version machinery while creating a larger unsafe surface. Only the
`macros` and `abi3-py39` features are enabled; the latter narrows compatibility to the stable Python
3.9+ ABI. PyO3's macros introduce a build-time proc-macro graph, and the interpreter/runtime cost is
paid only when this separate extension is built and imported. No PyO3 or CPython type enters
`framework-abi`, `framework-core`, or the C ABI.

The wrapper's dependency seam is the framework-owned `FrameworkSlice`,
`framework_owned_buffer_copy`, and `FrameworkOwnedBuffer` API. A future binding implementation can
replace PyO3 inside this package without changing those portable contracts or Rust/C callers.
This F36 slice does not add a wheel release pipeline, async Python API, capability-specific Python
surface, buffer protocol, or claim of Python runtime validation.

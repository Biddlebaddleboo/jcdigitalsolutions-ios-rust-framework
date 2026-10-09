# Portable byte and UTF-8 data

`framework-data` is a standalone `no_std` utility for borrowed byte and UTF-8 views, explicit
owned copies, and allocation-transfer conversions. It has no platform, D1 crate, or third-party
dependency. Owned values use Rust `alloc`; the crate itself does not select or initialize an
allocator.

## Borrowed views

`ByteView<'a>` borrows an exact `&'a [u8]`. `Utf8View<'a>` borrows exact valid UTF-8 `&'a str`.
Constructing or inspecting either view keeps the original storage and allocates nothing. A byte
view's `try_as_utf8()` validates with Rust UTF-8 rules and, on success, returns a string view over
the same bytes. Invalid input returns `DataError::InvalidUtf8`; it is not normalized, decoded with
replacement characters, or copied. `Utf8View::as_byte_view()` exposes the same text bytes as a
borrowed byte view.

```rust
use framework_data::{ByteView, DataError, Utf8View};

fn inspect(bytes: &[u8]) -> Result<&str, DataError> {
    let text = ByteView::new(bytes).try_as_utf8()?;
    Ok(text.as_str())
}

fn text_bytes(text: &str) -> &[u8] {
    Utf8View::new(text).as_byte_view().as_bytes()
}
```

The lifetime ties each borrowed view to its input. A view never owns or extends the lifetime of its
source. A backend that needs data after a borrow ends must make an explicit copy or take ownership
through one of the transfer methods below.

The iOS `ios-data` adapter converts between `ByteView` / `OwnedBytes` and immutable Core Foundation
`CFData` with an explicit copy in each direction. It exposes a borrowed `&CFData` for native APIs;
it does not add native UTF-8/`CFString` conversion. See the [iOS data guide](../ios/data.md).

## Explicit copies and allocation transfer

| Operation | Effect |
| --- | --- |
| `ByteView::copy_to_owned()` | Copies bytes into `OwnedBytes`, allocating as needed |
| `Utf8View::copy_to_owned()` | Copies UTF-8 bytes into `OwnedText`, allocating as needed |
| `OwnedBytes::from_vec()` / `into_vec()` | Moves a `Vec<u8>` allocation into or out of the wrapper |
| `OwnedText::from_string()` / `into_string()` | Moves a `String` allocation into or out of the wrapper |
| `OwnedText::into_bytes()` | Moves the string's backing vector into `OwnedBytes` |
| `OwnedBytes::try_into_text()` | Validates and moves the vector allocation into `String` on success |

`OwnedBytes` and `OwnedText` do not implement `Clone`, which avoids an implicit full-data copy.
Their constructors and `into_*` methods move ownership without copying. A successful
`OwnedBytes::try_into_text()` also uses `String::from_utf8` without copying. If the bytes are
invalid, `OwnedUtf8Error` retains the original vector and reports `valid_up_to()` in bytes;
`into_bytes()` recovers the original allocation. Copy methods require allocator-backed storage and
copy the full byte length; an empty copy may not need a heap allocation. The crate does not promise
a particular allocation strategy or recover from allocator failure.

```rust
use framework_data::{ByteView, OwnedBytes};
use std::{string::String, vec::Vec};

fn take_text_bytes(bytes: Vec<u8>) -> Result<String, OwnedBytes> {
    let text = OwnedBytes::from_vec(bytes).try_into_text().map_err(|error| error.into_bytes())?;
    Ok(text.into_string())
}

fn copy_input(bytes: &[u8]) -> Vec<u8> {
    ByteView::new(bytes).copy_to_owned().into_vec()
}
```

## Scope and relation to other contracts

The crate handles byte slices and UTF-8 only. It does not detect encodings, normalize Unicode,
transcode, mutate borrowed storage, wrap foreign/native buffers, or define FFI ownership. It does
not change the specific string and byte conversions already defined by D1 file, preference, and
network contracts; those crates remain independent of `framework-data`.

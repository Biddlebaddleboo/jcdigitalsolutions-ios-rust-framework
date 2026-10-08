# Plain-text clipboard

**framework-sharing** defines a portable **no_std** contract for plain-text clipboard read, write,
and clear. It does not access a device, request permission, create a global service, or present share
UI. A platform guide owns native privacy prompts and UI behavior.

## Rust API

~~~rust
use framework_sharing::{Clipboard, ClipboardBackend};

async fn copy_text<B: ClipboardBackend>(clipboard: &mut Clipboard<B>)
    -> Result<(), framework_sharing::ClipboardError>
{
    clipboard.write("Copied text").await
}
~~~

**Clipboard<B>** owns caller-supplied backend state. The concrete **ClipboardBackend** type is
selected statically; each associated future uses **core::future::Future**. The crate adds no boxed
trait object, executor, **Send** requirement, registry, or hidden initialization. Availability is a
non-prompting backend query.

## Text, ownership, and shared state

The contract covers UTF-8 plain text only. **Clipboard::read** returns an owned **String** when a
readable plain-text value exists, **None** when no such representation exists, and **Err** for a
backend failure. A backend must copy or convert native clipboard text into Rust-owned storage; the
result does not borrow native storage. The facade makes no extra result copy, but a native adapter
may allocate and copy during conversion.

**Clipboard::write** borrows the caller's **str** until its future completes or is dropped. The
facade makes no owned input copy; a backend must copy text if it needs the bytes after the borrow
ends or if its native API requires owned storage. **Clipboard::clear** removes the plain-text value
only. Rich text, images, files, arbitrary pasteboard representations, and share-sheet UI are outside
this API.

The clipboard is shared state, not an exclusive resource. Other apps or native code may change it
at any time. A read does not reserve its result, and a successful write or clear does not guarantee
the state stays unchanged. This contract does not define the platform's privacy indicator, prompt,
paste access policy, or UI behavior; see the platform backend guide.

## Future and cancellation semantics

An operation starts on first poll. Dropping it before first poll starts no backend work. Dropping a
started future suppresses its result and asks the backend to cancel native work where supported. A
native operation may still finish or change clipboard state after Rust drops the future. If native
cancellation is unavailable or races with completion, the backend must detach callback state safely,
discard late results, and release that state exactly once. Dropping is not a rollback guarantee.

Backend errors preserve the framework **ErrorKind** and optional signed native code through
**ClipboardError::Backend**. No permission semantics or native error taxonomy are imposed on the
portable contract.

## Portable validation

~~~sh
cargo fmt --all -- --check
cargo test -p framework-sharing
cargo check -p framework-sharing --no-default-features
git diff --check
~~~

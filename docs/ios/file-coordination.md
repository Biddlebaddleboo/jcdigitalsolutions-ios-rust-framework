# iOS File Coordination

`ios-files::IosFileCoordinator` wraps Foundation's public `NSFileCoordinator` single-item read and
write accessors for a caller-supplied `NSURL`. It is an additive iOS extension; it does not change
the portable `framework-files` contract or `IosFiles` sandbox-root behavior

## API

Create an `IosFileCoordinator`, then call `coordinate_read` or `coordinate_write` with a file URL,
the corresponding `NSFileCoordinatorReadingOptions` or `NSFileCoordinatorWritingOptions`, and an
accessor closure:

```rust,ignore
let mut coordinator = IosFileCoordinator::new();
let value = coordinator.coordinate_read(
    &url,
    NSFileCoordinatorReadingOptions::empty(),
    |coordinated_url| read_with_foundation(coordinated_url),
)?;
```

The installed `objc2-foundation` 0.3.2 generated declarations provide both
`coordinateReadingItemAtURL_options_error_byAccessor` and
`coordinateWritingItemAtURL_options_error_byAccessor`. These methods take a typed options value,
an optional `NSError` output, and a synchronous accessor block. The Foundation binding documents
the same [read accessor](https://developer.apple.com/documentation/foundation/nsfilecoordinator/coordinate%28readingitemat%3Aoptions%3Aerror%3Abyaccessor%3A%29?language=objc)
and [write accessor](https://developer.apple.com/documentation/foundation/nsfilecoordinator/coordinate%28writingitemat%3Aoptions%3Aerror%3Abyaccessor%3A%29?language=objc)
contracts

## URL and accessor lifetime

- The input must be a file URL; a non-file URL returns `FileError::Backend` with
  `ErrorKind::InvalidInput`
- Foundation may pass a coordinated URL different from the input URL. The accessor receives that
  URL by borrow and must use it for its file operation during the accessor call
- The coordinated URL borrow ends when the accessor returns. Retaining a native URL does not
  extend Foundation's coordination window
- If Foundation reports an `NSError`, it takes precedence over the accessor result; its `code` is
  mapped to the portable platform code when representable as `i32`. Otherwise, the coordinate call
  returns the accessor's `R` value or its `FileError` unchanged. The portable error does not
  preserve the NSError domain or user info. If coordination fails before the accessor, Foundation
  may skip the accessor; if no accessor result and no `NSError` are returned, the API reports
  `ErrorKind::Internal`
- An accessor panic is caught at the Objective-C block boundary and maps to `ErrorKind::Internal`.
  The panic hook still runs; `panic=abort` remains fatal

The write caller selects Foundation's standard write options. The API does not infer whether the
accessor replaces, moves, or deletes the item and does not itself perform file I/O

## Thread, blocking, and reentrancy

`IosFileCoordinator` is `!Send`/`!Sync`; create, use, and drop one instance on the same thread.
There is no UIKit main-thread requirement. Each method is synchronous and blocks its caller until
Foundation returns from coordination and the accessor finishes. No executor, queue, timeout, or
cancellation layer is provided

Methods take `&mut self` to serialize one coordinator instance. Do not start another coordinated
operation for the same URL from the accessor, or block there on work that needs that file. Do not
hold locks that a file presenter or another coordinated operation needs. Separate coordinator
instances and other processes are outside this local serialization; Foundation may wait for their
file presenters and operations

## Limits

The caller supplies a file URL and is responsible for its validity, access rights, and any required
security-scoped resource lifetime. For an already-scoped URL, the separate
[`IosSecurityScopedAccess` guard](security-scoped-access.md) balances the scope start/stop pair; it
does not change this coordinator's behavior. `IosFileCoordinator` does not present a document
picker, resolve a bookmark, or acquire a security scope. It does not register an `NSFilePresenter`
or claim FileProvider/document-provider behavior. The coordinator does not validate that the URL is
within one of `IosFiles`' app-sandbox roots, and this extension does not provide root containment,
multi-item access, move/delete coordination, async access, durability, or conflict resolution

Locked device/simulator `cargo check` and strict all-target Clippy pass for `ios-files` on Xcode
26.6 / SDK 26.5, below the Xcode 27.x baseline. These target checks establish compile/lint only;
they do not prove runtime coordination, provider access, document-picker behavior, or security-scope
handling

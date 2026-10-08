# iOS URLSession download-file adoption

`ios-files` exposes one synchronous operation for a URLSession temporary download file:

```rust
pub fn adopt_url_session_download(
    &mut self,
    temporary_file_url: &NSURL,
    destination: AppPath<'_>,
) -> Result<WriteOutcome, FileError>
```

Call it from `URLSession:downloadTask:didFinishDownloadingToURL:` before that delegate callback
returns. URLSession owns the temporary source location; the operation borrows the URL and reads the
file, but does not retain the URL, move the source, or remove it. The source must be a file URL to a
regular file. A non-file URL, final source symlink, or non-regular source is rejected. The method
cannot prove URLSession provenance or containment for arbitrary source URLs; pass only the callback
URL.

## Destination boundary

The destination is an existing `framework_files::AppPath`, not an arbitrary URL or native path.
`ios-files` applies its existing path validation, opens each parent component relative to the
selected sandbox-root descriptor with `O_DIRECTORY | O_NOFOLLOW`, and rejects intermediate
symlinks. The final entry is never followed. If it is a symlink, atomic replacement replaces the
symlink itself and leaves its target untouched. A directory at the final path causes the rename to
fail. The operation creates no parent directories.

## Copy and atomic commit

The backend creates a private mode-`0600` staging file in the already-open destination parent and
copies the source with `std::io::copy`. This avoids a payload-sized `Vec`, but it is not zero-copy:
the complete file is copied once. The standard library may use bounded buffering or an optimized
OS file-copy path, and the operation can block for the duration of that copy.

The staging file is closed before `renameat` installs it at the final path. Successful `renameat`
is the atomic commit point and the method then returns `WriteAtomicity::Atomic`. Before that point,
the final path remains absent or names the old entry; after it, it names the complete new file.
There is no direct-write or non-atomic fallback. The same-directory stage keeps the commit on the
same filesystem.

Atomic visibility is not crash durability. The backend does not call `fsync` on the staged file or
parent directory. A crash before commit can leave a hidden `.ios-files-*` stage; there is no
startup scavenger. After copy or rename failure, the method closes and unlinks the stage
best-effort, preserves the primary operation error, and does not commit the destination. Cleanup
failure can leave a hidden stage. The URLSession source remains owned by URLSession/caller cleanup.

This operation adds no permission prompt, entitlement, document-provider access, security-scoped
URL handling, persistent transfer state, or crash-recovery contract. It does not change the
existing `ios-files` iOS 10.0 API floor. The inspected Xcode 26.6 / iOS 26.5 SDK header marks
`NSURL.fileSystemRepresentation` available from iOS 7.0 and documents its autoreleased inner
pointer; the implementation opens the path within an explicit autorelease pool. The host remains
below the repository's Xcode 27.x build baseline. See [the B14 plan](../../PLAN_IOS_FILE_ADOPTION.md)
and [the sandbox-files guide](files.md) for the broader backend contract and symlink policy.

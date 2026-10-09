# PLAN_IOS_FILE_ADOPTION.md — Workstream B14: URLSession Temporary-File Adoption

## Status

B14 is implemented locally in `ios-files` and documented. The final-entry alias guard and checked
staging-file close were added after independent review. Locked iOS device/simulator checks, strict
Clippy, crate rustdoc, formatting, and diff checks pass; no tests were added or run. These checks do
not establish live URLSession delivery or runtime filesystem behavior.

The app-data link/import example retains a reference to `IosFiles::adopt_url_session_download` and
to a helper that starts then drops `IosSecurityScopedAccess`; the gate checks the adoption symbol
and both `startAccessingSecurityScopedResource` and `stopAccessingSecurityScopedResource` selector
strings. The updated gate passed for device and Simulator. This is selected-symbol/link evidence
only and does not establish URLSession provenance, security-scope availability, or file-operation
behavior.

These target checks ran with Xcode 26.6 and iPhoneOS/iPhoneSimulator SDK 26.5, below the required
Xcode 27.x baseline.

## Objective

Add one synchronous, iOS-only `ios-files` operation that copies a URLSession download callback's temporary file URL into an existing sandbox `AppPath`. The operation must reuse B1's destination path and symlink boundary, avoid a whole-body `Vec`, stage beside the destination, and expose the complete new destination only at a documented atomic rename commit point.

This is an additive sandbox-file operation within canonical capability row 010. It is not row 014's
caller-supplied file-coordination extension; it adds no provider, picker, or security-scope lifecycle
support.

B14 is a prerequisite for the B13 background-transfer backend. It does not approve or implement B13, D10, or a general transfer API.

## Dependencies

- B1 `ios-files` and D1 `framework-files::AppPath` are integrated.
- Foundation A and the existing `objc2-foundation` `NSURL` binding are integrated.
- No Swift ABI, C ABI, additional crate, or root workspace change is required.

## Write scope

- `platform/ios/ios-files/**`
- `docs/ios/file-adoption.md`
- `PLAN_IOS_FILE_ADOPTION.md`
- the B14 link in `PLAN_IOS_NATIVE.md`

Do not edit B13, D10, other backends, shared dependencies, the root manifest/lockfile, capability manifests, or tests. B1 owns `platform/ios/ios-files/**`.

## Required API and behavior

Expose an iOS-only method on `IosFiles` with this semantic signature:

```rust
pub fn adopt_url_session_download(
    &mut self,
    temporary_file_url: &NSURL,
    destination: AppPath<'_>,
) -> Result<WriteOutcome, FileError>
```

- The URL must be the file URL naming a regular URLSession temporary download file. The caller must invoke this synchronous method before the URLSession download delegate callback returns. The method borrows and reads the URL; it does not retain the URL or remove the URLSession source file. It cannot prove URLSession provenance or containment for arbitrary source URLs; callers must pass only the callback URL.
- Resolve the destination only through existing `IosFiles::root`, `path_parts`, and `open_parent` rules. Do not resolve destination strings by prefix checks, accept arbitrary destination URLs, follow intermediate symlinks, or follow a final symlink.
- Reject an existing destination entry that names the same device/inode as the opened source, using `fstatat` with `AT_SYMLINK_NOFOLLOW` so a destination symlink is still replaced as an entry. Recheck immediately before commit; this does not serialize concurrent mutation of the final destination. As with all B1 descriptor-relative operations, an opened parent directory can be renamed outside the selected root by another native handle and remain usable through its descriptor, so adoption does not guarantee confinement under that concurrent namespace mutation.
- Open the source read-only, reject a non-file URL, final symlink, or non-regular file, and preserve the relevant POSIX error category/code. Use `NSURL.fileSystemRepresentation` only inside an autorelease pool because Foundation returns an autoreleased inner pointer.
- Read through the opened source descriptor without a file-coordination lock or snapshot. Concurrent source mutation through another handle can affect the copied bytes; the caller must not mutate the URLSession callback file during adoption.
- Create a private `0600` staging file in the already-open destination parent using the existing exclusive/no-follow temporary-file helper. Copy with `std::io::copy`; do not allocate or materialize a payload-sized `Vec`. Any library/OS buffering must remain independent of payload size.
- Finish and explicitly close the staged copy, reporting a close error before committing. Use the existing same-directory `renameat` operation as the only commit point. It creates a missing destination or atomically replaces an existing final entry; a final symlink is replaced as an entry and is never followed. There is no non-atomic fallback.
- Return `WriteAtomicity::Atomic` only after `renameat` succeeds. Before that success, a reader sees the old destination or no destination, never staged partial bytes. After it succeeds, the final path names the complete copied file.
- On source, copy, close, alias-check, or rename failure, leave the final destination uncommitted and remove the staging entry best-effort. Preserve the primary operation error if cleanup also fails. The URLSession source remains owned by the caller/URLSession.

## Costs and limits

- The operation is synchronous and can block in the URLSession delegate callback.
- The source is not coordinated or snapshotted. This is suitable only while the callback-owned temporary file remains stable for the duration of the copy.
- It performs one full disk-to-disk payload copy; it is not a zero-copy move. `std::io::copy` avoids a payload-sized allocation and may use bounded buffering or an optimized OS copy path. The temporary staging file and final file share one destination directory/filesystem.
- New destination entries use the existing `0600` mode. Atomic visibility is not crash durability; do not add an `fsync` or durability claim in this slice. A process crash before commit can leave a hidden `.ios-files-*` staging entry; no startup scavenger is added.
- The existing `ios-files` API floor and sandbox roots remain unchanged. No permission, entitlement, document-provider, security-scoped URL, arbitrary file URL, persistence, or crash-recovery behavior is added.

## Validation and handoff

- Do not add or run tests for this delegated slice.
- Run the non-test iOS device/simulator `cargo check` and strict Clippy gates for `ios-files`, formatting, crate docs, and diff checks where the installed toolchain permits.
- Record exact commands/results and host/Xcode limitations. Report API, atomic commit behavior, source ownership, copy cost, cleanup limits, and unresolved assumptions.

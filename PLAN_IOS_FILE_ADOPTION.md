# PLAN_IOS_FILE_ADOPTION.md — Workstream B14: URLSession Temporary-File Adoption

## Objective

Add one synchronous, iOS-only `ios-files` operation that copies a URLSession download callback's temporary file URL into an existing sandbox `AppPath`. The operation must reuse B1's destination path and symlink boundary, avoid a whole-body `Vec`, stage beside the destination, and expose the complete new destination only at a documented atomic rename commit point.

B14 is a prerequisite for the proposed B13 background-transfer backend. It does not approve or implement B13, D10, or a general transfer API.

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
- Open the source read-only, reject a non-file URL, final symlink, or non-regular file, and preserve the relevant POSIX error category/code. Use `NSURL.fileSystemRepresentation` only inside an autorelease pool because Foundation returns an autoreleased inner pointer.
- Create a private `0600` staging file in the already-open destination parent using the existing exclusive/no-follow temporary-file helper. Copy with `std::io::copy`; do not allocate or materialize a payload-sized `Vec`. Any library/OS buffering must remain independent of payload size.
- Finish and close the staged copy before committing. Use the existing same-directory `renameat` operation as the only commit point. It creates a missing destination or atomically replaces an existing final entry; a final symlink is replaced as an entry and is never followed. There is no non-atomic fallback.
- Return `WriteAtomicity::Atomic` only after `renameat` succeeds. Before that success, a reader sees the old destination or no destination, never staged partial bytes. After it succeeds, the final path names the complete copied file.
- On source, copy, or rename failure, leave the final destination uncommitted and remove the staging entry best-effort. Preserve the primary operation error if cleanup also fails. The URLSession source remains owned by the caller/URLSession.

## Costs and limits

- The operation is synchronous and can block in the URLSession delegate callback.
- It performs one full disk-to-disk payload copy; it is not a zero-copy move. `std::io::copy` avoids a payload-sized allocation and may use bounded buffering or an optimized OS copy path. The temporary staging file and final file share one destination directory/filesystem.
- New destination entries use the existing `0600` mode. Atomic visibility is not crash durability; do not add an `fsync` or durability claim in this slice. A process crash before commit can leave a hidden `.ios-files-*` staging entry; no startup scavenger is added.
- The existing `ios-files` API floor and sandbox roots remain unchanged. No permission, entitlement, document-provider, security-scoped URL, arbitrary file URL, persistence, or crash-recovery behavior is added.

## Validation and handoff

- Do not add or run tests for this delegated slice.
- Run the non-test iOS device/simulator `cargo check` and strict Clippy gates for `ios-files`, formatting, crate docs, and diff checks where the installed toolchain permits.
- Record exact commands/results and host/Xcode limitations. Report API, atomic commit behavior, source ownership, copy cost, cleanup limits, and unresolved assumptions.

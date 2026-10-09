# B109: iOS Regular-File Allocated Block Count

## Status

`IosFiles::regular_file_allocated_blocks_512` returns the raw `st_blocks` count for a regular file
in 512-byte units. It complements logical file size and makes no exact physical-storage claim

## Objective

Expose a read-only iOS query for one sandbox regular file's filesystem-reported block allocation
without reading its contents

## Contract

- `regular_file_allocated_blocks_512(&self, path: AppPath<'_>) -> Result<u64, FileError>` uses the
  existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. It must be a regular
  file; a final symlink, directory, or special entry returns `InvalidInput`.
- The `u64` is the `st_blocks` count in 512-byte units, widened from the signed Apple `blkcnt_t`.
  A negative field maps to `InvalidInput`; zero is a valid reported count.
- This field differs from logical byte length. Sparse files may use fewer allocated blocks than
  their logical size implies. It is the filesystem-reported value, not exact physical-device
  usage, a measure of exclusive storage, or a stable cross-filesystem comparison.
- Missing final entries map through the existing POSIX error mapper to `NotFound`; parent,
  permission, and metadata errors retain the existing mapped category and native code.
- The query is synchronous, reads no contents, retains no descriptor, follows no final symlink,
  accepts no absolute URL, starts no security scope, and grants no access. It has the existing
  concurrent native directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It does not add allocated-size semantics to the portable
  `FileBackend`/`Files` contract.

## Static evidence

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_blocks` as the actual allocated block count in 512-byte units; it notes that a short
  symlink may report zero, which is one reason this API accepts only regular files.
- The installed iPhoneOS 26.5 SDK declares `st_blocks` as `blkcnt_t`; the locked `libc` Apple
  binding defines `blkcnt_t` as `i64` and exposes `stat.st_blocks` with that type.
- `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0. B109
  adds no deployment-floor requirement, dependency, framework, or native symbol.
- The adapter already uses one `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup for regular-file size.
  B109 adds no permission prompt, Info.plist key, entitlement, provider behavior, bookmark,
  coordination, or persistence behavior.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check` and scoped `git diff --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run.

# B99: iOS Entry Identity Snapshot

## Status

`IosFiles::entry_identity_snapshot` returns the final entry's `(st_dev, st_ino)` pair from one
no-follow metadata lookup. It is a short-lived comparison value, not a persistent ID or retained
handle

## Objective

Expose one bounded POSIX entry-identity snapshot for app code that needs a best-effort comparison
of sandbox path entries, without reading file contents or opening the entry

## Contract

- `entry_identity_snapshot(&self, path: AppPath<'_>) -> Result<IosFileIdentitySnapshot, FileError>`
  uses the existing semantic `AppDirectory`, path validation, root descriptor, and
  descriptor-relative parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. The result carries
  `st_dev` as a nonnegative `u64` and `st_ino` as `u64`; a symlink reports the symlink's own pair.
- The query accepts regular files, directories, symlinks, and special entries. Missing final
  entries map through the existing POSIX mapper to `NotFound`; a negative device value maps to
  `InvalidInput`; path and metadata failures keep existing mapped categories and native codes.
- Equal pairs can indicate the same live inode when observed at about the same time, including
  hard-linked names. The value is not globally unique, persistent, or resistant to inode reuse
  after unlink. It does not pin the object. Separate queries and later operations are not atomic
  with concurrent path mutation.
- The query is synchronous, reads no contents, retains no descriptor, follows no final symlink,
  accepts no absolute URL, starts no security scope, and grants no access. It has the existing
  concurrent native directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It does not add a stable file-identity promise to the portable
  `FileBackend`/`Files` contract or change `AppPath` semantics.

## Static evidence

- The installed iOS SDK `sys/stat.h` identifies `st_dev` as the device containing the inode and
  `st_ino` as the file serial number. The checked-in `libc` Apple binding types these as `dev_t`
  (`i32`) and `ino_t` (`u64`); the adapter converts the device value to a nonnegative `u64`.
- Darwin `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0.
  This method adds no newer API, framework, dependency, or native symbol.
- The adapter already uses `(st_dev, st_ino)` internally to reject source/destination aliasing.
  B99 makes the raw pair available for caller comparison while documenting its reuse and race
  limits.
- No permission prompt, Info.plist key, entitlement, provider API, bookmark, coordination, or
  persistence behavior is involved.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and
  `git diff --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run. These
  checks used Rust/Cargo 1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5;
  local Xcode is below the repo's required Xcode 27.x baseline.

# B112: iOS Entry Access-Time Snapshot

## Status

`IosFiles::entry_access_time` returns the filesystem-reported POSIX access-time fields from one
no-follow lookup. It rejects symlinks and makes no access-log or reliable-change-token claim

## Objective

Expose a read-only iOS snapshot of `st_atime` for a validated sandbox entry, with explicit limits
on what an access timestamp proves

## Contract

- `entry_access_time(&self, path: AppPath<'_>) -> Result<IosFileAccessTime, FileError>` uses the
  existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. Final symlinks return
  `InvalidInput`; regular files, directories, and special entries report the final inode's
  `st_atime` and `st_atime_nsec` fields.
- The result type holds signed `i64` seconds relative to the Unix epoch and `u32` nanoseconds within
  that POSIX second. A nanosecond field outside `0..1_000_000_000` maps to `InvalidInput`.
- Apple defines `st_atime` as the time file data was last accessed and lists `mknod`, `utimes`, and
  `read` as operations that change it. The value is explicitly settable and the API cannot prove
  that every data access refreshed the field. It is a filesystem-reported diagnostic, not an
  access log, content version, reliable change token, or durability proof.
- Filesystems may report coarser precision. This is one point-in-time observation; data access or
  metadata changes can race it. The query reads no target contents and does not reserve the path.
- Missing final entries map through the existing POSIX error mapper to `NotFound`; parent,
  permission, and metadata errors preserve the existing mapped category and native code.
- The query is synchronous, follows no final symlink, accepts no absolute URL, starts no security
  scope, and grants no access. It has the existing concurrent native directory-rename containment
  limit.
- This is an iOS-only `IosFiles` API. It adds no timestamp concept to the portable
  `FileBackend`/`Files` contract and adds no dependency, framework, or native symbol.

## Static evidence

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_atimespec` as the time of last access and states that `st_atime` is changed by
  `mknod`, `utimes`, and `read`. It also says symbolic links do not have their own timestamps, so
  B112 rejects them rather than exposing containing-directory time as link time.
- The installed iPhoneOS 26.5 SDK declares `st_atimespec` as the time of last access and exposes
  the `st_atime`/`st_atimensec` aliases. The locked `libc` Apple binding exposes
  `stat.st_atime` and `stat.st_atime_nsec`.
- `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0. B112
  adds no deployment-floor requirement, dependency, framework, or native symbol.
- No permission prompt, Info.plist key, entitlement, provider behavior, bookmark, coordination, or
  persistence behavior is involved.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check` and scoped `git diff --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run.

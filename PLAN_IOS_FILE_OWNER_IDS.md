# B118: iOS POSIX Owner and Group IDs

## Status

`IosFiles::entry_owner_ids` returns the raw fixed-width owner and group IDs from one no-follow
metadata lookup. The values are numeric metadata, not user identities or access decisions

## Objective

Expose a read-only iOS snapshot of one app-sandbox entry's POSIX owner and group IDs for
filesystem diagnostics, without account lookup or access-policy inference

## Contract

- `entry_owner_ids(&self, path: AppPath<'_>) -> Result<IosFileOwnerIds, FileError>` uses the
  existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. Final symlinks return
  `InvalidInput`; regular files, directories, and special entries return the same entry's `st_uid`
  and `st_gid` values.
- `IosFileOwnerIds` returns the Apple `uid_t`/`gid_t` values as fixed-width `u32` numeric fields
  through `user_id()` and `group_id()`. Values are not account names, persistent identities across
  install or device changes, group membership, or a check of effective access.
- Missing final entries map through the existing POSIX error mapper to `NotFound`; parent,
  permission, and metadata errors preserve the existing mapped category and native code.
- The query is synchronous, reads no contents, retains no descriptor, follows no final symlink,
  accepts no absolute URL, starts no security scope, and grants no access. It has the existing
  concurrent native directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It adds no ownership or identity concept to the portable
  `FileBackend`/`Files` contract and adds no dependency, framework, or native symbol.

## Static evidence

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_uid` as the file owner's user ID and `st_gid` as its group ID. The same document
  says symbolic links have no own owner or group fields, so B118 rejects final symlinks.
- The installed iPhoneOS 26.5 SDK declares `st_uid` and `st_gid` as `uid_t` and `gid_t`. The locked
  `libc` Apple binding defines both as `u32` for this target and exposes them on `stat`.
- `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0. B118
  adds no deployment-floor requirement, permission prompt, Info.plist key, entitlement, dependency,
  framework, or native symbol.
- This reports only kernel metadata for a path already contained by `ios-files`; there is no
  account database lookup, identity disclosure API, or access-policy evaluation.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check` and scoped `git diff --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run.

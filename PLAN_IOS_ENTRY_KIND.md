# B93: iOS Single-Entry Kind Query

## Status

`IosFiles::entry_kind` classifies one sandbox `AppPath` as `File`, `Directory`, or `Other` using the
same no-follow metadata helper as `read_directory`. It reads no contents and adds no portable
`FileBackend` operation, URL access, or security-scope behavior

## Objective

Add a prompt-free iOS-only query for the type of one known sandbox entry without reading its
contents or scanning its parent directory

## Contract

- `entry_kind(&self, path: AppPath<'_>) -> Result<FileKind, FileError>` uses the existing semantic
  `AppDirectory`, path validation, root descriptor, and descriptor-relative parent traversal.
- Classification is identical to `read_directory`: regular files return `File`, directories return
  `Directory`, and symlinks or other non-file/non-directory entries return `Other`.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. A missing final entry
  maps to `NotFound`; parent, permission, and metadata errors use the existing POSIX error mapper.
- The returned `FileKind` does not pin the entry for a later operation. Another handle may replace
  the final entry after this query; concurrent parent-directory rename has the existing documented
  containment limit.
- The query is synchronous and may block on filesystem metadata. It reads no bytes, allocates no
  payload-sized buffer, follows no final symlink, accepts no absolute URL, starts no security scope,
  and grants no access.
- This is an iOS-only inherent method on `IosFiles`; it does not alter the portable
  `FileBackend`/`Files` contract or the meaning of `AppPath` and `FileKind`.

## Static evidence

- `entry_kind` is already the adapter's `fstatat(..., AT_SYMLINK_NOFOLLOW)` classifier for
  `read_directory` and replacement safety checks. B93 reuses it rather than adding a new POSIX or
  Foundation path policy.
- The helper maps `S_IFREG` to `File`, `S_IFDIR` to `Directory`, and every other entry type,
  including symlinks, to `Other`, matching the portable `FileKind` documentation.
- The existing package iOS floor, framework imports, dependencies, and link symbols do not change.
- No prompt, Info.plist key, entitlement, provider API, bookmark, coordination, or persistence
  behavior is involved.

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

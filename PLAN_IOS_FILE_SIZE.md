# B90: iOS Regular File Size Snapshot

## Status

`IosFiles::regular_file_size` returns the size of one regular file under a validated `AppPath` via
one `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup. It reads no file contents, follows no final
symlink, and does not use a URL or security-scope API. Locked iOS device and arm64 Simulator
checks, strict Clippy, rustdoc, format, docs-check, and diff-check pass. No tests, live app-sandbox
operation, or consumer/probe binary was run

## Objective

Add one read-only, iOS-only file-size query to `IosFiles` for callers that need the length before
allocating or reading file contents. Keep it separate from the portable `framework-files` facade;
do not change access, file coordination, or sandbox-root semantics

## Contract

- `regular_file_size(&self, path: AppPath<'_>) -> Result<u64, FileError>` uses the existing semantic
  `AppDirectory`, `path_parts`, root descriptor, and `open_parent` path rules.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. A regular file
  returns its nonnegative `st_size` as `u64`; a final symlink, directory, or special entry returns
  `InvalidInput`.
- A missing final entry maps through the existing POSIX error mapper to `NotFound`. Parent path,
  permission, and other errors preserve the mapped category and native code.
- The result is a point-in-time metadata observation, not a lock or content snapshot. Another
  handle may change or replace the file before a later read; a concurrent native directory rename
  retains the same containment limit already documented for `IosFiles`.
- The method is synchronous and may block on filesystem metadata lookup. It reads no file bytes,
  allocates no payload-sized buffer, follows no final symlink, accepts no absolute URL, starts no
  security scope, and grants no access.
- This is an iOS-only utility on `IosFiles`; it does not add a method to the portable
  `FileBackend`/`Files` contract or change `AppDirectory` semantics.

## Static evidence

- The `ios-files` adapter already uses `fstatat` with `AT_SYMLINK_NOFOLLOW` in `exists` and
  directory-entry classification. B90 reuses the same descriptor-relative root and parent
  traversal and adds no framework, Objective-C API, dependency, or native link symbol.
- Darwin `struct stat.st_size` is converted with `u64::try_from`; a negative value maps to
  `InvalidInput`. The existing `file_error` mapper handles `ENOENT` as `NotFound` and preserves
  the relevant POSIX code for other failures.
- `IosFiles::new` and the existing descriptor-relative calls establish the iOS 10.0 package floor;
  this method adds no newer API.
- No permission prompt, Info.plist key, entitlement, file-provider API, or security-scope behavior
  is involved.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 clippy --locked --offline -p ios-files --target aarch64-apple-ios -- -D warnings`
  and `cargo +1.94.1 clippy --locked --offline -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`,
  `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and
  `git diff --check`.
- Checks used Xcode 26.6 build `17F113`, iPhoneOS/iPhoneSimulator SDK 26.5, and Rust/Cargo 1.94.1;
  the local Xcode is below the repository's required Xcode 27.x baseline.

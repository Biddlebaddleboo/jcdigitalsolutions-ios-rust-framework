# B134: iOS Directory Entry-Kind Counts

## Disposition

Add `IosFiles::directory_entry_kind_counts` for a caller that needs direct file/directory/other totals but not names. It preserves `read_directory`'s no-follow `FileKind` classification without a Rust-owned name list. It does not change the portable `FileBackend` contract

## API

- `directory_entry_kind_counts(&self, path: AppPath<'_>) -> Result<IosDirectoryEntryKindCounts, FileError>` uses the existing semantic app directory, validated relative path, retained root descriptor, and no-follow component traversal.
- `IosDirectoryEntryKindCounts` holds private `u64` counts and exposes read-only `files()`, `directories()`, and `other()` accessors. `Other` includes symbolic links and all entry types that are not regular files or directories.
- Open the directory with `open_directory`; duplicate its descriptor for `fdopendir`; use `readdir` for direct names; skip only `.` and `..`; classify each name with the existing `entry_kind` helper, which uses `fstatat(..., AT_SYMLINK_NOFOLLOW)`.
- Do not decode names as UTF-8, copy them to `String`s, or build a `Vec<DirectoryEntry>`. Names with arbitrary non-NUL bytes are eligible for classification.
- A name may vanish or change between `readdir` and `fstatat`; report the mapped error rather than return partial counts. This is not an atomic snapshot or deletion guard. `remove_directory` remains authoritative.
- The method is synchronous and performs one no-follow metadata lookup per observed child. libc may allocate or prefetch memory for its directory stream. The query reads no file contents, follows no final symlink, accepts no arbitrary URL, starts no security scope, and keeps the existing concurrent directory-rename containment limit.

## Utility and limits

`read_directory` already classifies each entry, but it also validates UTF-8, allocates each owned name, and retains a vector. A caller that needs a category summary can instead receive three fixed-width totals. This avoids name-list allocation but does not avoid the per-entry `fstatat` calls required for matching `FileKind` semantics.

Each count is a point-in-time best-effort result. Namespace changes can cause a lookup error or affect which entries are observed. Do not treat any count as a reservation, completeness guarantee, or permission to remove or replace a path.

## Platform evidence

- Apple's iOS [`opendir(3)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/opendir.3.html) documents `readdir` directory streams, `closedir` cleanup, and possible memory allocation for a stream.
- Apple's iOS [`stat(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html) describes no-follow metadata calls; the existing adapter uses `fstatat` with `AT_SYMLINK_NOFOLLOW` for `FileKind`.
- The installed iPhoneOS 26.5 SDK declares `fdopendir` from iOS 8.0. `fstatat` is also available from iOS 8.0. Existing `ios-files` has an iOS 10.0 package floor.
- Locked `libc` 0.2.189 exposes the Apple `fdopendir`, `readdir`, `closedir`, `dirent::d_name`, `fstatat`, and `AT_SYMLINK_NOFOLLOW` bindings. These operations are already used by the crate.
- No new framework, permission, usage-description key, entitlement, portable operation, dependency, deployment-floor requirement, or native import is added.

## Validation

- Do not add or run tests, execute a consumer, run a linked probe, or call the API against a live app filesystem.
- Run device and arm64 Simulator `cargo +1.94.1 check --locked --offline -p ios-files`.
- Run strict Clippy for the same targets with `-- -D warnings`.
- Run iOS-target rustdoc with `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Run `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.

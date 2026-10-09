# B128: iOS Direct Directory-Entry Count

## Disposition

Add `IosFiles::directory_entry_count` as an iOS-only count of direct directory names. It serves summaries or item-count UI without building the full `DirectoryEntry` list or looking up each entry's kind. It does not change the portable `FileBackend` contract

## API

- `directory_entry_count(&self, path: AppPath<'_>) -> Result<u64, FileError>` uses the existing semantic app directory, validated relative path, retained root descriptor, and no-follow component traversal.
- Open the final directory with the existing `open_directory` helper. Duplicate its descriptor for `fdopendir`; close the directory stream exactly once with `closedir`.
- Count every direct name except `.` and `..`. Do not inspect entry kind, open child entries, decode names as UTF-8, or allocate a Rust `Vec<DirectoryEntry>` or per-name `String`.
- Return `ResourceExhausted` if the `u64` count overflows. Map path, open, stream, and close errors through the existing POSIX mapping.
- The method is synchronous and O(n) in entry count. libc may allocate memory for its directory stream; this API only avoids a Rust-owned name list and per-entry metadata lookups.
- Concurrent directory mutation can affect the observed names. The result is not an atomic snapshot, stable token, reservation, deletion guard, or proof that a later directory operation will succeed.
- The query reads no file contents, follows no final symlink, accepts no arbitrary URL, starts no security scope, and retains the existing concurrent directory-rename containment limit.

## Utility and limits

`read_directory` returns a `Vec<DirectoryEntry>`, copies each UTF-8 name to a `String`, and calls the no-follow kind lookup for each entry. A caller that needs only a count can avoid those result allocations, name conversions, and per-entry `fstatat` calls. The count includes symlinks, special entries, and names whose bytes are not valid UTF-8. It counts direct names only; it is not recursive and does not indicate whether the entries are files or usable content.

The count is best-effort under concurrent mutation. Do not use it as proof of emptiness for a later delete. `remove_directory` remains authoritative for that operation.

## Platform evidence

- Apple's iOS [`opendir(3)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/opendir.3.html) documents directory-stream reads with `readdir`, stream cleanup with `closedir`, and possible memory allocation by `opendir`. The implementation checks thread-local errno when `readdir` returns null and checks `closedir` before it returns a count.
- The installed iPhoneOS 26.5 SDK `dirent.h` declares `fdopendir(int)` with availability from iOS 8.0. The existing `ios-files` package floor is iOS 10.0, so this adds no floor requirement.
- Locked `libc` 0.2.189 exposes `fdopendir`, `readdir`, `closedir`, and `dirent::d_name` for Apple targets. The crate already uses those symbols for `read_directory`.
- No new dependency, framework, permission, usage-description key, entitlement, portable symbol, or native import is introduced.

## Validation

- Do not add or run tests, execute a consumer, run a linked probe, or call the API against a live app filesystem.
- Run device and arm64 Simulator `cargo +1.94.1 check --locked --offline -p ios-files`.
- Run strict Clippy for the same targets with `-- -D warnings`.
- Run iOS-target rustdoc with `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Run `cargo fmt --package ios-files -- --check` and scoped `git diff --check`.

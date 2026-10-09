# B131: iOS Directory Nonempty Check

## Disposition

Add `IosFiles::directory_is_empty` as a narrow point-in-time query that stops after the first direct name other than `.` and `..`. It serves callers that need an empty/nonempty answer but not a full listing or count; for a nonempty directory it need not scan all names. It does not change the portable `FileBackend` contract

## API

- `directory_is_empty(&self, path: AppPath<'_>) -> Result<bool, FileError>` uses the existing semantic app directory, validated relative path, retained root descriptor, and no-follow component traversal.
- The implementation opens the final directory with `open_directory`, duplicates its descriptor for `fdopendir`, and uses the shared directory scanner in early-stop mode. `true` means the stream reached end before any direct name other than `.` or `..`; `false` means it saw at least one such name.
- Stop after the first child name. Do not inspect entry kind, open child entries, decode names as UTF-8, or build a Rust `Vec<DirectoryEntry>` or per-name `String`.
- Close the directory stream exactly once. Map path, open, stream, and close errors through the existing POSIX mapping.
- The call is synchronous. It may avoid the full scan when the directory is nonempty, though libc may allocate or prefetch data for its directory stream.
- Concurrent namespace mutation can affect which names the stream sees. The result is not an atomic snapshot, stable token, reservation, deletion guard, or proof that a later operation will succeed. `remove_directory` remains authoritative.
- The query reads no file contents, follows no final symlink, accepts no arbitrary URL, starts no security scope, and retains the existing concurrent directory-rename containment limit.

## Utility and limits

`directory_entry_count` gives a count but must scan all returned names. A caller that needs only an empty/nonempty decision can stop after its first observed child. This can avoid the rest of the Rust-side enumeration loop for common nonempty directories; it does not promise a specific number of underlying filesystem reads because libc controls directory-stream buffering.

Names of all byte forms and entry kinds count as children. This method does not claim that a directory is empty at any later time. Callers must use the actual removal result rather than this query to determine whether removal succeeds.

## Platform evidence

- Apple's iOS [`opendir(3)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/opendir.3.html) documents directory streams read with `readdir`, cleanup with `closedir`, and possible stream memory allocation by `opendir`.
- The installed iPhoneOS 26.5 SDK `dirent.h` declares `fdopendir(int)` with availability from iOS 8.0. The existing `ios-files` package floor is iOS 10.0.
- Locked `libc` 0.2.189 exposes `fdopendir`, `readdir`, `closedir`, and `dirent::d_name` for Apple targets. The crate already uses these symbols in `read_directory`; B131 adds no native symbol or dependency.
- No new framework, permission, usage-description key, entitlement, portable operation, or deployment-floor requirement is introduced.

## Validation

- Do not add or run tests, execute a consumer, run a linked probe, or call the API against a live app filesystem.
- Run device and arm64 Simulator `cargo +1.94.1 check --locked --offline -p ios-files`.
- Run strict Clippy for the same targets with `-- -D warnings`.
- Run iOS-target rustdoc with `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Run `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.

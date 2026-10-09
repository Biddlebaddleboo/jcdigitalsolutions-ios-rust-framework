# B272: iOS File Added-to-Directory Time Snapshot

## Scope

B272 adds `IosFiles::entry_added_time(&self, path: AppPath<'_>) -> Result<IosFileAddedTime, FileError>` for one regular sandbox file

The method queries `ATTR_CMN_ADDEDTIME` through `fgetattrlist` on an opened regular-file descriptor. It does not add a timestamp concept to the portable `FileBackend`

## API and evidence

`IosFileAddedTime::seconds_since_unix_epoch()` returns signed `i64` seconds; `nanoseconds()` returns `u32` nanoseconds within `0..1_000_000_000`

The installed iPhoneOS 26.5 SDK defines public `ATTR_CMN_ADDEDTIME` as `0x10000000` in `sys/attr.h`. Locked `libc` 0.2.190 binds this constant and `fgetattrlist`. The SDK declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability and gives no separate attribute availability annotation, so B272 adds no higher API floor

Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as a `timespec` with the time a filesystem object was created or renamed into its containing directory. The same manual warns that behavior may be inconsistent for hard-linked items. This differs from B101's `st_birthtime` creation-time no-go and from B96 modification, B107 status-change, and B112 access time

`fgetattrlist` uses a fixed buffer sized for its leading `u32` length and one `libc::timespec`. The parser requires the exact expected returned length and a nanosecond field in `0..1_000_000_000`; an omitted attribute maps to `Unsupported`

## Contract limits

- The method validates `AppPath`, walks parents with the existing no-follow descriptor helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, and accepts regular files only. A final symlink or other entry kind returns `InvalidInput`
- The result is the filesystem-reported timestamp for the opened file at query time. It may mean creation or rename into a containing directory; it is not a reliable creation-time, path-history, content-version, change-token, or durability record
- XNU warns of inconsistent values for hard-linked items. The method does not reject hard links or infer which alias path the filesystem uses
- Filesystem precision and attribute support may vary. `EINVAL`, `ENOTSUP`, or a returned length containing only the header maps to `Unsupported`; other native errors preserve the mapped `FileError` category and POSIX code
- The operation reads no contents, accepts no arbitrary URL, starts no security scope, and preserves the existing concurrent rename of an already-open parent-directory limit
- No permission prompt, Info.plist key, entitlement, dependency, framework, workspace, or lock change is needed
- Apple lists `fgetattrlist` in the File Timestamp required-reason category. A host app that uses this method must declare an applicable approved reason in its final `PrivacyInfo.xcprivacy`

## Validation

Passed these non-test checks

- `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 fmt --package ios-files -- --check`
- `target/debug/xtask docs-check`
- Scoped `git diff --check`

No tests, linked probes, consumer execution, runtime queries, or live file operations are in scope

# B301: iOS Stored Backup-Time Marker Snapshot

## Scope

B301 adds `IosFiles::entry_stored_backup_time(&self, path: AppPath<'_>) -> Result<IosFileBackupTimeMarker, FileError>` as an iOS-only metadata query for one app-sandbox regular file or directory

The method reads only the filesystem-stored `ATTR_CMN_BKUPTIME` value through `fgetattrlist` on an opened descriptor. It does not add a timestamp concept to the portable `FileBackend`

## API and evidence

`IosFileBackupTimeMarker::seconds_since_unix_epoch()` returns signed `i64` seconds; `nanoseconds()` returns the exact `u32` nanosecond field in `0..1_000_000_000`

The installed iPhoneOS 26.5 SDK defines public `ATTR_CMN_BKUPTIME` as `0x00002000` in `sys/attr.h`. Locked `libc` 0.2.190 binds the constant, `libc::timespec`, and `fgetattrlist(fd, attrList, attrBuf, attrBufSize, options)`. `unistd.h` declares `fgetattrlist` with iOS 3.0 availability and the SDK gives no separate attribute availability annotation. The existing `ios-files` package floor remains iOS 10.0, so B301 adds no deployment-floor requirement

Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_CMN_BKUPTIME` as a read/write `timespec` for the time the filesystem object was last backed up, says it is for backup utilities, and says the filesystem stores but does not interpret the value. B301 exposes the read value only. It does not establish that any backup ran or that a backup includes the object

`fgetattrlist` returns a leading `u32` length followed by the aligned attribute. The shared checked parser requires the exact expected returned length, maps a header-only response to `Unsupported`, preserves signed seconds and the exact nanosecond field, and rejects a nanosecond value outside `0..1_000_000_000`

## Contract limits

- The method validates `AppPath`, walks parents with the existing no-follow descriptor helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, and accepts regular files and directories only. A final symlink or other entry kind returns `InvalidInput`
- It reports only the stored filesystem marker at query time. It is not proof of iOS, iCloud, or other backup completion, inclusion, freshness, durability, or recoverability. It does not set the marker or perform backup work
- Filesystem support and stored precision may vary. `EINVAL`, `ENOTSUP`, or a returned header-only response maps to `Unsupported`; malformed lengths or nanoseconds map to `InvalidInput`; other native errors retain the existing mapped category and POSIX code
- The query reads no contents, accepts no arbitrary URL, starts no security scope, and retains B1's concurrent rename of an already-open parent-directory limit
- No permission prompt, Info.plist key, entitlement, dependency, framework, or portable `FileBackend` operation is added
- Apple lists `fgetattrlist` in the [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). The host app must declare an applicable approved reason in its final `PrivacyInfo.xcprivacy`

## Validation

- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios` and the same command for `aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 clippy --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios -- -D warnings` and the same command for `aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios` and the same command for `aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 fmt -p ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and `git diff --check`
- No tests, live filesystem calls, consumers, runtime queries, or link/import probes ran. Checks used Rust 1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5; local Xcode is below the repository's Xcode 27.x baseline

# B226: iOS Regular-File Link-ID Snapshot

## Scope

B226 adds `IosFiles::regular_file_link_id_snapshot(AppPath) -> Result<IosFileLinkIdSnapshot, FileError>` as an iOS-only point-in-time metadata query for one regular file entry. It returns the opaque `u64` `ATTR_CMNEXT_LINKID` value through an open file descriptor. This is distinct from B99's `(st_dev, st_ino)` snapshot, B211's clone ID, B214's full-clone count, and B223's private-size count. It adds no portable `FileBackend` method and does not create hard links or change file contents.

## API and binding evidence

The installed iPhoneOS 26.5 SDK defines public `ATTR_CMNEXT_LINKID` as `0x00000010` in `sys/attr.h`. Locked `libc` 0.2.190 binds the attribute, `fgetattrlist`, and `FSOPT_ATTR_CMN_EXTENDED`; no local constant, dependency, or root lock change is needed. The SDK declares `fgetattrlist` in `unistd.h` as available from iOS 3.0. It provides no separate availability annotation for this extended-common field, so no additional OS floor or filesystem support claim is made.

Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_CMNEXT_LINKID` as a `u_int64_t` unique within a mounted volume. XNU says it is persistent on volumes that support `VOL_CAP_FMT_PERSISTENTOBJECTIDS`, such as HFS+ and APFS, and that on HFS+ and APFS a file-system object and its hard-link entries have distinct link IDs. The implementation does not query the volume capability, so the API makes no cross-mount persistence claim. The documented output layout is a leading `u_int32_t` byte length followed by the `u64` value; B226 uses a 12-byte buffer.

Apple lists `fgetattrlist` in the [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). A host app that uses this query must include an applicable approved reason in its `PrivacyInfo.xcprivacy`; this library does not choose or add a host reason. The query adds no permission, usage-description key, or entitlement.

## Contract limits

- The method opens the final component with `O_NOFOLLOW | O_NONBLOCK`, checks the open descriptor is a regular file, then calls `fgetattrlist` on that descriptor. A later path replacement or unlink does not retarget the query. The existing concurrent rename of an already-open parent directory can still move that directory outside the selected root before the final open.
- Treat the value as an opaque ID only within the current mounted volume. Do not use it as a content hash, clone-group ID, inode number, persistent identifier, cross-mount token, or authorization result. On HFS+ and APFS, hard-link entries may have distinct IDs even when they reference the same file data.
- Filesystems that do not support the attribute may return `EINVAL` or `ENOTSUP`, which the adapter maps to `Unsupported`; other native errors retain the existing `FileError` mapping. A short or unexpected buffer length returns `InvalidInput`.
- The query reads no file contents, accepts no arbitrary URL, starts no security scope, and does not change portable `FileBackend` behavior.

## Validation

- Passed device and arm64 Simulator checks:
  `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy:
  `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
  and `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed rustdoc:
  `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 fmt --package ios-files -- --check`,
  `target/debug/xtask docs-check`, and scoped `git diff --check`.
- No tests, linked probes, consumer execution, runtime queries, or live file operations were run.

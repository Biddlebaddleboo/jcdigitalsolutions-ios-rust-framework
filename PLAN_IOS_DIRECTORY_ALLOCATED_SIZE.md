# B235: iOS Directory-Object Allocated-Size Snapshot

## Scope

B235 adds `IosFiles::directory_allocated_size_snapshot(AppPath) -> Result<IosDirectoryAllocatedSizeSnapshot, FileError>` as an iOS-only metadata query for one app-sandbox directory. It reports the physical allocation of the directory object itself, not the total contents beneath it. It adds no portable `FileBackend` method and does not change directory traversal or write semantics.

## API and SDK evidence

The installed iPhoneOS 26.5 SDK defines public `ATTR_DIR_ALLOCSIZE` in `sys/attr.h`; locked `libc` 0.2.190 binds it and `fgetattrlist`, so no new constant, dependency, or root Cargo change is needed. The SDK declares `fgetattrlist` in `unistd.h` as available from iOS 3.0 and gives no separate availability annotation for this directory attribute.

Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_DIR_ALLOCSIZE` as an `off_t` number of bytes on disk used by the directory (its physical size). The attribute is requested in the `dirattr` field and returns in a 12-byte buffer: a leading `u_int32_t` length followed by the 8-byte `off_t`, aligned to four bytes. The implementation rejects a negative value or unexpected returned length.

Apple lists `fgetattrlist` in the [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). A host app that uses this query must include an applicable approved reason in its `PrivacyInfo.xcprivacy`; this library does not choose or add a host reason. The query adds no permission, usage-description key, or entitlement.

## Contract limits

- The method opens the directory through the validated `AppPath` with the backend's descriptor-relative `openat` traversal and `O_DIRECTORY | O_NOFOLLOW`, then queries the open descriptor. A later path replacement or unlink does not retarget the query. The existing concurrent rename of an already-open parent directory can still move that directory outside the selected root before the final open.
- The returned value describes only filesystem allocation for this directory object. It does not include the allocated size of child files, resource forks, or descendant directories, and must not be presented as a recursive folder-size total.
- The value is point-in-time, volume-format-dependent metadata. It is not an app quota, reservation, guarantee of capacity after deletion, or I/O performance signal. The call makes no cost or fast-query claim.
- Filesystems that do not support the attribute may return `EINVAL` or `ENOTSUP`, which the adapter maps to `Unsupported`; other native errors retain the existing `FileError` mapping. A negative `off_t` or unexpected buffer length returns `InvalidInput`.
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

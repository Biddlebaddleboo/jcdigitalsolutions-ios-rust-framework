# B223: iOS Regular-File Private-Size Snapshot

## Scope

B223 adds `IosFiles::regular_file_private_size_snapshot(AppPath) -> Result<IosFilePrivateSizeSnapshot, FileError>` as an iOS-only descriptor-bound metadata query. It returns a caller-owned `u64` byte count corresponding to the `off_t` `ATTR_CMNEXT_PRIVATESIZE` value. This is distinct from B109's allocated-size metadata and B214's count of full clone peers; it adds no portable `FileBackend` method and does not change clone or delete behavior.

## API and binding evidence

The installed iPhoneOS 26.5 SDK defines `ATTR_CMNEXT_PRIVATESIZE` in public `sys/attr.h` as `0x00000008` in the extended-common `forkattr` group. Locked `libc` 0.2.190 binds this attribute and `fgetattrlist`, so no local constant, dependency, or root lock change is needed. The SDK declares `fgetattrlist` in `unistd.h` as available from iOS 3.0; it gives no separate availability annotation for the attribute, so no additional OS floor or runtime support claim is made.

Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the field as an `off_t` count of bytes not trapped inside a clone or snapshot that would be freed immediately if the file were deleted. The output buffer for this single field is 12 bytes: a leading `u_int32_t` byte length followed by the 8-byte `off_t`, aligned to four bytes. The implementation rejects negative values and unexpected buffer lengths.

Apple lists `fgetattrlist` in the [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). A host app that uses this query must include an applicable approved reason in its `PrivacyInfo.xcprivacy`; this library does not choose or add a host reason. The query adds no permission, usage-description key, or entitlement.

## Contract limits

- The method opens the final component with `O_NOFOLLOW | O_NONBLOCK`, checks the open descriptor is a regular file, then calls `fgetattrlist` on that descriptor. A later path replacement or unlink does not retarget the query. The existing concurrent rename of an already-open parent directory can still move that directory outside the selected root before the final open.
- The returned value is a point-in-time filesystem report with XNU's documented immediate-free meaning. It is not a capacity reservation, app quota, guarantee that a later delete will free the same amount, or guarantee of a later write. Other files, snapshots, clones, and concurrent mutations can affect what would be freed.
- Filesystems that do not support this attribute may return `EINVAL` or `ENOTSUP`, which the adapter maps to `Unsupported`; other native errors retain the existing `FileError` mapping. A negative `off_t` or unexpected buffer length returns `InvalidInput`.
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

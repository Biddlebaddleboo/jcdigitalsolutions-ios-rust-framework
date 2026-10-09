# B214: iOS Regular-File Full-Clone Count Snapshot

## Scope

B214 adds `IosFiles::regular_file_full_clone_count_snapshot(AppPath) -> Result<IosFileFullCloneCountSnapshot, FileError>` as an iOS-only descriptor-bound metadata query. It returns XNU's fixed-width `u32` count of full clones reported for one regular file. This is distinct from B208's raw per-file extended flags and B211's opaque per-file clone ID. It does not add a portable `FileBackend` method or change B196 clone behavior.

## API and SDK evidence

The installed iPhoneOS 26.5 SDK defines public `ATTR_CMNEXT_CLONE_REFCNT` as `0x00001000` in `sys/attr.h`, in the `forkattr` group used with `FSOPT_ATTR_CMN_EXTENDED`. Locked `libc` 0.2.190 binds `fgetattrlist`, `ATTR_CMNEXT_CLONEID`, and `ATTR_CMNEXT_EXT_FLAGS`, but not this macro, so the implementation uses a local Rust constant with the exact public SDK value. The SDK declares `fgetattrlist` in `unistd.h` as available from iOS 3.0; it gives no separate availability annotation for the extended-common attribute, so no additional OS floor or runtime support claim is made.

Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_CMNEXT_CLONE_REFCNT` as a `u_int32_t` number of full clones, each sharing all of its blocks with the queried file. The same documentation defines the output layout as a leading `u_int32_t` byte length followed by the requested attribute, aligned to four bytes. B214 uses an eight-byte buffer for this one field.

Apple lists `fgetattrlist` in the [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). A host app that uses this query must include an applicable approved reason in its `PrivacyInfo.xcprivacy`; this library does not choose or add a host reason. The query adds no permission, usage-description key, or entitlement.

## Contract limits

- The method opens the final component with `O_NOFOLLOW | O_NONBLOCK`, checks the open descriptor is a regular file, then calls `fgetattrlist` on that descriptor. A later path replacement or unlink does not retarget the query. The existing concurrent rename of an already-open parent directory can still move that directory outside the selected root before the final open.
- The result is the filesystem's point-in-time `u32` full-clone count. It excludes partial block-sharing peers, does not enumerate paths or clone IDs, and does not establish a durable identity or a stable sharing relationship. Concurrent clone creation or removal may change later results.
- Filesystems that do not support the requested attribute may return `EINVAL` or `ENOTSUP`, which the adapter maps to `Unsupported`; other native errors retain the existing `FileError` mapping. An unexpected buffer length returns `InvalidInput`.
- The query reads no file contents, accepts no arbitrary URL, starts no security scope, and does not change portable `FileBackend` behavior. It adds no dependency or root Cargo change.

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

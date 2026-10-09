# B251: iOS Regular-File Resource-Fork Logical-Size Snapshot

## Status

Implemented as an iOS-only metadata snapshot in `ios-files`. It does not change the portable
`framework-files::FileBackend` contract.

## API

`IosFiles::regular_file_resource_fork_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileResourceForkSizeSnapshot, FileError>` returns the current `ATTR_FILE_RSRCLENGTH` value in bytes. `IosFileResourceForkSizeSnapshot::bytes()` exposes that fixed-width `u64` value.

The implementation validates the `AppPath`, traverses parent components using the existing
descriptor-relative no-follow helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, checks
the opened descriptor is a regular file, and calls `fgetattrlist` for `ATTR_FILE_RSRCLENGTH` on that
descriptor. The result remains bound to the opened file if its name is concurrently replaced after
open. It is a point-in-time value; concurrent writes may change the reported length.

## Meaning and limits

Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_RSRCLENGTH` as an `off_t` length
in bytes for the resource fork. The query is distinct from B242's all-fork logical total,
B90's `st_size`-based file size, and B248's resource-fork allocated-byte count. A zero value is only
the native reported length and does not establish that a resource fork is absent. The operation
does not enumerate fork names, open a resource fork, read contents, or grant additional file
access. Filesystem support may vary; `EINVAL` or `ENOTSUP` from the attribute query maps to
`Unsupported`.

The operation accepts only validated app-sandbox `AppPath` values. It does not accept arbitrary
URLs, start security-scoped access, add coordination, or alter the existing limit that a concurrent
native rename of an already-open parent can move that directory outside the selected root. It does
not change portable file semantics.

Apple lists `fgetattrlist` as a File Timestamp required-reason API. An app that uses this operation
must include an applicable approved File Timestamp reason in its final `PrivacyInfo.xcprivacy`; the
library does not select the host's reason or supply a privacy manifest.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_RSRCLENGTH` as the resource
  fork's logical length in bytes, separately from `ATTR_FILE_DATALENGTH` for the data fork and
  `ATTR_FILE_TOTALSIZE` for all forks:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_RSRCLENGTH` as `0x00001000` in
  `sys/attr.h`; locked `libc` 0.2.190 binds the constant and `fgetattrlist`. The SDK declares
  `fgetattrlist` in `unistd.h` as available from iOS 3.0 and gives no separate availability
  annotation for this attribute. No new dependency, local constant, or root lock edit is needed.
- Apple's current XNU source has the `va_rsrc_length` resource-fork field and retains the separate
  `va_data_size` and `va_total_size` values:
  <https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/vnode.h>
- Apple's required-reason API reference lists `fgetattrlist` under File Timestamp:
  <https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype>

## Validation

Passed:

- `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 fmt --package ios-files -- --check`
- `target/debug/xtask docs-check`
- scoped `git diff --check`

No tests, linked probes, consumer execution, runtime queries, or live file operations were run.

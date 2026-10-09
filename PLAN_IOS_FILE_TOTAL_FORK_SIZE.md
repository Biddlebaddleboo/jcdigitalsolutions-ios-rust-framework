# B242: iOS Regular-File Total Fork Size Snapshot

## Status

Implemented as an iOS-only metadata snapshot in `ios-files`. It does not change the portable
`framework-files::FileBackend` contract.

## API

`IosFiles::regular_file_total_fork_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileTotalForkSizeSnapshot, FileError>` returns the current `ATTR_FILE_TOTALSIZE` value in bytes. `IosFileTotalForkSizeSnapshot::bytes()` exposes that fixed-width `u64` value.

The implementation validates the `AppPath`, traverses parent components using the existing
descriptor-relative no-follow helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, checks
the opened descriptor is a regular file, and calls `fgetattrlist` for `ATTR_FILE_TOTALSIZE` on that
descriptor. The result remains bound to the opened file if its name is concurrently replaced after
open. It is a point-in-time value; concurrent writes may change the reported total.

## Meaning and limits

XNU defines `va_total_size` as the size in bytes of all forks. `ATTR_FILE_TOTALSIZE` is therefore a
logical all-fork total, not necessarily the data-fork byte length returned by `st_size` or
`IosFiles::regular_file_size`. It does not report allocated disk blocks, and it is not a buffer
size for `FileBackend::read`. This API does not expose a fork list, open a resource fork, or read
file contents. Filesystem support may vary; `EINVAL` or `ENOTSUP` from the attribute query maps to
`Unsupported`.

The operation accepts only validated app-sandbox `AppPath` values. It does not accept arbitrary
URLs, start security-scoped access, add coordination, or alter the existing limit that a
concurrent native rename of an already-open parent can move that directory outside the selected
root. It does not alter portable file semantics.

Apple lists `fgetattrlist` as a File Timestamp required-reason API. An app that uses this operation
must include an applicable approved File Timestamp reason in its final `PrivacyInfo.xcprivacy`; the
library does not select the host's reason or supply a privacy manifest.

## Evidence

- XNU's public `getattrlist(2)` manual defines `fgetattrlist` as querying metadata through the
  supplied file descriptor, notes that not all volumes support every attribute, and defines the
  attribute buffer's leading length field: <https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2>
- XNU's public `vnode.h` defines `va_total_size` as the byte size of all forks, distinct from
  `va_data_size` for the fork managed by the current vnode:
  <https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/vnode.h>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_TOTALSIZE` as `0x00000002` in
  `sys/attr.h`; locked `libc` 0.2.190 binds the constant and `fgetattrlist`. The SDK declares
  `fgetattrlist` in `unistd.h` as available from iOS 3.0 and gives no separate availability
  annotation for `ATTR_FILE_TOTALSIZE`. No new dependency, local attribute constant, or root lock
  edit is needed. The operation uses no Objective-C, Foundation, Swift, permission, entitlement,
  or usage-description key.
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

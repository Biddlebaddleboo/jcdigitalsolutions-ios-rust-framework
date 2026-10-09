# B248: iOS Regular-File Resource-Fork Allocated-Size Snapshot

## Status

Implemented as an iOS-only metadata snapshot in `ios-files`. It does not change the portable
`framework-files::FileBackend` contract.

## API

`IosFiles::regular_file_resource_fork_allocated_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileResourceForkAllocatedSizeSnapshot, FileError>` returns the current `ATTR_FILE_RSRCALLOCSIZE` value in bytes. `IosFileResourceForkAllocatedSizeSnapshot::bytes()` exposes that fixed-width `u64` value.

The implementation validates the `AppPath`, traverses parent components using the existing
descriptor-relative no-follow helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, checks
the opened descriptor is a regular file, and calls `fgetattrlist` for `ATTR_FILE_RSRCALLOCSIZE` on
that descriptor. The result remains bound to the opened file if its name is concurrently replaced
after open. It is a point-in-time value; concurrent writes may change the reported allocation.

## Meaning and limits

Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_RSRCALLOCSIZE` as an `off_t`
count of bytes on disk used by the resource fork. It excludes data-fork allocation and is distinct
from `ATTR_FILE_ALLOCSIZE`, which covers all forks. A zero result is only the native reported size;
this API does not infer that no resource fork exists. It does not enumerate fork names, open a
resource fork, read contents, or grant additional file access. It is not a guarantee of exclusive
physical-device storage. Filesystem support may vary; `EINVAL` or `ENOTSUP` from the attribute
query maps to `Unsupported`.

The operation accepts only validated app-sandbox `AppPath` values. It does not accept arbitrary
URLs, start security-scoped access, add coordination, or alter the existing limit that a concurrent
native rename of an already-open parent can move that directory outside the selected root. It does
not change portable file semantics.

Apple lists `fgetattrlist` as a File Timestamp required-reason API. An app that uses this operation
must include an applicable approved File Timestamp reason in its final `PrivacyInfo.xcprivacy`; the
library does not select the host's reason or supply a privacy manifest.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_RSRCALLOCSIZE` as bytes on
  disk used by the resource fork, separately from data-fork allocation and the all-forks allocation
  attribute:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_RSRCALLOCSIZE` as `0x00002000` in
  `sys/attr.h`; locked `libc` 0.2.190 binds the constant and `fgetattrlist`. The SDK declares
  `fgetattrlist` in `unistd.h` as available from iOS 3.0 and gives no separate availability
  annotation for this attribute. No new dependency, local constant, or root lock edit is needed.
- Apple's current XNU source distinguishes `va_total_alloc` (all forks) from `va_data_alloc` (data
  fork); it also shows the filesystem-facing `va_rsrc_alloc` field:
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

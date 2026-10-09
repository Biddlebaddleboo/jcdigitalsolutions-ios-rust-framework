# B244: iOS Regular-File Data-Fork Allocated-Size Snapshot

## Status

Implemented as an iOS-only metadata snapshot in `ios-files`. It does not change the portable
`framework-files::FileBackend` contract.

## API

`IosFiles::regular_file_data_fork_allocated_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileDataForkAllocatedSizeSnapshot, FileError>` returns the current `ATTR_FILE_DATAALLOCSIZE` value in bytes. `IosFileDataForkAllocatedSizeSnapshot::bytes()` exposes that fixed-width `u64` value.

The implementation validates the `AppPath`, traverses parent components using the existing
descriptor-relative no-follow helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, checks
the opened descriptor is a regular file, and calls `fgetattrlist` for `ATTR_FILE_DATAALLOCSIZE` on
that descriptor. The result remains bound to the opened file if its name is concurrently replaced
after open. It is a point-in-time value; concurrent writes may change the reported allocation.

## Meaning and limits

Apple's `getattrlist(2)` reference defines `ATTR_FILE_DATAALLOCSIZE` as an `off_t` count of bytes on
disk used by the data fork (its physical size). It is data-fork-specific and excludes resource-fork
allocation. It differs in contract from B109's `st_blocks` count, which is per-entry in 512-byte
units; the API does not promise a fixed conversion or exclusive physical-device accounting. It
does not read file contents, expose fork names, open a resource fork, or change portable `read`
semantics. Filesystem support may vary; `EINVAL` or `ENOTSUP` from the attribute query maps to
`Unsupported`.

The operation accepts only validated app-sandbox `AppPath` values. It does not accept arbitrary
URLs, start security-scoped access, add coordination, or alter the existing limit that a concurrent
native rename of an already-open parent can move that directory outside the selected root. It does
not change portable file semantics.

Apple lists `fgetattrlist` as a File Timestamp required-reason API. An app that uses this operation
must include an applicable approved File Timestamp reason in its final `PrivacyInfo.xcprivacy`; the
library does not select the host's reason or supply a privacy manifest.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_DATAALLOCSIZE` as bytes on
  disk used by the data fork, separately from `ATTR_FILE_RSRCALLOCSIZE` for the resource fork and
  `ATTR_FILE_ALLOCSIZE` for all forks:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_DATAALLOCSIZE` as `0x00000400` in
  `sys/attr.h`; locked `libc` 0.2.190 binds the constant and `fgetattrlist`. The SDK declares
  `fgetattrlist` in `unistd.h` as available from iOS 3.0 and gives no separate availability
  annotation for this attribute. No new dependency, local constant, or root lock edit is needed.
- Apple's current XNU source retains the `fgetattrlist` implementation and all-fork `va_total_alloc`
  versus per-data-fork `va_data_alloc` distinction:
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

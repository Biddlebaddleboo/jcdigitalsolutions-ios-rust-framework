# B253: All-Forks Allocated-Size No-Go

## Audit result

Do not add `ATTR_FILE_ALLOCSIZE` as another regular-file allocation snapshot in this slice. XNU
defines it as an `off_t` count of bytes on disk used by all file forks. B109 already provides the
existing per-entry `st_blocks` allocation count in 512-byte units, while B244 and B248 expose
separate data-fork and resource-fork allocation snapshots. No current `ios-files` operation needs
another all-fork allocation encoding. No fixed conversion between `st_blocks` and
`ATTR_FILE_ALLOCSIZE` is claimed.

The no-go is about incremental app-facing value, not API feasibility. The public attribute is
available to request, but a second aggregate metric would duplicate the existing storage-reporting
surface without a caller-defined use. The portable `framework-files::FileBackend` remains unchanged.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_ALLOCSIZE` as an `off_t` count
  of bytes on disk used by all file forks:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_ALLOCSIZE` as `0x00000004` in
  `sys/attr.h`; locked `libc` 0.2.190 binds it. The SDK provides no separate attribute
  availability annotation beyond `fgetattrlist`'s iOS 3.0 declaration. Any actual call would be
  filesystem-dependent and would require an applicable approved File Timestamp reason in the host
  `PrivacyInfo.xcprivacy`.
- B109's existing `st_blocks` API and the focused B244/B248 data/resource fork allocation APIs are
  documented in [`PLAN_IOS_FILE_ALLOCATED_BLOCKS.md`](PLAN_IOS_FILE_ALLOCATED_BLOCKS.md),
  [`PLAN_IOS_FILE_DATA_FORK_ALLOCATED_SIZE.md`](PLAN_IOS_FILE_DATA_FORK_ALLOCATED_SIZE.md), and
  [`PLAN_IOS_FILE_RESOURCE_FORK_ALLOCATED_SIZE.md`](PLAN_IOS_FILE_RESOURCE_FORK_ALLOCATED_SIZE.md).

## Closure criterion

Revisit only if a caller needs an all-fork allocation byte value for an operation or UI that
cannot use B109 or the per-fork B244/B248 values. Do not claim equivalence or a fixed conversion
between these encodings without filesystem-specific evidence.

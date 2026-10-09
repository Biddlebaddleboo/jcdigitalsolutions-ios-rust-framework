# B237: iOS Directory Logical Data-Length No-Go

## Candidate

`ATTR_DIR_DATALENGTH` is a public directory attribute that returns a logical byte length. It could be added as a point-in-time scalar beside B235's directory allocation size.

## Evidence

- The installed iPhoneOS 26.5 SDK defines `ATTR_DIR_DATALENGTH` as `0x00000020` in public `sys/attr.h`; locked `libc` 0.2.190 binds it and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the field as an `off_t` length of the directory in bytes (its logical size), returned only for directories. It is distinct from `ATTR_DIR_ALLOCSIZE`, which B235 exposes as the physical allocation of the directory object.
- The field measures the directory object, not the sum of child files or descendant directories. `AppPath` and `directory_entry_count` already provide path and direct-entry information; this facade has no operation that consumes the directory object's logical byte length.
- The public `fgetattrlist` declaration is available from iOS 3.0; the SDK gives no separate availability annotation for this field. Any use would also require an applicable approved File Timestamp reason in the host `PrivacyInfo.xcprivacy`.

## Decision

No API or dependency change for B237. A second directory-object size would be easy to misread as a folder-content length and adds no supported operation beyond B235's metadata-only physical allocation snapshot. Do not use `ATTR_DIR_DATALENGTH` as a recursive size, child count, or storage-reclamation estimate.

## Closure criteria

Revisit only if a concrete app-data feature needs the logical byte length of the directory object itself, and its UI/API explicitly distinguishes that from child contents and B235's allocated bytes.

## Validation

This report uses the installed public SDK header, locked `libc` source, and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.

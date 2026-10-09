# B238: iOS Directory I/O Block-Size No-Go

## Candidate

`ATTR_DIR_IOBLOCKSIZE` reports a block-size value for a directory. A point-in-time query would be a small fixed-width metadata API.

## Evidence

- The installed iPhoneOS 26.5 SDK defines `ATTR_DIR_IOBLOCKSIZE` as `0x00000010` in public `sys/attr.h`; locked `libc` 0.2.190 binds it and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as a `u_int32_t` optimal block size when reading or writing data.
- `ios-files` reads directory names through `readdir`; callers do not issue raw directory-data I/O or control its buffers. The value would not tune the facade's file-content operations either. B155 already exposes volume `f_iosize` only as a general sizing hint, with no alignment or performance guarantee.
- The public `fgetattrlist` declaration is available from iOS 3.0; the SDK gives no separate availability annotation for this field. Any use would also require an applicable approved File Timestamp reason in the host `PrivacyInfo.xcprivacy`.

## Decision

No API or dependency change for B238. In this facade, a directory block-size value has no consumer operation, buffer, or tuning control. Do not present it as a required alignment, a faster enumeration setting, or a performance guarantee.

## Closure criteria

Revisit only if `ios-files` gains a public raw directory-data I/O operation whose buffer sizing can use this exact filesystem hint, with an explicit non-mandatory and non-performance-guarantee contract.

## Validation

This report uses the installed public SDK header, locked `libc` source, and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.

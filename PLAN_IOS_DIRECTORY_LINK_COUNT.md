# B240: iOS Directory Hard-Link Count No-Go

## Candidate

`ATTR_DIR_LINKCOUNT` returns a fixed-width count for a directory. It could be mistaken for a direct child-directory count or a measure of directory contents.

## Evidence

- The installed iPhoneOS 26.5 SDK defines `ATTR_DIR_LINKCOUNT` as `0x00000001` in public `sys/attr.h`; locked `libc` 0.2.190 binds it and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines it as a `u_int32_t` number of hard links to the directory, excluding historical `.` and `..` entries. On filesystems that do not support directory hard links, the value is `1`.
- This attribute does not define a count of direct child directories. B134 already counts direct entries classified as directories with no-follow `fstatat`, and the facade does not create directory hard links.
- The public `fgetattrlist` declaration is available from iOS 3.0; the SDK gives no separate availability annotation for this field. Any use would also require an applicable approved File Timestamp reason in the host `PrivacyInfo.xcprivacy`.

## Decision

No API or dependency change for B240. The hard-link count cannot serve as a portable subdirectory count or app-data quota. Keep B134's direct-entry scan as the supported operation; do not subtract `.` and `..` or infer child counts from this field.

## Closure criteria

Revisit only if the facade gains a supported directory-hard-link operation or a caller has a concrete need for the raw mount-specific directory-link count, with no child-count or portability claim.

## Validation

This report uses the installed public SDK header, locked `libc` source, and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.

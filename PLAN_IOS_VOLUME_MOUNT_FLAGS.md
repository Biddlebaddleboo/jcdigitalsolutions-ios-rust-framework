# B401: iOS Volume Mount-Flags No-Go

## Candidate

`ATTR_VOL_MOUNTFLAGS` returns the mount flags for a volume. A raw flag set might appear to show which filesystem operations are allowed

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_MOUNTFLAGS` as `0x00004000` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_MOUNTFLAGS`; the XNU manual defines its result as `u_int32_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) says the value copies the flags passed to `mount(2)` and is equivalent to `statfs.f_flags`.
- B152 already exposes the one mount property used by this facade, `MNT_RDONLY`; B161 rejects other mount flags as effective path or app-authorization signals. No current file operation uses the raw set.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add a raw mount-flags API. It duplicates `statfs.f_flags` and adds no supported operation. Raw flags cannot establish effective sandbox access, path writability, or later operation success. Keep B152's narrow read-only mount snapshot and B161's no-go boundary

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

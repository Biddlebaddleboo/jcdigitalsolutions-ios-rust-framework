# B422: iOS Volume Extended-Mount-Flags No-Go

## Candidate

`ATTR_VOL_MOUNTEXTFLAGS` returns a volume's extended mount flags. It might appear to provide additional path or operation policy beyond `f_flags`

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_MOUNTEXTFLAGS` as `0x00080000` in `sys/attr.h` and `struct statfs.f_flags_ext` as `uint32_t` in `sys/mount.h`.
- Locked `libc` 0.2.190 binds `statfs.f_flags_ext` but does not bind `ATTR_VOL_MOUNTEXTFLAGS`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the attribute as `u_int32_t` and says it is equivalent to `statfs.f_flags_ext`.
- B152 exposes only `MNT_RDONLY`; B161 declines other mount flags as effective path/app authorization signals. No current `ios-files` operation consumes the extended flags.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add `ATTR_VOL_MOUNTEXTFLAGS`. It duplicates the public `fstatfs.f_flags_ext` field and adds no supported operation or effective-access guarantee. If a future operation needs one documented flag, query and document that specific behavior rather than exposing a raw mount policy set

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK headers, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

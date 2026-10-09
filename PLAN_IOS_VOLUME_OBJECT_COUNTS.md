# B392: iOS Volume Object Counts No-Go

## Candidate

`ATTR_VOL_OBJCOUNT`, `ATTR_VOL_FILECOUNT`, and `ATTR_VOL_DIRCOUNT` return current filesystem-object, file, and directory counts for a mounted volume. They might appear to describe app file capacity

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_OBJCOUNT` as `0x00000100`, `ATTR_VOL_FILECOUNT` as `0x00000200`, and `ATTR_VOL_DIRCOUNT` as `0x00000400` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds all three attributes; Apple's XNU manual defines each result as `u_int32_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) describes the values as current counts on the volume. It does not define an app/container count, a per-user limit, or a guarantee that a later create will succeed.
- B149 already declines volume-wide `statfs.f_files` and `f_ffree` as app-data capacity values. B128 and B134 count direct entries in a selected app directory when the caller needs a local count.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for these attributes; `ios-files` has an iOS 10.0 floor.
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. Host use would need an applicable approved reason in `PrivacyInfo.xcprivacy`.

## Decision

Do not add these volume counters. A volume-wide count cannot say how many entries belong to the app or whether a later create will succeed. It adds no app-data operation beyond B128/B134's local directory counts and can be mistaken for file-creation capacity

Revisit only if Apple documents an app-visible per-container count or a supported caller operation that uses the volume-wide count without implying an app limit

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

# B395: iOS Volume Mount-Point Path No-Go

## Candidate

`ATTR_VOL_MOUNTPOINT` returns the mounted volume's path. It could appear to help a caller map an `AppDirectory` to a native absolute path

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_MOUNTPOINT` as `0x00001000` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_MOUNTPOINT` and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as an `attrreference` to a UTF-8, null-terminated mount-point path, equivalent to `statfs.f_mntonname`, with length no greater than `MAXPATHLEN`.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor.
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. Host use would need an applicable approved reason in `PrivacyInfo.xcprivacy`.

## Decision

Do not add a mount-point path API. The value is a host filesystem path, not an `AppPath`, app-container identity, or URL that grants access. The facade already resolves semantic app roots through Foundation and keeps later operations descriptor-relative; no supported operation needs a second path form. Returning the mount point would expose an implementation-specific absolute path without a supported app-data decision

Revisit only if a public iOS caller operation needs the volume's mount point and Apple documents the path's relevant lifetime and privacy contract

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

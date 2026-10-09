# B404: iOS Volume Mounted-Device Value No-Go

## Candidate

`ATTR_VOL_MOUNTEDDEVICE` returns the value that identifies the mounted source device or mount. It might appear useful as a volume identity or diagnostic string

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_MOUNTEDDEVICE` as `0x00008000` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_MOUNTEDDEVICE` and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the result as an `attrreference` equivalent to `statfs.f_mntfromname`. For local volumes it is the mounted device path; for network volumes it is a unique string that identifies the mount. Its length is no greater than `MAXPATHLEN`.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add this device/mount string. It duplicates `statfs.f_mntfromname`, can reveal a host device path, and does not establish a stable app-container or persistent volume identity. `ios-files` has no mount, device, or network-volume operation that consumes it. Keep the filesystem's mounted-source detail out of `AppPath` and app-data contracts

Revisit only if a supported iOS operation requires the mounted-source value and defines its path/string privacy and lifetime

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

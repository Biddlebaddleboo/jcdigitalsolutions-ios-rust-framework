# B416: iOS Volume Quota-Size No-Go

## Candidate

`ATTR_VOL_QUOTA_SIZE` reports the maximum size of a volume. Its name might suggest an app or user quota

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_QUOTA_SIZE` as `0x10000000` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_QUOTA_SIZE`; Apple's XNU manual defines its result as `off_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) describes the value as the maximum size of the volume in bytes. It does not define an app-container or per-user quota.
- B146 already exposes the filesystem-reported total volume capacity. `ios-files` opens semantic app directories on a shared host volume and has no API that maps a volume-level maximum to an app budget.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add a quota-size API. The documented value is volume-level, not an app or user quota, and no current operation uses it. B146's volume capacity and B137's available-capacity values remain distinct from any app-container limit or create/write guarantee

Revisit only if Apple documents a per-container or per-user limit that the app-data facade can query for its semantic root

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

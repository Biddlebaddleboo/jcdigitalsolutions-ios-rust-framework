# B362: iOS Volume UUID No-Go

## Candidate

`ATTR_VOL_UUID` exposes a file-system UUID. A fixed 16-byte snapshot might let a caller compare volumes, but `ios-files` has no operation that needs a volume identity.

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_UUID` as `0x00040000` in `sys/attr.h` and describes its value as `uuid_t`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_UUID` and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) calls the value the file-system UUID and says it is typically a version 5 UUID. It does not define a persistence guarantee across remount, restore, clone, or replacement, nor identify it as an app-container ID.
- A volume request must include `ATTR_VOL_INFO`. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor.
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. Any host use would need an applicable approved reason in its `PrivacyInfo.xcprivacy`.

## Decision

Do not add a volume UUID API. No supported file operation needs a volume identifier, and the public contract gives no app-container meaning or stable cross-mount lifetime. A raw UUID could enable volume correlation without a current user-facing need. Do not use it as a cache key, persistent identity, clone preflight, or sandbox identity. Revisit only for a concrete caller decision and a documented lifetime contract.

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement.

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe.

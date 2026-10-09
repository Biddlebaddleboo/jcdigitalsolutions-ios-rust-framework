# B374: iOS Volume Maximum-Object Count No-Go

## Candidate

`ATTR_VOL_MAXOBJCOUNT` reports the maximum number of filesystem objects on a volume. It could look like an app-container file-count limit

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_MAXOBJCOUNT` as `0x00000800` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_MAXOBJCOUNT`; Apple's XNU manual defines its result as `u_int32_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) describes the maximum number of filesystem objects that can be stored on the volume. It does not define an app/container quota, a per-user limit, or a guarantee that a later create will succeed.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor.
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. Host use would need an applicable approved reason in `PrivacyInfo.xcprivacy`.

## Decision

Do not add a volume maximum-object count. It would describe a whole-volume limit rather than a sandbox budget, and it cannot authorize or predict a create. B149 already rejects volume-wide node counts as an app/container budget; B128 and B134 count direct entries in a selected app directory when the caller needs a local count. A successful create remains authoritative

Revisit only for a concrete user-facing volume-capacity need that can label this whole-volume maximum without implying an app quota or create guarantee

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

# B419: iOS Volume Reserved-Size No-Go

## Candidate

`ATTR_VOL_RESERVED_SIZE` reports a minimum size for the volume. Its name might suggest space reserved for an app or for a later write

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_RESERVED_SIZE` as `0x20000000` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_RESERVED_SIZE`; Apple's XNU manual defines its result as `off_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) describes the value as the minimum size of the volume in bytes. It does not define this as an app allocation, an app quota, or space reserved for a later operation.
- B137 reports the bytes available to non-superusers on the volume; B146 reports total volume capacity. The file facade has no space-reservation operation, and neither value guarantees a later create or write.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add a reserved-size API. The volume minimum is not app-reserved capacity and cannot guide or guarantee a sandbox write. B137/B146 remain the supported volume-wide availability and total-capacity snapshots

Revisit only if Apple documents app-visible reserved capacity or the facade gains a reservation API with matching semantics

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

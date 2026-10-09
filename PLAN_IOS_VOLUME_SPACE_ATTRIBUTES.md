# B428: iOS Volume Space-Attribute No-Go

## Candidate

`ATTR_VOL_SIZE`, `ATTR_VOL_SPACEFREE`, and `ATTR_VOL_SPACEAVAIL` return volume-wide byte counts. They may appear to add storage-capacity values to `ios-files`.

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_SIZE` as `0x00000004`, `ATTR_VOL_SPACEFREE` as `0x00000008`, and `ATTR_VOL_SPACEAVAIL` as `0x00000010` in `sys/attr.h`; locked `libc` 0.2.190 binds all three.
- Apple's XNU [`getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines all three payloads as `off_t`. `ATTR_VOL_SIZE` is total volume size; `ATTR_VOL_SPACEFREE` is volume free space; `ATTR_VOL_SPACEAVAIL` is space available to non-privileged processes and is explicitly equivalent to `f_bavail` as `ATTR_VOL_SPACEFREE` is to `f_bfree`.
- B146 already reports total volume capacity using checked `f_blocks * f_bsize`; B137 reports non-superuser-available bytes using checked `f_bavail * f_bsize`; B143 rejects `f_bfree` as reserved blocks may not be available to the sandbox. These are separate point-in-time queries and do not promise app quota or later-write success.
- Any `getattrlist` volume request must include `ATTR_VOL_INFO`. The installed SDK declares `fgetattrlist` from iOS 3.0, below `ios-files`' iOS 10.0 floor; the call is listed in Apple's File Timestamp required-reason category. This does not improve on the existing B137/B146 Disk Space snapshots.

## Decision

Do not add these attributes. They provide no distinct supported app-data operation beyond B137/B146, while `ATTR_VOL_SPACEFREE` repeats B143's value that must not be presented as sandbox-available capacity. Their independent `fgetattrlist` calls would also remain point-in-time snapshots, not an atomic capacity tuple.

Revisit only if a concrete caller needs an `off_t`-specific volume-space value with semantics not already covered by B137/B146, and the use has an approved privacy reason.

This audit adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement.

## Validation

Checked the installed iPhoneOS 26.5 SDK declarations, locked `libc` 0.2.190 bindings, existing B137/B143/B146 contracts, and Apple's XNU manual. No build, test, runtime query, consumer, live filesystem call, or probe was run.

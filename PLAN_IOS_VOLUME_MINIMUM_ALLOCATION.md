# B368: iOS Volume Minimum-Allocation No-Go

## Candidate

`ATTR_VOL_MINALLOCATION` reports the minimum allocation size of a mounted volume. A fixed-width byte snapshot might appear useful to predict the cost of a small file

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_MINALLOCATION` as `0x00000020` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_MINALLOCATION`; the XNU manual defines its result as `off_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) calls this the minimum allocation size in bytes and says a one-byte file will consume this amount. It does not define an app quota or a general per-path future allocation result.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor.
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. Host use would need an applicable approved reason in `PrivacyInfo.xcprivacy`.

## Decision

Do not add a minimum-allocation snapshot now. No supported operation uses the value to choose an allocation strategy or report a path's actual cost. B90 and B109 report a file's logical length and observed allocated blocks; B244 and B248 report the data/resource fork allocation snapshots. The volume minimum does not replace those per-entry results, establish an app quota, or guarantee that a later create/write succeeds

Revisit only for a concrete caller decision that uses the documented minimum for a one-byte file, with no claim that the value predicts general file allocation

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

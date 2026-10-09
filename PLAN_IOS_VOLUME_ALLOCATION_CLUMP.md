# B371: iOS Volume Allocation-Clump No-Go

## Candidate

`ATTR_VOL_ALLOCATIONCLUMP` reports an allocation-clump size for a volume. It might appear to guide write chunk sizes or reduce fragmentation

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_ALLOCATIONCLUMP` as `0x00000040` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_ALLOCATIONCLUMP`; the XNU manual defines its result as `off_t`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) says the filesystem attempts to allocate this much as a file grows to reduce fragmentation. The wording is an allocation hint, not a caller buffer rule or a guarantee of allocation behavior.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor.
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. Host use would need an applicable approved reason in `PrivacyInfo.xcprivacy`.

## Decision

Do not add an allocation-clump API. `framework-files` has no streaming write handle or buffer-size policy that could consume the hint; its write operations pass caller-owned byte slices to the filesystem. This value does not set allocation size, guarantee later write behavior, or predict a path's actual allocation. B90, B109, B244, and B248 report observed per-entry size/allocation values instead

Revisit only if the facade gains a streaming write policy with a documented use for this advisory volume value

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

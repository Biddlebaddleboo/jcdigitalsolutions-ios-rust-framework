# B413: iOS Raw Volume-Capability Set No-Go

## Candidate

`ATTR_VOL_CAPABILITIES` returns volume-format and volume-interface capability bit sets. A raw snapshot might appear to preflight filesystem operations

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_CAPABILITIES` as `0x00020000` in `sys/attr.h`.
- The SDK and locked `libc` 0.2.190 define `vol_capabilities_attr_t` as two four-word sets, `capabilities` and `valid`; locked `libc` binds the attribute, structure, and documented `VOL_CAP_FMT_*` / `VOL_CAP_INT_*` masks.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) describes the value as optional features supported by a volume. A bit only has meaning when its corresponding `valid` bit is set; unknown/reserved bits cannot imply support.
- Existing operations already use narrower observations where useful: B172/B175 expose cached Foundation rename/name-support keys, B199 exposes a cached file-cloning support hint, and B170 performs the effective-access query itself. No current operation consumes the remaining raw capability set.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add a generic volume-capability bit-set API. It adds no operation beyond the existing narrow support snapshots or direct operation results, and a raw set could be misread without per-bit validity handling and a supported consumer

Revisit only when a new facade operation needs a documented capability bit to guide one concrete action; expose that bit with its validity and limits rather than a generic feature promise

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

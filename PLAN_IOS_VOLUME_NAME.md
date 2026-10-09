# B398: iOS Volume Name No-Go

## Candidate

`ATTR_VOL_NAME` returns the name of a mounted volume. A UTF-8 value might appear useful as a label for an app's semantic directory

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_NAME` as `0x00002000` and `VOL_CAP_INT_VOL_RENAME` as `0x00000080` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds both constants and `fgetattrlist`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as an `attrreference` to a UTF-8, null-terminated volume name, with length no greater than `NAME_MAX + 1`. The manual marks it read/write and says it is writable only when `VOL_CAP_INT_VOL_RENAME` is set.
- The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category, so host use needs an applicable approved reason in `PrivacyInfo.xcprivacy`.

## Decision

Do not add a volume-name API. The value names a mounted volume, not Documents, Caches, Temporary, Application Support, or the app container. The adapter has no volume-rename operation, and a mounted-volume label does not inform any current file operation. Keep the filesystem name out of the `AppDirectory` contract

Revisit only if a public iOS caller has a concrete need to label the mounted volume and the host-facing string semantics are documented

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

# B407: iOS Volume Encodings-Bitmap No-Go

## Candidate

`ATTR_VOL_ENCODINGSUSED` returns a volume-wide bitmap of text encodings. It might appear to help the sandbox file API decode names that are not UTF-8

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_VOL_ENCODINGSUSED` as `0x00010000` in `sys/attr.h`.
- Locked `libc` 0.2.190 binds `ATTR_VOL_ENCODINGSUSED`; Apple's XNU manual defines its result as `unsigned long long`.
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines a bitmap of text encodings used on the volume and points to the HFS Plus Volume Format's `encodingsBitmap`. It does not associate an encoding with an individual filename or provide a per-entry conversion contract.
- `AppPath` accepts UTF-8 strings, and `read_directory` returns Rust `String` names or rejects names that are not UTF-8. The facade has no legacy-encoding input/output operation. B314 also declines the per-object `ATTR_CMN_SCRIPT` hint because no such operation exists.
- The manual requires `ATTR_VOL_INFO` with any volume attribute. The SDK declares `fgetattrlist` as available from iOS 3.0 and gives no separate availability annotation for this attribute; `ios-files` has an iOS 10.0 floor. Apple lists `fgetattrlist` in the File Timestamp required-reason API category.

## Decision

Do not add a volume encodings bitmap. It cannot decode a particular directory entry or strengthen the UTF-8 `AppPath` contract. A raw bitmap would expose legacy filesystem metadata without a supported file operation

Revisit only if the facade gains an explicit, public name-encoding conversion contract with per-entry encoding evidence

This is a report only. It adds no source API, dependency, framework, permission, usage-description key, entitlement, or deployment-floor requirement

## Validation

This audit used the installed public SDK header, locked `libc` source, and Apple's XNU manual. It ran no build, test, runtime query, consumer, live filesystem call, or probe

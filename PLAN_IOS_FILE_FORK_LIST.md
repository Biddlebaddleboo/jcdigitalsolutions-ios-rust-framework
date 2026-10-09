# B257: File Fork List No-Go

## Audit result

Do not request `ATTR_FILE_FORKLIST` in `ios-files`. Apple documents the attribute as an
`attrreference` to named forks, but explicitly states that the structure of the value is not yet
defined. The adapter cannot expose a safe stable typed list or validate a variable-length native
payload against a defined record format.

This facade has no fork enumeration, resource-fork open/read, or fork-specific mutation contract.
B242/B244/B248 expose fixed-width total/data/resource size snapshots and need no names. An opaque
unparsed byte buffer would not provide a useful caller decision and would expose an unstable native
format. The portable `framework-files::FileBackend` remains unchanged.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference describes `ATTR_FILE_FORKLIST` as an
  `attrreference` containing named forks and states that the structure of the attribute value is
  not yet defined:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK defines public `ATTR_FILE_FORKLIST` as `0x00000100` in
  `sys/attr.h`; locked `libc` 0.2.190 binds it. The SDK provides no separate availability
  annotation beyond `fgetattrlist`'s iOS 3.0 declaration. Volume support may vary.
- Any actual `fgetattrlist` use requires an applicable approved File Timestamp reason in the host
  `PrivacyInfo.xcprivacy`.

## Closure criterion

Revisit only if Apple defines a stable fork-list payload and the app-data API gains a separately
scoped fork operation that consumes the names. Do not infer resource-fork identity or access from
this undefined payload.

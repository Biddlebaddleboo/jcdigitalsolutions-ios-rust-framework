# B259: File Extent Attributes No-Go

## Audit result

Do not expose `ATTR_FILE_DATAEXTENTS` or `ATTR_FILE_RSRCEXTENTS` from `ios-files`. The iPhoneOS SDK
marks both attributes obsolete and HFS-specific. Apple's `getattrlist(2)` reference expressly says
new clients should not use `ATTR_FILE_DATAEXTENTS`; it describes the returned array as only the
first eight data-fork extents and warns that the result may not be entirely accurate. The app-data
facade has no operation that uses physical block locations or an extent map.

Adding a raw extent list would expose storage-layout details without enabling a supported sandbox
file operation, would need filesystem-specific parsing/meaning, and could imply completeness that
Apple does not promise. No portable API or source changes for this audit.

## Evidence

- Apple's archived iOS `getattrlist(2)` reference defines `ATTR_FILE_DATAEXTENTS` as an array of
  eight `diskextent` records for the first eight extents, states the value may be inaccurate, and
  says new clients should use `F_LOG2PHYS` instead; it defines `ATTR_FILE_RSRCEXTENTS` as the
  analogous resource-fork extent record array:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- The installed iPhoneOS 26.5 SDK marks `ATTR_FILE_DATAEXTENTS` (`0x00000800`) and
  `ATTR_FILE_RSRCEXTENTS` (`0x00004000`) obsolete and HFS-specific in public `sys/attr.h`.
- Locked `libc` 0.2.190 exposes `fgetattrlist`; the SDK's declaration floor is iOS 3.0, with no
  attribute-specific availability annotation. Volume support remains filesystem-dependent.

## Closure criterion

Revisit only if a supported app-data operation needs physical extent mapping and Apple provides a
current, accurate, volume-independent contract. Do not treat these HFS-specific attributes as a
complete extent map on APFS or other volume formats.

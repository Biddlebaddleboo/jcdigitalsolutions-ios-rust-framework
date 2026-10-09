# B262: File-Type Attribute No-Go

## Audit result

Do not expose `ATTR_FILE_FILETYPE` in `ios-files`. The installed public iPhoneOS SDK marks the
macro as always zero. Apple's archived iOS `getattrlist(2)` reference says its value is reserved,
clients should ignore it, and new volume-format implementations should not support the attribute.
It cannot classify the entry or inform a supported file operation.

The adapter already uses `fstatat(..., AT_SYMLINK_NOFOLLOW)` and its `FileKind` mapping for
no-follow entry classification. A permanently-zero attribute would provide no distinct contract
and could be misinterpreted as useful file type metadata.

## Evidence

- The installed iPhoneOS 26.5 SDK's public `sys/attr.h` defines `ATTR_FILE_FILETYPE` as
  `0x00000040` with the comment `always zero`.
- Apple's archived iOS `getattrlist(2)` reference says `ATTR_FILE_FILETYPE` has a reserved value,
  clients should ignore it, and new volume-format implementations should not support it:
  <https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html>
- If requested, it would use `fgetattrlist`, declared from iOS 3.0, with no separate attribute
  availability annotation; actual attribute support is filesystem-dependent. No request or runtime
  behavior is added.

## Closure criterion

No public file-type wrapper is needed unless Apple changes the attribute's documented semantics.
Continue using the existing no-follow POSIX `FileKind` classification.

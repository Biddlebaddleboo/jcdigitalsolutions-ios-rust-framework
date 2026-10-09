# B122: iOS `st_rdev` Special-Device Number No-Go

## Disposition

Do not add an `ios-files` query for `st_rdev`. The field is a device number only for a special-file inode, while the current sandbox facade classifies special entries as `Other` and does not provide a supported device-node operation

## Candidate

`st_rdev` is present in `struct stat` and could be returned for a character- or block-device entry. It is distinct from `st_dev`, which B99 already reports as the device containing an inode.

## Evidence and reason

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_rdev` as the device type for a special-file inode. It is not a general file or
  sandbox-volume identifier.
- The installed iPhoneOS 26.5 SDK declares `struct stat.st_rdev` as `dev_t`; the locked `libc`
  Apple binding exposes `stat.st_rdev` with the same target type.
- The existing `IosFiles::entry_kind` classifies special entries as `FileKind::Other`, and file
  content operations reject non-regular entries. A raw device number would add detail about an
  unsupported entry class without enabling a safe read, write, or device operation.
- The portable `FileBackend` has no device-node concept. Adding a query would not improve ordinary
  app-sandbox file behavior or change how the adapter safely rejects special entries.

## Closure criteria

Revisit only with a concrete iOS app-sandbox use for special-device entries and a supported
operation that needs the device number, not merely a future header field

## B263: `ATTR_FILE_DEVTYPE` no-go

Do not add a second special-device number query through `ATTR_FILE_DEVTYPE`. Apple's
`getattrlist(2)` reference defines it as a `u_int32_t` device type for a special-device file and
states that it is equivalent to `st_rdev`. It does not describe ordinary file kind, container
volume, or a usable app-sandbox device capability. The current facade rejects operations on
special entries, so this field would duplicate B122 without supporting a safe action.

The SDK exposes it as a public read/write attribute, but this audit proposes no getter or setter.
No special-file create/use semantics or portable device-node contract is added.

# B241: iOS Directory Mount-Status No-Go

## Candidate

`ATTR_DIR_MOUNTSTATUS` returns fixed-width flags describing what is mounted on a directory. It could appear useful for path containment or recursive traversal.

## Evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_DIR_MOUNTSTATUS` as `0x00000004` in `sys/attr.h`, with `DIR_MNTSTATUS_MNTPOINT=0x1` and `DIR_MNTSTATUS_TRIGGER=0x2`. Locked `libc` 0.2.190 binds the attribute and the mount-point bit.
- Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_DIR_MOUNTSTATUS` as a `u_int32_t` of mount-status flags and documents `DIR_MNTSTATUS_MNTPOINT` as indicating that a filesystem is mounted on the directory. The installed public header names `DIR_MNTSTATUS_TRIGGER`, but the checked manual does not define its behavior; no trigger behavior is inferred.
- The public `fgetattrlist` declaration is available from iOS 3.0; the SDK gives no separate availability annotation for this field. Any use would also require an applicable approved File Timestamp reason in the host `PrivacyInfo.xcprivacy`.
- `ios-files` does not mount filesystems or recursively traverse directory trees. Existing operations use retained app-directory roots and descriptor-relative no-follow traversal; no current operation uses a mount-status bit to decide whether it may act.

## Decision

No API or dependency change for B241. A mount-status snapshot would add no supported app-data action, and interpreting the header's trigger bit would exceed the inspected public documentation. Do not treat a mount point as an authorization result or imply that this facade can mount, unmount, or safely recurse through it.

## Closure criteria

Revisit only if this facade adds a concrete recursive traversal or mount-aware operation and Apple documents the relevant status-bit semantics and policy for iOS app-container paths.

## Validation

This report uses the installed public SDK header, locked `libc` source, and Apple's XNU source documentation. No source code, Cargo manifest or lockfile, build, test, runtime query, live filesystem call, or probe was used.

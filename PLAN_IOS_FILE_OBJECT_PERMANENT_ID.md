# B294: No-Go for a Persistent Object-ID Snapshot

## Candidate

`ATTR_CMN_OBJPERMANENTID` appears to offer a file or directory ID that remains stable across volume unmount and remount. A descriptor-bound snapshot could complement B99's `(st_dev, st_ino)` pair and B266's document ID, but it does not meet the read-only metadata contract of `ios-files`

## Evidence and blockers

- The installed iPhoneOS 26.5 SDK defines public `ATTR_CMN_OBJPERMANENTID` as `0x00000040` in `sys/attr.h`. Its `_types/_fsobj_id_t.h` defines the `fsobj_id_t` value as two `u_int32_t` fields: `fid_objno` and `fid_generation`. Locked `libc` 0.2.190 binds the attribute constant but does not bind this struct
- Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines the value as an `fsobj_id_t` that uniquely and persistently identifies an object within its volume, with persistence across mount/unmount. It also says some filesystems cannot return this value on a read-only mount and fail with `EROFS`; its example says original HFS modifies on-disk structures to generate persistent identifiers. This is not a guaranteed read-only metadata query
- The public SDK's `VOL_CAP_FMT_PERSISTENTOBJECTIDS` description says the capability marks volume formats with persistent IDs and ID lookup. The SDK also says that when `VOL_CAP_FMT_64BIT_OBJECT_IDS` is set, the `fid_objno` field in `fsobj_id_t` values from `ATTR_CMN_OBJPERMANENTID` is undefined. A valid implementation would need a volume-capability query and must reject that 64-bit-ID case
- The identifier is unique only within its volume. It is not a cross-volume identity; pairing it with a volume identifier would add another value and contract that this slice does not specify
- A capability pre-query would not remove the documented potential for on-disk metadata mutation while an ID is generated. Returning the two words as opaque values cannot make that query read-only or remove the volume-local and 64-bit-ID limits

## Decision

Do not add `ATTR_CMN_OBJPERMANENTID` to `ios-files`. Its persistent-ID semantics could help cross-mount reconciliation on some volumes, but the public contract allows filesystem metadata writes, depends on volume capabilities, and leaves a field undefined on 64-bit-ID volumes. The current facade has no consented metadata-mutation operation for ID generation. Keep B99's point-in-time inode pair and B266's document-ID contract distinct; neither is promoted to a persistent object identity by this audit

`ATTR_CMN_PAROBJID` is not a substitute: Apple's XNU manual says its parent ID lasts only for the mount, can be nondeterministic for hard-linked objects, and can be expensive to compute

## Scope and validation

This report adds no source/API, dependency, permission, deployment-floor claim, capability-matrix status, or aggregate count. It is a header/source-documentation audit only; do not run tests, consumers, probes, or live filesystem calls

Documentation gates: `target/debug/xtask docs-check`, `git diff --check`, and a trailing-whitespace scan for this plan

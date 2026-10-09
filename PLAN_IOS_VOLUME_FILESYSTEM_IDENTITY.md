# B158: No-Go for iOS Volume Filesystem Identity Metadata

## Disposition

Do not add an `ios-files` API for a volume filesystem identifier, owner, numeric type, subtype, or raw filesystem name based on `fstatfs`. Apple documents these as low-level mount metadata, but does not define a stable app-level filesystem identity or a useful iOS capability contract for them. The values would expose implementation detail without enabling a reliable app-data decision

## Candidate fields and evidence

- `f_fsid` is documented as a file system ID. The API documentation does not promise persistence across remount, reboot, filesystem migration, or OS update, nor define it as an app-container identifier. Treating it as a persistent volume key would exceed the documented contract.
- `f_owner` is the user that mounted the filesystem. It does not describe the app's effective sandbox permissions or the owner of a particular file.
- `f_type`, `f_fssubtype`, and `f_fstypename` describe filesystem implementation/type data. The public documentation does not provide a stable iOS mapping suitable for feature selection; a raw number or name would not establish behavior or supported filesystem capabilities.
- The same fields are available from `fstatfs` on a retained semantic root, but the query method does not make the values more stable or app-specific.

## Decision

Keep filesystem identity/type fields out of the public facade. Do not use them as stable cache keys, persistence identifiers, feature gates, or app-container identification. If a future product need arises, it must identify a user-facing decision and a documented stable API contract first. Existing supported volume values remain limited to the explicit capacity, read-only mount, and optimal-I/O-size snapshots; the separate B140/B143/B149 no-go reports cover other unsupported volume statistics.

## Platform evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) defines `f_fsid` as “file system id,” `f_owner` as the user that mounted the filesystem, and `f_type`, `f_fssubtype`, and `f_fstypename` as filesystem type/subtype/name fields. It does not document stable persistence or an iOS feature-support mapping for these fields.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` exposes the corresponding fields in `struct statfs`; their presence in the SDK establishes API shape, not persistence or caller utility.
- This is a no-go report only. It adds no source/API, framework, dependency, permission, privacy-manifest reason, deployment-floor claim, or capability-matrix status change.

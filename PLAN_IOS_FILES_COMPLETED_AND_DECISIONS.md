# PLAN_IOS_FILES_COMPLETED_AND_DECISIONS.md — iOS file slices closed at 36e3090a79a2

## Purpose and evidence limits

This is a *historical closure index*, not a new coding workstream. It replaces the 22 obsolete individual planning documents listed below. Their complete original text remains retrievable at Git commit `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247`; current code and `docs/ios/files.md` are the canonical implementation contracts. Continue all other iOS file work in the surviving plans until independently audited. Do not interpret compile/link validation as a successful real-device filesystem operation. Xcode 26.6/SDK 26.5 observations do not establish Xcode 27 compatibility.

## Implemented bounded API slices (closed as narrow tasks, not complete filesystem support)

- `PLAN_IOS_FILE_ADDED_TIME.md` → `IosFiles::entry_added_time — added-to-directory timestamp`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_ADDED_TIME.md`.
- `PLAN_IOS_FILE_CLONE_ID.md` → `IosFiles::regular_file_clone_id_snapshot — opaque clone identifier`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_CLONE_ID.md`.
- `PLAN_IOS_FILE_DATA_GENERATION.md` → `IosFiles::regular_file_data_generation_snapshot — identity + optional generation`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_DATA_GENERATION.md`.
- `PLAN_IOS_DIRECTORY_EMPTY_CHECK.md` → `IosFiles::directory_is_empty — early-stop direct-entry check`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_DIRECTORY_EMPTY_CHECK.md`.
- `PLAN_IOS_DIRECTORY_ENTRY_COUNT.md` → `IosFiles::directory_entry_count — direct names only`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_DIRECTORY_ENTRY_COUNT.md`.
- `PLAN_IOS_VOLUME_USED_CAPACITY.md` → `IosFiles::volume_used_capacity_bytes — filesystem-reported capacity`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_VOLUME_USED_CAPACITY.md`.
- `PLAN_IOS_APP_DIRECTORY_CASE_SENSITIVITY.md` → `IosFiles::app_directory_case_sensitivity — directory-root filesystem case behavior`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_APP_DIRECTORY_CASE_SENSITIVITY.md`.
- `PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md` → `IosFiles::entry_effective_access — point-in-time effective access mask`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md`.
- `PLAN_IOS_ENTRY_OBJECT_KIND.md` → `IosFiles::entry_object_kind — no-follow native file-kind classification`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_ENTRY_OBJECT_KIND.md`.
- `PLAN_IOS_FILE_TOTAL_FORK_SIZE.md` → `IosFiles::regular_file_total_fork_size_snapshot — combined fork bytes`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_TOTAL_FORK_SIZE.md`.
- `PLAN_IOS_FILE_RESOURCE_FORK_SIZE.md` → `IosFiles::regular_file_resource_fork_size_snapshot — resource fork logical bytes`. Verified a public function declaration in `platform/ios/ios-files/src/lib.rs` and corresponding description in `docs/ios/files.md`; original evidence and constraints at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_RESOURCE_FORK_SIZE.md`.

Common invariants retained: sandbox-root-scoped `AppPath`, descriptor-relative traversal, no-follow semantics where appropriate, checked native structure/length conversion, no unexpected content reads or permission prompts, point-in-time values rather than guaranteed durable identity/state. These remain native iOS extensions and must not silently extend portable `FileBackend`. Original plans include compile/link checks, not representative physical-device execution.

## Completed no-go / duplicate-API assessments

- `PLAN_IOS_FILE_EXTENTS.md`: Obsolete/HFS-specific data/resource extent attributes give incomplete and potentially inaccurate physical maps; no facade consumer. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_EXTENTS.md`.
- `PLAN_IOS_FILE_FORK_LIST.md`: ATTR_FILE_FORKLIST payload format is not publicly defined; no stable typed contract or fork enumeration consumer. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_FORK_LIST.md`.
- `PLAN_IOS_FILE_OBJECT_PERMANENT_ID.md`: No justified safe persistent identity contract; retain bounded (device,inode) near-time semantics. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_OBJECT_PERMANENT_ID.md`.
- `PLAN_IOS_VOLUME_UUID.md`: No consumer needs volume UUID; neither persistence nor app-container identity is guaranteed. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_VOLUME_UUID.md`.
- `PLAN_IOS_VOLUME_NAME.md`: Volume name is mutable and unrelated to a safe application-directory operation. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_VOLUME_NAME.md`.
- `PLAN_IOS_VOLUME_CAPABILITIES.md`: Raw capability bitfield insufficient for an app-facing preflight, valid-mask and policy semantics; avoid generic snapshot. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_VOLUME_CAPABILITIES.md`.
- `PLAN_IOS_VOLUME_QUOTA_SIZE.md`: Volume maximum is not a reliable sandbox/app quota. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_VOLUME_QUOTA_SIZE.md`.
- `PLAN_IOS_VOLUME_MOUNT_POINT.md`: Native mount path leaks irrelevant absolute-root detail and does not improve descriptor-relative sandbox access. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_VOLUME_MOUNT_POINT.md`.
- `PLAN_IOS_FILE_CREATION_TIME.md`: Darwin st_birthtime may substitute ctime without an availability bit; never expose as guaranteed creation time. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_CREATION_TIME.md`.
- `PLAN_IOS_FILE_PROTECTION_CLASS.md`: No public stable mapping from raw F_GETPROTECTIONCLASS value to named Foundation protection class. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_PROTECTION_CLASS.md`.
- `PLAN_IOS_FILE_FORK_COUNT.md`: Fork count alone cannot enumerate/read forks and is not an actionable app-data capability. Original bounded SDK evidence and rationale at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_FILE_FORK_COUNT.md`.

These are completed *research decisions*—not evidence that the underlying Apple API cannot ever be used. Do not reintroduce a rejected property absent a new concrete consumer, verified public contract and revised safety/availability assessment.

## Residual work and owners

- Retain all nonlisted `PLAN_IOS_*.md` until their implementation, test, blocker and link status is audited. `PLAN_IOS_APP_DATA.md` remains the owner for unclosed file semantics and test gaps.
- The current capability matrix remains the source of truth: `docs/capabilities/capability-status.json`. A completed property query does **not** complete its enclosing capability row.
- Any future user-facing operations must preserve sandbox root ownership, thread-safety, error mapping, symlink policy and forward compatibility, and add deterministic regression tests with device testing when material.
- Before implementation: read `PLAN.md`, then the residual workstream; consult this closure index only for a previously decided question, never all historical files.

## Additional retired filesystem slices (audit on e11b2fca5051)

These 14 bounded plans have been retired. Each full contract and evidence record is recoverable from the linked immutable original, with live API behavior described in `platform/ios/ios-files/src/lib.rs` and `docs/ios/files.md`. This is **not** a device-runtime or entire-capability completion claim.

### Implemented narrow APIs

- [PLAN_IOS_DIRECTORY_ALLOCATED_SIZE.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_DIRECTORY_ALLOCATED_SIZE.md) — IosFiles::directory_allocated_size_snapshot: no-follow directory-object allocation snapshot.
- [PLAN_IOS_DIRECTORY_KIND_COUNTS.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_DIRECTORY_KIND_COUNTS.md) — IosFiles::directory_entry_kind_counts: direct child type tallies; no content reads.
- [PLAN_IOS_ENTRY_KIND.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_ENTRY_KIND.md) — IosFiles::entry_kind: single-entry no-follow kind.
- [PLAN_IOS_FILE_ACCESS_TIME.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_FILE_ACCESS_TIME.md) — IosFiles::entry_access_time: not an access log or change token.
- [PLAN_IOS_FILE_ALLOCATED_BLOCKS.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_FILE_ALLOCATED_BLOCKS.md) — IosFiles::regular_file_allocated_blocks_512: 512-byte block count, not exact disk use.
- [PLAN_IOS_FILE_BACKUP_TIME_MARKER.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_FILE_BACKUP_TIME_MARKER.md) — IosFiles::entry_stored_backup_time: stored filesystem marker, not actual backup proof.
- [PLAN_IOS_APP_DIRECTORY_NAME_TRUNCATION.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_APP_DIRECTORY_NAME_TRUNCATION.md) — IosFiles::app_directory_truncates_long_names: Darwin _PC_NO_TRUNC true-means-truncation handling.

### Closed no-go decisions

- [PLAN_IOS_APP_DIRECTORY_CASE_PRESERVING.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_APP_DIRECTORY_CASE_PRESERVING.md) — Duplicate case-preserved-name volume Boolean already supplied by B175; no separate semantics..
- [PLAN_IOS_BACKUP_EXCLUSION.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_BACKUP_EXCLUSION.md) — Do not infer safe backup-exclusion setter semantics solely from a sandbox AppPath; preserve host/file-provider boundary..
- [PLAN_IOS_DIRECTORY_DATA_LENGTH.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_DIRECTORY_DATA_LENGTH.md) — No supported caller contract for directory logical data length..
- [PLAN_IOS_DIRECTORY_IO_BLOCK_SIZE.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_DIRECTORY_IO_BLOCK_SIZE.md) — No useful supported operation requires directory raw I/O block-size metadata..
- [PLAN_IOS_DIRECTORY_LINK_COUNT.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_DIRECTORY_LINK_COUNT.md) — Directory link count is not a direct child count or portable operation..
- [PLAN_IOS_DIRECTORY_MOUNT_STATUS.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_DIRECTORY_MOUNT_STATUS.md) — Raw mount-status flags are not a safe path-containment or traversal guarantee..
- [PLAN_IOS_FILE_ATTRIBUTION_TAG.md](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/blob/e11b2fca50513d24912b3b74bbf79e4315b75bbf/PLAN_IOS_FILE_ATTRIBUTION_TAG.md) — Opaque attribution tag has no justified sandbox-file caller contract..

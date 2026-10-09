# PLAN_IOS_APP_DATA.md — Workstream B1: iOS Files and Preferences

## Status

B1's file and preferences backends are in the tree. Both library roots are gated by
`#![cfg(target_os = "ios")]`; `IosFiles::new` resolves Foundation app-sandbox URLs, while
`IosPreferences::new` accesses `NSUserDefaults.standardUserDefaults`. Host-runnable tests reuse the
production `path_parts` and preference write-policy helpers. They do not instantiate the gated
backends or exercise descriptor-relative I/O, symlinks, atomic rename, live sandbox roots,
`NSUserDefaults`, or persistence. The native path gate rejects Windows drive prefixes.

Host app-data tests passed: `cargo test --locked --offline -p ios-files -p ios-preferences` (four
integration tests). Portable contract tests passed: `cargo test --locked --offline -p
framework-files -p framework-preferences` (eight tests). Device and simulator checks passed:
`cargo check --locked --offline -p ios-files -p ios-preferences --target aarch64-apple-ios` and
`cargo check --locked --offline -p ios-files -p ios-preferences --target aarch64-apple-ios-sim`.
Strict Clippy passed:
`cargo clippy --locked --offline --all-targets -p ios-files -p ios-preferences --target aarch64-apple-ios -- -D warnings`
and
`cargo clippy --locked --offline --all-targets -p ios-files -p ios-preferences --target aarch64-apple-ios-sim -- -D warnings`.
`cargo fmt --package ios-files --package ios-preferences -- --check` and `git diff --check` passed.

An isolated minimal consumer source and manifest were generated under ignored
`target/b1-link-probe`. Device and simulator builds passed with
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b1-link-probe/Cargo.toml --release --target aarch64-apple-ios` and
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b1-link-probe/Cargo.toml --release --target aarch64-apple-ios-sim`.
`otool -L target/b1-link-probe/target/aarch64-apple-ios/release/b1-link-probe` and
`otool -L target/b1-link-probe/target/aarch64-apple-ios-sim/release/b1-link-probe` showed
Foundation, CoreFoundation, `libobjc.A`, `libSystem.B`, and `libiconv.2`; neither imports UIKit,
Network, Swift, Python, CoreLocation, UserNotifications, or Security. Device load metadata records
minos 10.0; simulator metadata records minos 14.0. The focused gate
`sh platform/ios/ios-files/check-app-data-link-imports.sh` now builds a link-only example for each
crate and target, checks exact imports, rejects unrelated framework symbols, and checks device
minos 10.0 and simulator minos 14.0. CI runs this gate after both crates' target checks and strict
all-target Clippy. The probe executables were inspected but not run. Checks ran with Xcode 26.6 and
iPhoneOS/iPhoneSimulator SDK 26.5, below the required Xcode 27.x baseline.

The base `Files` facade does not accept user-selected, security-scoped, iCloud, or provider URLs. The
separate `IosFileCoordinator` can coordinate caller-supplied URLs synchronously, but does not manage
security scope, prove sandbox containment, or provide file-provider lifecycle support. The separate
`IosSecurityScopedAccess` guard balances one already-issued scope and does not change the base
facade; see [PLAN_IOS_SECURITY_SCOPED_ACCESS.md](PLAN_IOS_SECURITY_SCOPED_ACCESS.md). Preference
durability remains unspecified; `NSUserDefaults` writes are visible in-process before asynchronous
persistence.
Descriptor-relative no-follow traversal does not serialize concurrent native directory renames;
another handle can rename an opened directory inode outside the selected root while its descriptor
continues to refer to that inode, so selected-root containment is not guaranteed under that race.
B14 adds `IosFiles::adopt_url_session_download` as a row-010 sandbox-file operation, distinct from
B17 file coordination; it synchronously adopts only the URLSession callback temporary file into an
`AppPath`. Its copy, commit, and inherited directory-rename limits are recorded in
`PLAN_IOS_FILE_ADOPTION.md`.

The B83 app-data follow-up `IosResolvedBookmark::resolve_unscoped` maps caller-owned, non-security-scoped
Foundation bookmark data to a file URL and stale bit. The method is unsafe because its input scope
cannot be verified; `NSURLBookmarkResolutionWithoutImplicitStartAccessing` does not govern
security-scoped bookmark data. It needs iOS 14.2 and does not create bookmarks, prompt, grant access,
coordinate I/O, or extend sandbox containment. See
[`PLAN_IOS_BOOKMARK_RESOLUTION.md`](PLAN_IOS_BOOKMARK_RESOLUTION.md). B83 is distinct from B82,
which adds only a registered-domain count for the calling app's own FileProvider extension; see the
[B82 FileProvider count plan](PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md).

B85 adds `IosPlainBookmarkData::create` for a caller-owned file URL. It uses
`NSURLBookmarkCreationWithoutImplicitSecurityScope`, returns an opaque typed value, and offers safe
resolution only through that value; it does not grant access or add a URL to the sandbox `Files`
facade. Arbitrary stored `NSData` still uses B83's unsafe raw resolver contract. See
[`PLAN_IOS_BOOKMARK_CREATION.md`](PLAN_IOS_BOOKMARK_CREATION.md).

B90 adds `IosFiles::regular_file_size` as an iOS-only, metadata-only snapshot under an existing
`AppPath`. It uses the same validated path and no-follow parent traversal as `FileBackend`, then
one final `fstatat` with `AT_SYMLINK_NOFOLLOW`; it rejects non-regular entries and returns only a
`u64` byte length. It reads no file contents and changes no security-scope or sandbox-root
semantics. See [`PLAN_IOS_FILE_SIZE.md`](PLAN_IOS_FILE_SIZE.md).

B93 adds `IosFiles::entry_kind` as an iOS-only point-in-time classification for one validated
`AppPath`. It reuses the `FileKind` values and no-follow `fstatat` helper used by `read_directory`;
it reads no file contents, follows no final symlink, and changes no portable facade or access
semantics. See [`PLAN_IOS_ENTRY_KIND.md`](PLAN_IOS_ENTRY_KIND.md).

B96 adds `IosFiles::entry_modification_time` as an iOS-only raw POSIX seconds/nanoseconds snapshot
for one validated `AppPath`. It uses no-follow `fstatat`, reports a symlink's own timestamp, and
does not claim a content version or reliable change token. It changes no portable facade or access
semantics. See [`PLAN_IOS_FILE_MODIFICATION_TIME.md`](PLAN_IOS_FILE_MODIFICATION_TIME.md).

B101 audits an entry creation-time snapshot and records a no-go: on some filesystems `st_birthtime` contains `ctime`, and `stat` has no per-entry support signal. See [`PLAN_IOS_FILE_CREATION_TIME.md`](PLAN_IOS_FILE_CREATION_TIME.md).

B99 adds `IosFiles::entry_identity_snapshot` as an iOS-only `(st_dev, st_ino)` snapshot for one
validated `AppPath`. It reports the final entry itself without following symlinks, reads no
contents, and is explicitly not persistent or protected from inode reuse. See
[`PLAN_IOS_FILE_IDENTITY.md`](PLAN_IOS_FILE_IDENTITY.md).

B103 adds `IosFiles::entry_posix_permission_bits` as an iOS-only raw `st_mode & 0o7777` snapshot.
It reports stored mode bits without following a final symlink and makes no effective-access or
future-operation claim. See [`PLAN_IOS_FILE_PERMISSION_BITS.md`](PLAN_IOS_FILE_PERMISSION_BITS.md).

B105 adds `IosFiles::regular_file_hard_link_count` for regular files only. It reports `st_nlink`,
not alias paths or exclusive ownership; non-regular entries return `InvalidInput`. See
[`PLAN_IOS_FILE_LINK_COUNT.md`](PLAN_IOS_FILE_LINK_COUNT.md).

B107 adds `IosFiles::entry_status_change_time` as a no-follow POSIX status-change time snapshot.
It rejects final symlinks and is not a content version or reliable change token. See
[`PLAN_IOS_FILE_STATUS_CHANGE_TIME.md`](PLAN_IOS_FILE_STATUS_CHANGE_TIME.md).

B109 adds `IosFiles::regular_file_allocated_blocks_512` for regular files only. It returns raw
`st_blocks` units of 512 bytes and does not claim exact physical storage use or exclusive
allocation. See [`PLAN_IOS_FILE_ALLOCATED_BLOCKS.md`](PLAN_IOS_FILE_ALLOCATED_BLOCKS.md).

B112 adds `IosFiles::entry_access_time` as a no-follow POSIX access-time snapshot. It rejects final
symlinks; `st_atime` may be set and a read need not update it on every filesystem, so this is not a
guaranteed access log or reliable change token. See
[`PLAN_IOS_FILE_ACCESS_TIME.md`](PLAN_IOS_FILE_ACCESS_TIME.md).

B115 adds `IosFiles::entry_bsd_file_flags` with raw `st_flags` bits and named `UF_*`/`SF_*`
mask accessors. It preserves unknown bits, rejects final symlinks, and does not determine effective
access. See [`PLAN_IOS_BSD_FILE_FLAGS.md`](PLAN_IOS_BSD_FILE_FLAGS.md).

B118 adds `IosFiles::entry_owner_ids` with raw `u32` `st_uid`/`st_gid` values from one no-follow
lookup. It rejects final symlinks and does not map IDs to account names, membership, stable identity,
or effective access. See [`PLAN_IOS_FILE_OWNER_IDS.md`](PLAN_IOS_FILE_OWNER_IDS.md).

B121 audits `st_blksize` and records a no-go: it is an optimal I/O-size hint, but this facade has
no streaming descriptor, caller buffer, or I/O policy hook to use it. Publishing the value alone
risks a false size/performance claim. See [`PLAN_IOS_FILE_IO_BLOCK_SIZE.md`](PLAN_IOS_FILE_IO_BLOCK_SIZE.md).

B122 audits `st_rdev` and records a no-go: it describes special-file device metadata, but the
facade cannot read or write these `FileKind::Other` entries, so the number enables no supported
operation. See [`PLAN_IOS_FILE_SPECIAL_DEVICE_NUMBER.md`](PLAN_IOS_FILE_SPECIAL_DEVICE_NUMBER.md).

B125 audits `st_gen` and records a no-go: Apple documents it as superuser-only; a local SDK or
binding field does not establish app-sandbox access or persistent identity semantics. See
[`PLAN_IOS_FILE_GENERATION_NUMBER.md`](PLAN_IOS_FILE_GENERATION_NUMBER.md).

B128 adds `IosFiles::directory_entry_count` for direct names other than `.` and `..`, including all
entry kinds and non-UTF-8 names, without a Rust-owned listing or per-entry metadata lookup. It is
synchronous and O(n); libc may allocate a directory stream, and concurrent changes can affect the
count. It is not an atomic snapshot, reservation, or delete guard. See
[`PLAN_IOS_DIRECTORY_ENTRY_COUNT.md`](PLAN_IOS_DIRECTORY_ENTRY_COUNT.md).

B131 adds `IosFiles::directory_is_empty` for a best-effort early-exit check of direct names other than `.` and `..`. It may stop after the first child, but concurrent changes can affect the result; `true` is not a reservation or proof that a later removal will succeed. `remove_directory` remains authoritative. See [`PLAN_IOS_DIRECTORY_EMPTY_CHECK.md`](PLAN_IOS_DIRECTORY_EMPTY_CHECK.md).

B134 adds `IosFiles::directory_entry_kind_counts` and fixed-width `files()`, `directories()`, and `other()` totals using existing no-follow `FileKind` classification. It counts non-UTF-8 names but does not copy names or build a listing; if a name vanishes during metadata lookup, it returns an error rather than partial totals. It is not an atomic snapshot or deletion guard. See [`PLAN_IOS_DIRECTORY_KIND_COUNTS.md`](PLAN_IOS_DIRECTORY_KIND_COUNTS.md).

B235 adds `IosFiles::directory_allocated_size_snapshot(&self, path: AppPath<'_>) -> Result<IosDirectoryAllocatedSizeSnapshot, FileError>` using descriptor-bound `fgetattrlist(ATTR_DIR_ALLOCSIZE)`. The value is bytes used by the directory object itself, not a recursive child-content total or app quota. The host must declare an applicable approved File Timestamp required-reason API entry for actual use. See [`PLAN_IOS_DIRECTORY_ALLOCATED_SIZE.md`](PLAN_IOS_DIRECTORY_ALLOCATED_SIZE.md).

B237 declines `ATTR_DIR_DATALENGTH`: XNU defines it as the directory object's logical byte length, not a total of child file data. Exposing another internal directory-structure size would add no supported app-data operation and could be mistaken for folder contents. See [`PLAN_IOS_DIRECTORY_DATA_LENGTH.md`](PLAN_IOS_DIRECTORY_DATA_LENGTH.md).

B238 declines `ATTR_DIR_IOBLOCKSIZE`: XNU defines it only as an optimal read/write block-size hint for the directory object, but this facade exposes no raw directory-data I/O whose buffer could use the value. It is not an alignment rule or performance promise. See [`PLAN_IOS_DIRECTORY_IO_BLOCK_SIZE.md`](PLAN_IOS_DIRECTORY_IO_BLOCK_SIZE.md).

B240 declines `ATTR_DIR_LINKCOUNT` as a child-directory count: XNU defines hard links to the directory, excludes `.` and `..`, and returns `1` on filesystems that do not support directory hard links. B134's direct entry scan remains the supported subdirectory count. See [`PLAN_IOS_DIRECTORY_LINK_COUNT.md`](PLAN_IOS_DIRECTORY_LINK_COUNT.md).

B241 declines `ATTR_DIR_MOUNTSTATUS`: it reports mount-point/trigger flags, but the facade has no mount or recursive-traversal policy operation that can act on them; the public XNU manual documents only the mount-point bit. See [`PLAN_IOS_DIRECTORY_MOUNT_STATUS.md`](PLAN_IOS_DIRECTORY_MOUNT_STATUS.md).

B242 adds `IosFiles::regular_file_total_fork_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileTotalForkSizeSnapshot, FileError>` using descriptor-bound `fgetattrlist(ATTR_FILE_TOTALSIZE)`. XNU defines the result as logical bytes across all forks; it may differ from the data-fork size and is not a buffer-read size. It reads no content and does not expose fork APIs. The host must declare an applicable approved File Timestamp required-reason API entry for actual use. See [`PLAN_IOS_FILE_TOTAL_FORK_SIZE.md`](PLAN_IOS_FILE_TOTAL_FORK_SIZE.md).

B244 adds `IosFiles::regular_file_data_fork_allocated_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileDataForkAllocatedSizeSnapshot, FileError>` using descriptor-bound `fgetattrlist(ATTR_FILE_DATAALLOCSIZE)`. XNU defines the value as bytes on disk used by the data fork only, excluding resource-fork allocation. This is distinct from B109's raw per-entry `st_blocks` count in 512-byte units; no fixed conversion or exclusive physical-device allocation is promised. The host must declare an applicable approved File Timestamp required-reason API entry for actual use. See [`PLAN_IOS_FILE_DATA_FORK_ALLOCATED_SIZE.md`](PLAN_IOS_FILE_DATA_FORK_ALLOCATED_SIZE.md).

B248 adds `IosFiles::regular_file_resource_fork_allocated_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileResourceForkAllocatedSizeSnapshot, FileError>` using descriptor-bound `fgetattrlist(ATTR_FILE_RSRCALLOCSIZE)`. XNU defines the value as bytes on disk used by the resource fork only; the operation does not enumerate, read, or grant access to fork contents, and zero does not prove fork absence. Filesystem support varies. The host must declare an applicable approved File Timestamp required-reason API entry for actual use. See [`PLAN_IOS_FILE_RESOURCE_FORK_ALLOCATED_SIZE.md`](PLAN_IOS_FILE_RESOURCE_FORK_ALLOCATED_SIZE.md).

B251 adds `IosFiles::regular_file_resource_fork_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFileResourceForkSizeSnapshot, FileError>` using descriptor-bound `fgetattrlist(ATTR_FILE_RSRCLENGTH)`. XNU defines the value as the resource fork's logical byte length only; it does not expose or read fork data, and zero does not prove that a fork is absent. Filesystem support varies. The host must declare an applicable approved File Timestamp required-reason API entry for actual use. See [`PLAN_IOS_FILE_RESOURCE_FORK_SIZE.md`](PLAN_IOS_FILE_RESOURCE_FORK_SIZE.md).

B253 declines `ATTR_FILE_ALLOCSIZE` as a second all-fork allocation total: it reports bytes on disk across all forks, while B109 already exposes the per-entry `st_blocks` count and B244/B248 expose the data/resource fork allocation split. The facade has no operation that needs another aggregate allocation encoding, and no fixed conversion between the count and byte value is assumed. See [`PLAN_IOS_FILE_TOTAL_ALLOCATED_SIZE.md`](PLAN_IOS_FILE_TOTAL_ALLOCATED_SIZE.md).

B257 declines `ATTR_FILE_FORKLIST`: Apple documents its value as an `attrreference` to a fork-name list but says the value's structure is not defined. Returning or parsing that payload would create an unstable contract, and this facade has no fork enumeration/access operation. See [`PLAN_IOS_FILE_FORK_LIST.md`](PLAN_IOS_FILE_FORK_LIST.md).

B259 declines `ATTR_FILE_DATAEXTENTS` and `ATTR_FILE_RSRCEXTENTS`: the public SDK marks both obsolete and HFS-specific, and the app-data facade has no operation that consumes physical extent locations. Apple specifically tells new clients not to use the data-extents attribute. See [`PLAN_IOS_FILE_EXTENTS.md`](PLAN_IOS_FILE_EXTENTS.md).

B262 declines `ATTR_FILE_FILETYPE`: the public SDK marks it always zero, and Apple's reference says its value is reserved and clients should ignore it. See [`PLAN_IOS_FILE_TYPE.md`](PLAN_IOS_FILE_TYPE.md).

B263 declines `ATTR_FILE_DEVTYPE` as a second `st_rdev` query: it is an equivalent device-type value for special files, which the facade does not operate on. See [`PLAN_IOS_FILE_SPECIAL_DEVICE_NUMBER.md`](PLAN_IOS_FILE_SPECIAL_DEVICE_NUMBER.md).

B264 declines `ATTR_FILE_LINKCOUNT` because Apple defines it as equivalent to `st_nlink`; B105 already reports the hard-link count for one regular file. See [`PLAN_IOS_FILE_LINK_COUNT.md`](PLAN_IOS_FILE_LINK_COUNT.md).

B265 declines `ATTR_FILE_DATALENGTH` as redundant with B90's `st_size`-based data-stream length; B242 and B251 already provide the distinct all-fork total and resource-fork length. See [`PLAN_IOS_FILE_SIZE.md`](PLAN_IOS_FILE_SIZE.md).

B266 adds `IosFiles::regular_file_document_id_snapshot(&self, path: AppPath<'_>) -> Result<IosFileDocumentIdSnapshot, FileError>` through `fgetattrlist`, with `ATTR_CMN_DOCUMENT_ID` requested as an extended common attribute via `FSOPT_ATTR_CMN_EXTENDED`. `IosFileDocumentIdSnapshot::document_id()` maps XNU's invalid zero to `None`. The value tracks a document across moves and stays sticky to its assigned path across safe saves per XNU, but the API makes no inode, content-hash, clone-ID, link-ID, cross-volume, or durable-identity claim. Unsupported or omitted values map to `Unsupported`; the host must declare an applicable approved File Timestamp required-reason entry for actual use. See [`PLAN_IOS_FILE_DOCUMENT_ID.md`](PLAN_IOS_FILE_DOCUMENT_ID.md).

B272 adds `IosFiles::entry_added_time(&self, path: AppPath<'_>) -> Result<IosFileAddedTime, FileError>` via descriptor-bound `fgetattrlist(ATTR_CMN_ADDEDTIME)` for regular files. XNU defines this as the time an object was created or renamed into its containing directory and warns that values may be inconsistent for hard-linked items; it is not a reliable creation-time or path-history record. The B101 `st_birthtime` no-go remains distinct. The host must declare an applicable approved File Timestamp required-reason entry for actual use. See [`PLAN_IOS_FILE_ADDED_TIME.md`](PLAN_IOS_FILE_ADDED_TIME.md).

B275 adds `IosFiles::regular_file_data_generation_snapshot(&self, path: AppPath<'_>) -> Result<IosFileDataGenerationSnapshot, FileError>`. The device/inode identity pair comes from `fstat` on the opened regular-file descriptor; `ATTR_CMN_GEN_COUNT` comes from a separate `fgetattrlist` call on that same descriptor. This avoids path retargeting but does not make the two values an atomic cross-field snapshot. Zero maps to `None`, including XNU's memory-mapped-file case. XNU's equality rule applies only to the same filesystem object, so the value is not a general content-change token or persistent identity. The host must declare an applicable approved File Timestamp required-reason entry for actual use. See [`PLAN_IOS_FILE_DATA_GENERATION.md`](PLAN_IOS_FILE_DATA_GENERATION.md).

B256 declines `ATTR_FILE_IOBLOCKSIZE`: it is a per-file optimal read/write block-size hint, but the facade has no stream handle, caller buffer, or policy hook that consumes it. It is not an alignment or performance guarantee; B121 records the same no-go for `st_blksize`. See [`PLAN_IOS_FILE_IO_BLOCK_SIZE.md`](PLAN_IOS_FILE_IO_BLOCK_SIZE.md).

B260 declines obsolete `ATTR_FILE_CLUMPSIZE`: its data-fork allocation-clump hint is neither required nor guaranteed, and the facade has no allocation or streaming-write policy operation that consumes it. See [`PLAN_IOS_FILE_IO_BLOCK_SIZE.md`](PLAN_IOS_FILE_IO_BLOCK_SIZE.md).

B247 declines `ATTR_FILE_FORKCOUNT`: it reports only the number of forks, not fork names, per-fork sizes, or data access, and this adapter has no fork-enumeration or fork-operation contract that consumes the count. B242/B244/B248 already report all-fork logical, data-fork allocated, and resource-fork allocated byte snapshots without introducing fork handles. See [`PLAN_IOS_FILE_FORK_COUNT.md`](PLAN_IOS_FILE_FORK_COUNT.md).

B137 adds `IosFiles::volume_available_capacity_bytes(AppDirectory)` using checked `f_bavail * f_bsize` from `fstatfs` on the retained root descriptor. This is volume-wide free space available to non-superusers, not an app quota, reservation, or write guarantee. Apple requires an approved `NSPrivacyAccessedAPICategoryDiskSpace` reason in the final `PrivacyInfo.xcprivacy` that matches actual host use; the library does not select a reason or modify a root manifest. See [`PLAN_IOS_VOLUME_CAPACITY.md`](PLAN_IOS_VOLUME_CAPACITY.md).

B146 adds `IosFiles::volume_total_capacity_bytes(AppDirectory)` using checked `f_blocks * f_bsize` from `fstatfs` on the retained semantic-root descriptor. It reports mounted-volume capacity only, not physical-device capacity or an app quota. As with B137, the final host `PrivacyInfo.xcprivacy` must declare a Disk Space reason matching actual use. See [`PLAN_IOS_VOLUME_TOTAL_CAPACITY.md`](PLAN_IOS_VOLUME_TOTAL_CAPACITY.md).

B140 audits `NSURLDirectoryEntryCountKey` and `ATTR_DIR_ENTRYCOUNT` as a possible cheap-count path and records a no-go: the key is optional, `getattrlist(2)` does not guarantee low cost, the count is 32-bit, and not every volume format supports it. B128/B131 already provide deterministic count/empty-query behavior. See [`PLAN_IOS_FAST_DIRECTORY_COUNT.md`](PLAN_IOS_FAST_DIRECTORY_COUNT.md). B143 audits `f_bfree` and records a no-go: it includes reserved filesystem blocks, which the sandbox cannot consume; B137 already exposes the useful `f_bavail` non-superuser value. See [`PLAN_IOS_VOLUME_FREE_BLOCKS.md`](PLAN_IOS_VOLUME_FREE_BLOCKS.md).

B149 audits `f_files`/`f_ffree` and records a no-go: both are volume-wide node counts, not an app/container budget, and `f_ffree` does not establish whether a later sandbox create will succeed. Actual creation remains authoritative. See [`PLAN_IOS_VOLUME_FILE_NODE_COUNTS.md`](PLAN_IOS_VOLUME_FILE_NODE_COUNTS.md).
B152 adds `IosFiles::volume_is_read_only(AppDirectory)` using `fstatfs` on the retained semantic root. It reports only `MNT_RDONLY`; `false` does not establish effective path or app write access. B155 adds `IosFiles::volume_optimal_io_block_size_bytes(AppDirectory)` using `fstatfs.f_iosize`, a filesystem sizing hint only—not an alignment mandate or performance guarantee. B155 is a distinct volume-level field from B121’s per-entry `st_blksize` audit. Both query Disk Space required-reason APIs, so host `PrivacyInfo.xcprivacy` must use an approved reason matching actual use. See [`PLAN_IOS_VOLUME_READ_ONLY.md`](PLAN_IOS_VOLUME_READ_ONLY.md) and [`PLAN_IOS_VOLUME_IO_SIZE.md`](PLAN_IOS_VOLUME_IO_SIZE.md). B158 declines `f_fsid`, `f_owner`, `f_type`, `f_fssubtype`, and `f_fstypename` as app-facing volume identity because they do not define a persistent/container identity or stable iOS filesystem mapping. B161 declines `MNT_NOEXEC`, `MNT_NOSUID`, `MNT_NODEV`, `MNT_QUOTA`, and `MNT_DONTBROWSE` as effective path/app authorization signals; B152 exposes only the read-only mount bit. B164 adds `IosFiles::app_directory_name_max_bytes(AppDirectory)` using `fpathconf(_PC_NAME_MAX)` on the retained root. `Some(n)` is a direct-child component limit in bytes; `None` is the POSIX no-limit sentinel, and neither total/nested path limits nor later create/rename success are promised. See [`PLAN_IOS_APP_DIRECTORY_NAME_MAX.md`](PLAN_IOS_APP_DIRECTORY_NAME_MAX.md). B170 adds `IosFiles::entry_effective_access` and `IosEntryEffectiveAccess::{bits, allows_read, allows_write, allows_execute_or_search}` through no-follow `getattrlistat(ATTR_CMN_USERACCESS)`. It reports the current effective UID’s R_OK/W_OK/X_OK mask at query time, preserves unnamed bits, maps unsupported-volume attributes to `Unsupported`, and does not guarantee later operation success. See [`PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md`](PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md). B172 adds `IosFiles::volume_rename_support_snapshot(AppDirectory)` and `IosVolumeRenameSupportSnapshot` to expose Foundation’s cached `NSURLVolumeSupportsExclusiveRenamingKey` and `NSURLVolumeSupportsSwapRenamingKey` values as `Option<bool>`. `None` preserves missing/failed/untyped resource values; the cache is from `IosFiles::new` and does not guarantee a future rename. See [`PLAN_IOS_VOLUME_RENAME_SUPPORT.md`](PLAN_IOS_VOLUME_RENAME_SUPPORT.md). B175 adds `IosFiles::volume_name_support_snapshot(AppDirectory)` and `IosVolumeNameSupportSnapshot` for Foundation’s cached `NSURLVolumeSupportsCaseSensitiveNamesKey` and `NSURLVolumeSupportsCasePreservedNamesKey` values. Missing, failed, and untyped resource values remain `None`; this informational snapshot does not alter `AppPath` comparison behavior. See [`PLAN_IOS_VOLUME_NAME_SUPPORT.md`](PLAN_IOS_VOLUME_NAME_SUPPORT.md). B165 audits `_PC_PATH_MAX` and records a no-go: `openat` walks components under retained descriptors, so the full relative path length does not constrain this adapter. B164’s per-component `_PC_NAME_MAX` is the relevant limit. See [`PLAN_IOS_PATH_MAX.md`](PLAN_IOS_PATH_MAX.md). B168 audits file-protection-class metadata and records a no-go: the public SDK/bindings provide no numeric class mapping for `F_GETPROTECTIONCLASS`; Foundation’s string values would re-resolve a path/URL outside this descriptor contract. See [`PLAN_IOS_FILE_PROTECTION_CLASS.md`](PLAN_IOS_FILE_PROTECTION_CLASS.md).

B362 declines `ATTR_VOL_UUID` as a volume identity API. XNU describes a filesystem UUID, typically version 5, but does not define a cross-mount lifetime or app-container meaning; no supported `ios-files` operation needs this identifier. Any use also requires an applicable approved File Timestamp reason. See [`PLAN_IOS_VOLUME_UUID.md`](PLAN_IOS_VOLUME_UUID.md).

B365 adds `IosFiles::volume_used_capacity_bytes(AppDirectory) -> Result<u64, FileError>` through descriptor-bound `fgetattrlist`, after a `validattr.volattr` support check. XNU warns the volume-wide used bytes may differ from volume size minus free space on space-sharing volumes; this is not app usage or quota. Host use requires an applicable approved File Timestamp reason. See [`PLAN_IOS_VOLUME_USED_CAPACITY.md`](PLAN_IOS_VOLUME_USED_CAPACITY.md).

B368 declines `ATTR_VOL_MINALLOCATION`: the volume-level minimum does not predict a general path's future allocation or provide an app quota; B90, B109, B244, and B248 already expose observed per-entry size/allocation values. See [`PLAN_IOS_VOLUME_MINIMUM_ALLOCATION.md`](PLAN_IOS_VOLUME_MINIMUM_ALLOCATION.md).

B371 declines `ATTR_VOL_ALLOCATIONCLUMP`: the filesystem only attempts to allocate the clump as a file grows, and `framework-files` has no streaming buffer policy that consumes the hint. See [`PLAN_IOS_VOLUME_ALLOCATION_CLUMP.md`](PLAN_IOS_VOLUME_ALLOCATION_CLUMP.md).

B374 declines `ATTR_VOL_MAXOBJCOUNT` as a sandbox limit: it describes a whole-volume maximum and cannot predict or authorize a later create. B149's volume-node no-go and B128/B134's direct app-directory counts remain the relevant boundaries. See [`PLAN_IOS_VOLUME_MAX_OBJECT_COUNT.md`](PLAN_IOS_VOLUME_MAX_OBJECT_COUNT.md).

B392 declines `ATTR_VOL_OBJCOUNT`, `ATTR_VOL_FILECOUNT`, and `ATTR_VOL_DIRCOUNT`: they report whole-volume counts, not an app/container count or file-creation budget. B128/B134 remain the direct app-directory count APIs. See [`PLAN_IOS_VOLUME_OBJECT_COUNTS.md`](PLAN_IOS_VOLUME_OBJECT_COUNTS.md).

B395 declines `ATTR_VOL_MOUNTPOINT`: its host absolute path is not an `AppPath`, app-container ID, or access URL, and the descriptor-relative facade has no operation that needs it. See [`PLAN_IOS_VOLUME_MOUNT_POINT.md`](PLAN_IOS_VOLUME_MOUNT_POINT.md).

B398 declines `ATTR_VOL_NAME`: it names the mounted volume rather than an `AppDirectory`, and the adapter has no volume rename or display-label operation. See [`PLAN_IOS_VOLUME_NAME.md`](PLAN_IOS_VOLUME_NAME.md).

B401 declines `ATTR_VOL_MOUNTFLAGS`: it duplicates `statfs.f_flags`; B152 already reports `MNT_RDONLY`, and raw flags do not prove effective app access. See [`PLAN_IOS_VOLUME_MOUNT_FLAGS.md`](PLAN_IOS_VOLUME_MOUNT_FLAGS.md).

B404 declines `ATTR_VOL_MOUNTEDDEVICE`: it duplicates `statfs.f_mntfromname`, may reveal a host device path, and does not establish app-container identity. See [`PLAN_IOS_VOLUME_MOUNTED_DEVICE.md`](PLAN_IOS_VOLUME_MOUNTED_DEVICE.md).

B407 declines `ATTR_VOL_ENCODINGSUSED`: it is a volume-level encoding bitmap, not per-entry evidence or a conversion API; `AppPath` and `read_directory` retain their UTF-8 contract. See [`PLAN_IOS_VOLUME_ENCODINGS.md`](PLAN_IOS_VOLUME_ENCODINGS.md).

B413 declines a generic `ATTR_VOL_CAPABILITIES` bit-set API: existing operations use narrower support snapshots or report operation results, and no current operation consumes the remaining raw capabilities. See [`PLAN_IOS_VOLUME_CAPABILITIES.md`](PLAN_IOS_VOLUME_CAPABILITIES.md).

B416 declines `ATTR_VOL_QUOTA_SIZE`: XNU defines the value as a maximum volume size, not an app or user quota, and B146 already reports volume capacity. See [`PLAN_IOS_VOLUME_QUOTA_SIZE.md`](PLAN_IOS_VOLUME_QUOTA_SIZE.md).

B419 declines `ATTR_VOL_RESERVED_SIZE`: XNU defines it as the minimum size of the volume, not app-reserved bytes or a later-write guarantee. See [`PLAN_IOS_VOLUME_RESERVED_SIZE.md`](PLAN_IOS_VOLUME_RESERVED_SIZE.md).

B422 declines `ATTR_VOL_MOUNTEXTFLAGS`: it duplicates `statfs.f_flags_ext`, and no current operation consumes the extended mount flags. See [`PLAN_IOS_VOLUME_MOUNT_EXTENDED_FLAGS.md`](PLAN_IOS_VOLUME_MOUNT_EXTENDED_FLAGS.md).

B425 declines `ATTR_VOL_SIGNATURE`: XNU defines it only as a 32-bit value unique within a filesystem type for distinguishing that filesystem's volume formats; it does not define stable volume identity or an iOS format mapping, and no current operation consumes it. See [`PLAN_IOS_VOLUME_SIGNATURE.md`](PLAN_IOS_VOLUME_SIGNATURE.md).

B428 declines `ATTR_VOL_SIZE`, `ATTR_VOL_SPACEFREE`, and `ATTR_VOL_SPACEAVAIL`: B146 already reports total capacity, B137 reports bytes available to non-superusers, and B143 declines free blocks that may be reserved. These remain independent volume snapshots, not app quota or write guarantees. See [`PLAN_IOS_VOLUME_SPACE_ATTRIBUTES.md`](PLAN_IOS_VOLUME_SPACE_ATTRIBUTES.md).

B431 adds `IosFiles::app_directory_case_sensitivity(AppDirectory)` using descriptor-bound `fpathconf(_PC_CASE_SENSITIVE)` on the retained root. It maps `0` to `Some(false)`, `1` and legacy FSKit's errno-free `-1` Boolean encoding to `Some(true)`, and `EINVAL` for an unsupported property association to `None`. XNU treats nonzero as case-sensitive; Apple's HFS implementation reports `1`/`0` and returns `EINVAL` for an unknown selector. This fresh filesystem property is distinct from B175's cached Foundation volume-support value. See [`PLAN_IOS_APP_DIRECTORY_CASE_SENSITIVITY.md`](PLAN_IOS_APP_DIRECTORY_CASE_SENSITIVITY.md).

B439 declines a second `_PC_CASE_PRESERVING` query: B175 already caches Foundation's case-preserved-name Boolean for the same app-directory roots, and no current operation needs a fresher duplicate property. See [`PLAN_IOS_APP_DIRECTORY_CASE_PRESERVING.md`](PLAN_IOS_APP_DIRECTORY_CASE_PRESERVING.md).

B446 adds `IosFiles::app_directory_truncates_long_names(AppDirectory)` using `_PC_NO_TRUNC` on a retained app-root descriptor. Apple documents `1` as name truncation and `0` as preserving the name so an overlong operation returns `ENAMETOOLONG`; a legacy FSKit errno-free `-1` Boolean-true encoding is treated as truncation. AppPath traversal checks `_PC_NO_TRUNC` on each opened parent and queries `_PC_NAME_MAX` only when truncation is reported; it returns `ENAMETOOLONG` before a truncating operation could target a different name, and preserves the native syscall error when truncation is false. See [`PLAN_IOS_APP_DIRECTORY_NAME_TRUNCATION.md`](PLAN_IOS_APP_DIRECTORY_NAME_TRUNCATION.md).

B293 adds `IosFiles::entry_data_protection_class_code(&self, path: AppPath<'_>) -> Result<IosFileDataProtectionClassCode, FileError>` using descriptor-bound `fgetattrlist(ATTR_CMN_DATA_PROTECT_FLAGS)` for one regular file or directory. `IosFileDataProtectionClassCode::raw_value()` exposes only the opaque `u32` for preservation/display; the API does not map or name levels, compare across objects/OS versions, or infer access/security guarantees. The host must declare an applicable approved File Timestamp required-reason entry for actual use. See [`PLAN_IOS_FILE_DATA_PROTECTION_CLASS_CODE.md`](PLAN_IOS_FILE_DATA_PROTECTION_CLASS_CODE.md).

B294 audits `ATTR_CMN_OBJPERMANENTID` and records a no-go for this read-only metadata facade. XNU says obtaining the persistent volume-local ID may alter on-disk metadata to create it, and `fid_objno` is undefined on volumes with `VOL_CAP_FMT_64BIT_OBJECT_IDS`; a capability check cannot remove the possible mutation. See [`PLAN_IOS_FILE_OBJECT_PERMANENT_ID.md`](PLAN_IOS_FILE_OBJECT_PERMANENT_ID.md).

B296 adds `IosFiles::entry_object_kind(&self, path: AppPath<'_>) -> Result<IosEntryObjectKind, FileError>` using one `fstatat(..., AT_SYMLINK_NOFOLLOW)` call and `st_mode & S_IFMT`. It distinguishes files, directories, symlinks, FIFOs, sockets, block devices, and character devices, preserving other raw type bits as `Unknown(u32)`. It does not open/follow the final entry or alter portable `FileKind`; the value is point-in-time and retains the concurrent-parent-rename limitation. The host must declare an applicable approved File Timestamp required-reason entry for actual use. See [`PLAN_IOS_ENTRY_OBJECT_KIND.md`](PLAN_IOS_ENTRY_OBJECT_KIND.md).

B301 adds `IosFiles::entry_stored_backup_time(&self, path: AppPath<'_>) -> Result<IosFileBackupTimeMarker, FileError>` for one regular file or directory through descriptor-bound `fgetattrlist(ATTR_CMN_BKUPTIME)`. It preserves the filesystem-stored signed seconds and nanoseconds; the value is not proof of OS/iCloud backup completion, inclusion, freshness, durability, or recoverability. The method does not set the marker or perform backup work. Host use requires an applicable approved File Timestamp required-reason entry. See [`PLAN_IOS_FILE_BACKUP_TIME_MARKER.md`](PLAN_IOS_FILE_BACKUP_TIME_MARKER.md).

B304 declines `ATTR_CMN_PARENTID` as a new `ios-files` operation. The installed iPhoneOS 26.5 SDK defines it as `0x04000000`; locked `libc` 0.2.190 binds the attribute bit and `fgetattrlist`. The SDK declares `fgetattrlist` at iOS 3.0 with no separate availability annotation for this attribute; the `ios-files` floor is iOS 10.0. XNU defines `ATTR_CMN_PARENTID` as a `u_int64_t` that identifies the parent directory, but does not promise a path-specific parent choice for an opened inode with multiple hard links. XNU separately warns that `ATTR_CMN_FULLPATH` may be inconsistent for hard-linked items, especially when the filesystem does not support `PARENTID` natively. The facade has no caller operation that needs this opaque parent identifier, and the existing no-follow `AppPath` traversal already resolves parent directories without exposing an identifier with a documented lifetime. `fgetattrlist` is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype), so host use would require an applicable approved reason. No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B307 declines `ATTR_CMN_OBJTAG` as an app-facing filesystem-type query. The iPhoneOS 26.5 SDK defines `fsobj_tag_t` as `u_int32_t` and the attribute bit as `0x00000010`; locked `libc` 0.2.190 binds the bit. XNU defines the value as a filesystem type tag sourced from `enum vtagtype`, but the installed iPhoneOS SDK does not include `sys/vnode.h` or a public tag-value mapping. A raw value would not select or guarantee a supported filesystem operation, and the facade has no caller operation that needs it. A descriptor-bound `fgetattrlist` call would be available from iOS 3.0, below the package's iOS 10.0 floor, but would require an applicable approved File Timestamp reason in the host privacy manifest. No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and Apple's [required-reason API reference](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype).

B310 declines `ATTR_CMN_UUID` as a second owner-identity query. The installed iPhoneOS 26.5 SDK defines the attribute bit as `0x00800000` and `guid_t` as a 16-byte value; locked `libc` 0.2.190 binds the attribute bit. XNU defines the UUID as the file owner's GUID, analogous to `ATTR_CMN_OWNERID`; B118 already exposes numeric owner/group IDs, and the app-container facade has no operation that needs a second owner identifier or an account mapping. The public contract does not establish a useful app-facing identity lifetime. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B314 declines `ATTR_CMN_SCRIPT` as an app-facing name-encoding operation. The installed iPhoneOS 26.5 SDK defines `text_encoding_t` as `u_int32_t` and the attribute bit as `0x00000100`; locked `libc` 0.2.190 binds the bit. XNU documents it as a read/write text-encoding hint for a filesystem object's name, for lossless conversion between Unicode and traditional Mac OS script encodings. `AppPath` accepts UTF-8 `&str`, and `DirectoryEntry` stores a Rust `String`; the iOS directory adapter also rejects names that are not UTF-8. The facade has no legacy script-encoding input or output operation that can consume this hint, so a raw encoding code would add no useful path contract. `fgetattrlist` is declared from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B319 declines `ATTR_CMN_GRPUUID` as a second owner-identity query. The installed iPhoneOS 26.5 SDK defines the attribute bit as `0x01000000`; locked `libc` 0.2.190 binds it. XNU defines the result as a `guid_t` for the group to which the object belongs, analogous to `ATTR_CMN_GRPID`. B118 already exposes the numeric `st_gid`, and the app-container facade has no group-GUID mapping or operation that uses a second group identity. The value would not establish group membership or effective access. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B321 declines `ATTR_CMN_EXTENDED_SECURITY` as a Rust ACL snapshot. The installed iPhoneOS 26.5 SDK defines the attribute as `0x00400000`; locked `libc` 0.2.190 binds it. XNU describes the result as a variable-length `attrreference` containing `kauth_filesec`, with the ACL entry used; the public `sys/kauth.h` layout contains a variable-count array of ACE records. The facade has no ACL inspection, mutation, or policy-evaluation operation, and a raw ACL would need an explicit contract for ACE rights, flags, and identities. B170 already reports the bounded effective-access mask for the current process. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and the iPhoneOS SDK's `sys/kauth.h`.

B323 declines `ATTR_CMN_FULLPATH` as a second path API. The installed iPhoneOS 26.5 SDK defines the attribute as `0x08000000`; locked `libc` 0.2.190 binds it. XNU documents a UTF-8, null-terminated path that resolves symlinks and warns that hard-linked items can return inconsistent paths. `AppPath` is the facade's validated sandbox-relative path, and no operation needs a resolved absolute path; exposing one would add no supported behavior and would conflict with the facade's no-follow path contract. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B324 declines `ATTR_CMN_FNDRINFO` as a Finder metadata API. The installed iPhoneOS 26.5 SDK defines the attribute as `0x00004000`; locked `libc` 0.2.190 binds it. XNU describes a 32-byte, read/write Finder metadata value equivalent to `FileInfo` plus `ExtendedFileInfo` (or the directory equivalents), with multibyte fields in big-endian order. The app-data facade has no Finder metadata operation or contract for these legacy Finder fields, so exposing the raw blob would add no supported behavior. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B328 declines `ATTR_CMN_OBJID` as an object-identity API. The installed iPhoneOS 26.5 SDK defines the attribute as `0x00000020` and `fsobj_id_t` as two `u_int32_t` fields; locked `libc` 0.2.190 binds the bit. XNU says this ID lasts only for the mount, may change after remount, and can differ between an HFS+ file and its hard links. It is deprecated starting iOS 11.0; XNU says to use `ATTR_CMNEXT_LINKID` instead and limits older use to filesystems without 64-bit IDs. B99 already exposes a point-in-time device/inode pair and B226 exposes link IDs. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B329 declines `ATTR_CMN_PAROBJID` as a parent-identity query. The installed iPhoneOS 26.5 SDK defines it as `0x00000080` with `fsobj_id_t`; locked `libc` 0.2.190 binds the attribute. XNU says this parent ID lasts only for the mount, is nondeterministic for hard-linked objects, and can have significant computation cost. B304 already rejects a parent identifier because `AppPath` resolves parents through the descriptor-relative walk and no facade operation consumes an ID. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B332 declines `ATTR_CMN_FILEID` as a second inode-number query. The installed iPhoneOS 26.5 SDK defines the attribute as `0x02000000`; locked `libc` 0.2.190 binds it. XNU defines its `u_int64_t` result as equivalent to `st_ino`; B99 already returns the no-follow `(st_dev, st_ino)` pair from one metadata lookup. A separate `fgetattrlist` request adds no distinct identity contract and requires an applicable approved File Timestamp reason. No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype).

B333 declines `ATTR_CMN_DEVID` as a second volume-device query. The installed iPhoneOS 26.5 SDK defines it as `0x00000002` and `FSOPT_RETURN_REALDEV` as `0x00000200`; locked `libc` 0.2.190 binds both. XNU defines the default `dev_t` as equivalent to `st_dev`; the option can instead return the physical volume device when volume groups expose a common logical value. B99 already returns `st_dev`, and the facade has no operation that needs physical-volume identity. A separate `fgetattrlist` request adds no supported behavior and requires an applicable approved File Timestamp reason. No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype).

B334 declines `ATTR_CMN_ERROR` as a standalone file capability. The iPhoneOS 26.5 SDK defines the bit as `0x20000000`, but locked `libc` 0.2.190 does not bind it. XNU documents it as an optional per-entry `u_int32_t` error field for `getattrlistbulk`; that bulk call requires `ATTR_CMN_NAME` and `ATTR_CMN_RETURNED_ATTRS`. The SDK makes `getattrlistbulk` available from iOS 8.0, but `ios-files` uses `readdir` and has no bulk metadata result or per-entry failure contract. An error field alone adds no useful file metadata. No source API was added. See [XNU `getattrlistbulk(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlistbulk.2) and the iPhoneOS SDK's `sys/attr.h` and `sys/unistd.h`.

B336 declines `ATTR_CMN_RETURNED_ATTRS` as a standalone capability. The installed iPhoneOS 26.5 SDK defines it as `0x80000000` and says it is valid only with `getattrlist` and `getattrlistbulk`, where it leads the returned buffer; locked `libc` 0.2.190 binds the bit. XNU defines it as the set of requested attributes actually returned, with `FSOPT_PACK_INVAL_ATTRS` requiring this field when unsupported attributes are packed as defaults. `ios-files` has no generic caller-selected attribute query, and the per-operation facade already maps unsupported metadata to `Unsupported`; this buffer protocol does not add a file value. No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and the iPhoneOS SDK's `sys/attr.h`.

B337 declines `ATTR_CMN_NAME` as a separate descriptor-bound name API. The installed iPhoneOS 26.5 SDK defines the attribute as `0x00000001`; locked `libc` 0.2.190 binds it. XNU describes the result as a UTF-8, null-terminated name; `ios-files::read_directory` already obtains each name with `readdir` and returns it in `DirectoryEntry`, while `AppPath` supplies names for path operations. No current operation needs a second name query. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B342 declines `ATTR_CMN_OBJTYPE` as a second entry-kind query. The installed iPhoneOS 26.5 SDK defines `fsobj_type_t` as `u_int32_t` and the attribute bit as `0x00000008`; locked `libc` 0.2.190 binds the bit. XNU says its values come from `enum vtype` in `sys/vnode.h`, which is absent from the installed iPhoneOS SDK. B296 already exposes no-follow entry kind from `st_mode & S_IFMT`, including an `Unknown(u32)` value; the raw vnode code has no public SDK mapping or additional caller operation. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2).

B345 declines `ATTR_CMN_FSID` as a volume-identity API. The installed iPhoneOS 26.5 SDK defines the attribute as `0x00000004` and `FSOPT_RETURN_REALDEV` as `0x00000200`; locked `libc` 0.2.190 binds both. XNU defines the `fsid_t` result as equivalent to `statfs.f_fsid`, with the option able to return a physical-volume value when volume groups expose a common logical value. B158 already concludes that `f_fsid` does not establish persistent volume or app-container identity; `ios-files` has no operation that needs this identifier. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and [`PLAN_IOS_VOLUME_FILESYSTEM_IDENTITY.md`](PLAN_IOS_VOLUME_FILESYSTEM_IDENTITY.md).

B351 declines `ATTR_CMN_ACCESSMASK` as a duplicate stored-permission query. The installed iPhoneOS 26.5 SDK defines the bit as `0x00020000`; locked `libc` 0.2.190 binds it. XNU defines the `u_int32_t` result as the permission bits of `st_mode`, excluding file-type bits. B103 already returns the raw `st_mode & 0o7777` value from the same no-follow entry lookup, so this would add no distinct contract and would not report effective access. `fgetattrlist` is available from iOS 3.0, below the package's iOS 10.0 floor, and is listed in Apple's [File Timestamp required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype). No source API was added. See [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) and [`PLAN_IOS_FILE_PERMISSION_BITS.md`](PLAN_IOS_FILE_PERMISSION_BITS.md).

B178 declines `NSURLVolumeMaximumFileSizeKey` until Apple documentation and the SDK/binding establish an integer byte-count representation; the current reference labels its `NSNumber` Boolean, so converting it could report a false `0` or `1` byte limit. See [`PLAN_IOS_VOLUME_MAXIMUM_FILE_SIZE.md`](PLAN_IOS_VOLUME_MAXIMUM_FILE_SIZE.md). B181 declines `NSURLVolumeSupportsHardLinksKey` because neither `FileBackend` nor `ios-files` creates hard links; B105 already reports a given regular file’s `st_nlink` count. A volume support flag would guide no supported operation. See [`PLAN_IOS_VOLUME_HARD_LINK_SUPPORT.md`](PLAN_IOS_VOLUME_HARD_LINK_SUPPORT.md). B184 declines `NSURLVolumeSupportsSymbolicLinksKey` because the facade has no symlink create/read/follow operation; its current no-follow and unlink behavior is unchanged. B187 declines `NSURLVolumeSupportsAdvisoryFileLockingKey` because the facade has no lock/open-handle API or lock call. B190 declines `NSURLVolumeSupportsSparseFilesKey` because the facade has no sparse-file create, hole, or extent operation; B109 allocated-block counts do not prove sparseness. See [`PLAN_IOS_VOLUME_SPARSE_FILES.md`](PLAN_IOS_VOLUME_SPARSE_FILES.md). See [`PLAN_IOS_VOLUME_SYMLINK_SUPPORT.md`](PLAN_IOS_VOLUME_SYMLINK_SUPPORT.md) and [`PLAN_IOS_VOLUME_ADVISORY_LOCKING.md`](PLAN_IOS_VOLUME_ADVISORY_LOCKING.md).

B193 identifies public `fclonefileat` as a bounded native copy-on-write operation. B196 adds
`IosFiles::clone_regular_file(source, destination)` for one regular `AppPath` source and a
nonexistent destination, with no-follow/beneath flags and no Rust byte buffer; native errors,
cross-volume failures, copy-on-write behavior, metadata semantics, and the concurrent opened-parent
rename limit remain documented in [`PLAN_IOS_FILE_CLONING.md`](PLAN_IOS_FILE_CLONING.md). B199
adds `IosFiles::volume_clone_support_snapshot(AppDirectory) -> Result<Option<bool>, FileError>` as
a cached Foundation volume-level hint only; it does not preflight or guarantee a clone.
[`PLAN_IOS_VOLUME_CLONING_SUPPORT.md`](PLAN_IOS_VOLUME_CLONING_SUPPORT.md) records its API floor and
limits. B202 declines a volume access-permission support snapshot because no descriptor-safe
permission/ACL setter exists in this facade; B205 declines URL-based backup-exclusion mutation
because URL re-resolution cannot preserve its no-follow `AppPath` contract. See
[`PLAN_IOS_VOLUME_ACCESS_PERMISSION_SUPPORT.md`](PLAN_IOS_VOLUME_ACCESS_PERMISSION_SUPPORT.md) and
[`PLAN_IOS_BACKUP_EXCLUSION.md`](PLAN_IOS_BACKUP_EXCLUSION.md).

B208 adds `IosFiles::regular_file_extended_flags_snapshot(&self, path: AppPath<'_>) -> Result<IosFileExtendedFlags, FileError>` using `fgetattrlist` on an opened regular-file descriptor.
It preserves unknown `ATTR_CMNEXT_EXT_FLAGS` bits and names only SDK-defined flags; clone-sharing
flags do not identify a particular peer, and sparse/purgeable flags are not allocation or future
behavior guarantees. B211 adds `IosFiles::regular_file_clone_id_snapshot(&self, path: AppPath<'_>) -> Result<IosFileCloneIdSnapshot, FileError>` for the opaque `ATTR_CMNEXT_CLONEID` value. Equal IDs
are a current XNU report for pure clones, not a persistent identity, content hash, or proof of
current block sharing. B214 adds `IosFiles::regular_file_full_clone_count_snapshot(&self, path: AppPath<'_>) -> Result<IosFileFullCloneCountSnapshot, FileError>` for the descriptor-bound `ATTR_CMNEXT_CLONE_REFCNT` count. XNU defines it as the number of full clones, each sharing all blocks with this file; it does not count partial-sharing peers or expose clone IDs/paths. This is distinct from B208's raw per-file flags and B211's opaque clone ID. `fgetattrlist` is listed in Apple's File Timestamp required-reason API category, so the host must declare an applicable approved `PrivacyInfo.xcprivacy` reason for actual use. See [`PLAN_IOS_FILE_EXTENDED_FLAGS.md`](PLAN_IOS_FILE_EXTENDED_FLAGS.md), [`PLAN_IOS_FILE_CLONE_ID.md`](PLAN_IOS_FILE_CLONE_ID.md), and [`PLAN_IOS_FILE_FULL_CLONE_COUNT.md`](PLAN_IOS_FILE_FULL_CLONE_COUNT.md).

B220 declines `ATTR_CMNEXT_RECURSIVE_GENCOUNT`: XNU returns a useful counter only for APFS directories marked `maintain-dir-stats`, otherwise zero, and the public header exposes no settable extended-common attributes. No public app API to mark an app-sandbox directory was found. See [`PLAN_IOS_RECURSIVE_DIRECTORY_GENERATION.md`](PLAN_IOS_RECURSIVE_DIRECTORY_GENERATION.md).

B223 adds `IosFiles::regular_file_private_size_snapshot(&self, path: AppPath<'_>) -> Result<IosFilePrivateSizeSnapshot, FileError>` for the no-follow descriptor-bound `ATTR_CMNEXT_PRIVATESIZE` value. XNU defines it as bytes not trapped inside a clone or snapshot and freed immediately if the file is deleted; it is not allocated size or a future capacity reservation. The host must declare an applicable approved File Timestamp required-reason API entry for actual `fgetattrlist` use. See [`PLAN_IOS_FILE_PRIVATE_SIZE.md`](PLAN_IOS_FILE_PRIVATE_SIZE.md).

B226 adds `IosFiles::regular_file_link_id_snapshot(&self, path: AppPath<'_>) -> Result<IosFileLinkIdSnapshot, FileError>` for the opaque `ATTR_CMNEXT_LINKID` value. It is unique only within the mounted volume; although XNU describes persistent IDs on some volumes, this API does not query that capability. HFS+/APFS hard-link entries may have distinct link IDs, so this is not B99's inode identity, B211's clone ID, or content identity. The host must declare an applicable approved File Timestamp required-reason API entry for actual `fgetattrlist` use. See [`PLAN_IOS_FILE_LINK_ID.md`](PLAN_IOS_FILE_LINK_ID.md).

B232 declines `ATTR_CMNEXT_ATTRIBUTION_TAG`: XNU defines an optional numeric owner tag and zero when a file is not attributed, but no public mapping from that numeric ID to a bundle identifier or app-data use was found. The field also has no public setter. See [`PLAN_IOS_FILE_ATTRIBUTION_TAG.md`](PLAN_IOS_FILE_ATTRIBUTION_TAG.md).

B231 declines `ATTR_CMNEXT_RELPATH`: it returns a mount-relative physical path rather than an `AppPath`, has documented hard-link inconsistency, and gives this facade no supported operation that needs the container's mount path. See [`PLAN_IOS_MOUNT_RELATIVE_FILE_PATH.md`](PLAN_IOS_MOUNT_RELATIVE_FILE_PATH.md).

## Objective

Implement the iOS backends for the D1 `framework-files` and `framework-preferences` contracts using public iOS filesystem/Foundation APIs. Keep the portable crates `no_std`; platform code may use the platform runtime but must not change portable semantics.

## Dependencies

- Foundation A is integrated
- iOS runtime B is integrated
- D1 `framework-files` and `framework-preferences` contracts are integrated

## Write scope

- D1's sandbox implementation in `platform/ios/ios-files/src/lib.rs` and its existing helpers; B14 owns the URLSession adoption operation and B17 owns the file-coordination module, while root reconciles shared exports and indexes
- `platform/ios/ios-preferences/**`
- `docs/ios/files.md`
- `docs/ios/preferences.md`
- focused iOS backend tests within these crates

Do not edit D1 crates, root workspace configuration, Swift ABI, C bindings, network backend, or unrelated capability families. `platform/ios/*` is already a workspace glob.

## Required implementation

- Implement the D1 `FileBackend` contract with caller-owned backend state and no global service registration.
- Resolve each `AppDirectory` through a documented public iOS sandbox API; keep user-selected, security-scoped document-provider URLs out of this base sandbox backend.
- Enforce the portable relative-path contract at the native boundary. Document symlink handling and concurrent directory-rename limits separately; do not claim selected-root containment where descriptor-relative operations can outlive a native rename of a traversed directory.
- Implement create/replace modes distinctly. If atomic replacement is required, use a same-directory temporary file and atomic rename where the selected API guarantees it; do not equate atomic visibility with crash durability.
- `ReplaceExisting` must require an existing regular file: map a missing target to `NotFound` and final symlink, directory, or special entries to `InvalidInput`. Check before staging and immediately before atomic `RENAME_SWAP`; document that these checks cannot serialize same-path native mutation, including in the non-atomic open/truncate/write fallback.
- Implement D1 preference bytes through `NSUserDefaults`-compatible property-list storage without claiming Keychain secrecy, cross-device sync, or immediate durable flush.
- Report `NotGuaranteed` preference atomicity and reject an atomicity request only when the D1 contract requires that behavior.
- Preserve stable framework error categories and the relevant POSIX/Foundation native code.
- Record Foundation/Objective-C linkage only in the iOS crates that need it. Reuse the existing locked Objective-C/Foundation crate versions and minimal features; add no unrelated dependency.

## Validation and handoff

- Run `cargo check` and Clippy for both crates on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Run portable contract tests from the integrated D1 crates; add deterministic backend tests only for semantics this adapter can prove without user permission or a live app sandbox.
- Inspect both target binaries or a minimal consumer for imported frameworks; confirm network, Swift, and unrelated capability frameworks are absent.
- Document minimum iOS version only when verified from SDK metadata, required Info.plist keys, permission/entitlement state, path and value-copy costs, callback/thread behavior, and native escape handles.
- Report exact checks, unsupported provider-aware file coordination, user-selected document access, preference durability, and any host/toolchain caveat.

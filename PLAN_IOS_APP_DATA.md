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

B137 adds `IosFiles::volume_available_capacity_bytes(AppDirectory)` using checked `f_bavail * f_bsize` from `fstatfs` on the retained root descriptor. This is volume-wide free space available to non-superusers, not an app quota, reservation, or write guarantee. Apple requires an approved `NSPrivacyAccessedAPICategoryDiskSpace` reason in the final `PrivacyInfo.xcprivacy` that matches actual host use; the library does not select a reason or modify a root manifest. See [`PLAN_IOS_VOLUME_CAPACITY.md`](PLAN_IOS_VOLUME_CAPACITY.md).

B146 adds `IosFiles::volume_total_capacity_bytes(AppDirectory)` using checked `f_blocks * f_bsize` from `fstatfs` on the retained semantic-root descriptor. It reports mounted-volume capacity only, not physical-device capacity or an app quota. As with B137, the final host `PrivacyInfo.xcprivacy` must declare a Disk Space reason matching actual use. See [`PLAN_IOS_VOLUME_TOTAL_CAPACITY.md`](PLAN_IOS_VOLUME_TOTAL_CAPACITY.md).

B140 audits `NSURLDirectoryEntryCountKey` and `ATTR_DIR_ENTRYCOUNT` as a possible cheap-count path and records a no-go: the key is optional, `getattrlist(2)` does not guarantee low cost, the count is 32-bit, and not every volume format supports it. B128/B131 already provide deterministic count/empty-query behavior. See [`PLAN_IOS_FAST_DIRECTORY_COUNT.md`](PLAN_IOS_FAST_DIRECTORY_COUNT.md). B143 audits `f_bfree` and records a no-go: it includes reserved filesystem blocks, which the sandbox cannot consume; B137 already exposes the useful `f_bavail` non-superuser value. See [`PLAN_IOS_VOLUME_FREE_BLOCKS.md`](PLAN_IOS_VOLUME_FREE_BLOCKS.md).

B149 audits `f_files`/`f_ffree` and records a no-go: both are volume-wide node counts, not an app/container budget, and `f_ffree` does not establish whether a later sandbox create will succeed. Actual creation remains authoritative. See [`PLAN_IOS_VOLUME_FILE_NODE_COUNTS.md`](PLAN_IOS_VOLUME_FILE_NODE_COUNTS.md).
B152 adds `IosFiles::volume_is_read_only(AppDirectory)` using `fstatfs` on the retained semantic root. It reports only `MNT_RDONLY`; `false` does not establish effective path or app write access. B155 adds `IosFiles::volume_optimal_io_block_size_bytes(AppDirectory)` using `fstatfs.f_iosize`, a filesystem sizing hint only—not an alignment mandate or performance guarantee. B155 is a distinct volume-level field from B121’s per-entry `st_blksize` audit. Both query Disk Space required-reason APIs, so host `PrivacyInfo.xcprivacy` must use an approved reason matching actual use. See [`PLAN_IOS_VOLUME_READ_ONLY.md`](PLAN_IOS_VOLUME_READ_ONLY.md) and [`PLAN_IOS_VOLUME_IO_SIZE.md`](PLAN_IOS_VOLUME_IO_SIZE.md). B158 declines `f_fsid`, `f_owner`, `f_type`, `f_fssubtype`, and `f_fstypename` as app-facing volume identity because they do not define a persistent/container identity or stable iOS filesystem mapping. B161 declines `MNT_NOEXEC`, `MNT_NOSUID`, `MNT_NODEV`, `MNT_QUOTA`, and `MNT_DONTBROWSE` as effective path/app authorization signals; B152 exposes only the read-only mount bit. B164 adds `IosFiles::app_directory_name_max_bytes(AppDirectory)` using `fpathconf(_PC_NAME_MAX)` on the retained root. `Some(n)` is a direct-child component limit in bytes; `None` is the POSIX no-limit sentinel, and neither total/nested path limits nor later create/rename success are promised. See [`PLAN_IOS_APP_DIRECTORY_NAME_MAX.md`](PLAN_IOS_APP_DIRECTORY_NAME_MAX.md). B170 adds `IosFiles::entry_effective_access` and `IosEntryEffectiveAccess::{bits, allows_read, allows_write, allows_execute_or_search}` through no-follow `getattrlistat(ATTR_CMN_USERACCESS)`. It reports the current effective UID’s R_OK/W_OK/X_OK mask at query time, preserves unnamed bits, maps unsupported-volume attributes to `Unsupported`, and does not guarantee later operation success. See [`PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md`](PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md). B172 adds `IosFiles::volume_rename_support_snapshot(AppDirectory)` and `IosVolumeRenameSupportSnapshot` to expose Foundation’s cached `NSURLVolumeSupportsExclusiveRenamingKey` and `NSURLVolumeSupportsSwapRenamingKey` values as `Option<bool>`. `None` preserves missing/failed/untyped resource values; the cache is from `IosFiles::new` and does not guarantee a future rename. See [`PLAN_IOS_VOLUME_RENAME_SUPPORT.md`](PLAN_IOS_VOLUME_RENAME_SUPPORT.md). B175 adds `IosFiles::volume_name_support_snapshot(AppDirectory)` and `IosVolumeNameSupportSnapshot` for Foundation’s cached `NSURLVolumeSupportsCaseSensitiveNamesKey` and `NSURLVolumeSupportsCasePreservedNamesKey` values. Missing, failed, and untyped resource values remain `None`; this informational snapshot does not alter `AppPath` comparison behavior. See [`PLAN_IOS_VOLUME_NAME_SUPPORT.md`](PLAN_IOS_VOLUME_NAME_SUPPORT.md). B165 audits `_PC_PATH_MAX` and records a no-go: `openat` walks components under retained descriptors, so the full relative path length does not constrain this adapter. B164’s per-component `_PC_NAME_MAX` is the relevant limit. See [`PLAN_IOS_PATH_MAX.md`](PLAN_IOS_PATH_MAX.md). B168 audits file-protection-class metadata and records a no-go: the public SDK/bindings provide no numeric class mapping for `F_GETPROTECTIONCLASS`; Foundation’s string values would re-resolve a path/URL outside this descriptor contract. See [`PLAN_IOS_FILE_PROTECTION_CLASS.md`](PLAN_IOS_FILE_PROTECTION_CLASS.md).

B178 declines `NSURLVolumeMaximumFileSizeKey` until Apple documentation and the SDK/binding establish an integer byte-count representation; the current reference labels its `NSNumber` Boolean, so converting it could report a false `0` or `1` byte limit. See [`PLAN_IOS_VOLUME_MAXIMUM_FILE_SIZE.md`](PLAN_IOS_VOLUME_MAXIMUM_FILE_SIZE.md). B181 declines `NSURLVolumeSupportsHardLinksKey` because neither `FileBackend` nor `ios-files` creates hard links; B105 already reports a given regular file’s `st_nlink` count. A volume support flag would guide no supported operation. See [`PLAN_IOS_VOLUME_HARD_LINK_SUPPORT.md`](PLAN_IOS_VOLUME_HARD_LINK_SUPPORT.md). B184 declines `NSURLVolumeSupportsSymbolicLinksKey` because the facade has no symlink create/read/follow operation; its current no-follow and unlink behavior is unchanged. B187 declines `NSURLVolumeSupportsAdvisoryFileLockingKey` because the facade has no lock/open-handle API or lock call. B190 declines `NSURLVolumeSupportsSparseFilesKey` because the facade has no sparse-file create, hole, or extent operation; B109 allocated-block counts do not prove sparseness. See [`PLAN_IOS_VOLUME_SPARSE_FILES.md`](PLAN_IOS_VOLUME_SPARSE_FILES.md). See [`PLAN_IOS_VOLUME_SYMLINK_SUPPORT.md`](PLAN_IOS_VOLUME_SYMLINK_SUPPORT.md) and [`PLAN_IOS_VOLUME_ADVISORY_LOCKING.md`](PLAN_IOS_VOLUME_ADVISORY_LOCKING.md).

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

# iOS sandbox files

## Scope and setup

`ios-files` implements `framework_files::FileBackend` as a caller-owned `IosFiles` value. It has
no global registration, executor, callback, or permission prompt. Construct it with
`IosFiles::new()` and pass it to `framework_files::Files`.

The backend resolves Documents, Caches, and Application Support with Foundation's public
[`NSFileManager` directory URL API](https://developer.apple.com/documentation/foundation/nsfilemanager)
`URLForDirectory:inDomain:appropriateForURL:create:error:`. It resolves Temporary with
[`NSFileManager.temporaryDirectory`](https://developer.apple.com/documentation/foundation/filemanager/temporarydirectory).
Each URL is opened once with a directory/no-follow open, and the backend retains the descriptor
for its lifetime. These are app-sandbox locations; the sandbox backend and `Files` facade do not
accept user-selected document-provider URLs, security-scoped bookmarks, iCloud container URLs, or
file-provider URLs. The separate [`IosPlainBookmarkData` helper](bookmark-resolution.md) creates
location-only bookmark data from a caller-owned file URL with implicit scope omitted and safely
resolves only that data through its opaque type. B83's `IosResolvedBookmark::resolve_unscoped` still
uses an unsafe contract for raw caller-supplied bookmark data. Neither helper starts a security
scope or adds a URL to the sandbox `Files` facade.
[A separate iOS extension](file-coordination.md)
coordinates caller-supplied file URLs for synchronous read/write access, but does not establish
sandbox-root containment or file-provider lifecycle support. Callers that already hold a scoped
file URL may balance its Foundation access lifetime with the separate
[security-scope guard](security-scoped-access.md); that guard does not select a URL or add file I/O.

`IosFiles::adopt_url_session_download` is a separate additive operation outside `FileBackend` that
copies a URLSession download callback's temporary file into an `AppPath`; call it before the
delegate callback returns. The method cannot verify URLSession provenance, so pass only that
callback URL. This operation does not add arbitrary file-URL, provider, picker, or security-scope
support; see the [file-adoption guide](file-adoption.md) for copy cost, atomic commit, and
namespace-race limits.

The backend's API floor is iOS 10.0. The Xcode 26.5 iOS SDK headers declare
`renameatx_np`, `NSFileManager.temporaryDirectory`, and both volume rename-support resource keys
available from iOS 10.0. The crate does not set a deployment target; a device app must select iOS
10.0 or later. The link probe's device executable recorded minos 10.0. The Rust arm64 simulator
target recorded minos 14.0 in `LC_BUILD_VERSION`, despite the same deployment-target environment
value. These are SDK/header and binary load-command records, not runtime tests.

No Info.plist usage-description key, permission, or entitlement is required for these ordinary
sandbox directories. The app sandbox and standard iOS file-protection policy still apply.

## Path and symlink policy

Every call rechecks the portable slash-separated relative-path rules at the native boundary: no
empty path, leading/trailing slash, Windows drive prefix (for example, `C:folder`), empty segment,
`.` or `..` segment, backslash, or NUL. Each component is passed separately to descriptor-relative
POSIX calls. Parent traversal uses
`openat` with `O_DIRECTORY | O_NOFOLLOW`; final file opens also use `O_NOFOLLOW`. With the directory
namespace unchanged, this avoids string-prefix checks and prevents intermediate or final symlinks
from redirecting reads, writes, or directory traversal outside the selected root. It does not
prevent a concurrent native rename from moving an already-open directory outside that root, as
described below. The `libc` dependency is private to this crate and provides the POSIX
declarations/constants needed for this boundary; Foundation remains the narrow public API for
sandbox URL resolution.

This is symlink-resistant traversal, not confinement against concurrent native directory renames.
An already-open descriptor continues to refer to its directory inode if another native handle moves
that directory outside the selected root; a later descriptor-relative operation can then act in the
moved directory. The backend does not serialize that namespace mutation.

## Regular file size

`IosFiles::regular_file_size` returns the `u64` byte length of one regular file without reading its
contents into a `Vec`. It accepts the same validated `AppPath` and uses the already-open semantic
root, no-follow parent traversal, and one `fstatat(..., AT_SYMLINK_NOFOLLOW)` final-entry lookup.
A missing path returns `NotFound`; a final symlink, directory, or special entry returns
`InvalidInput`. The result is a point-in-time metadata snapshot and may differ from a later read if
another handle changes or replaces the file. It is iOS-only and does not change the portable
`framework-files` contract, follow arbitrary URLs, start a security scope, or prove stronger
containment under concurrent parent-directory rename. See the [B90 plan](../../PLAN_IOS_FILE_SIZE.md).

`IosFiles::regular_file_total_fork_size_snapshot` returns XNU's logical byte count across all
forks of one regular file. It opens the validated `AppPath` with no-follow descriptor traversal and
queries `ATTR_FILE_TOTALSIZE` through `fgetattrlist` on that file descriptor. This total can differ
from the data-fork length returned by `regular_file_size`; it is not a `FileBackend::read` buffer
size and does not expose or read resource-fork contents. Filesystem support can vary, and the value
is a point-in-time snapshot. The query reads no contents, accepts no arbitrary URL, starts no
security scope, and does not change portable file behavior. Apple lists `fgetattrlist` in the File
Timestamp required-reason API category, so the host must declare an applicable approved reason in
`PrivacyInfo.xcprivacy` for actual use. See the [B242 plan](../../PLAN_IOS_FILE_TOTAL_FORK_SIZE.md).

`IosFiles::regular_file_data_fork_allocated_size_snapshot` returns the filesystem-reported bytes
allocated to the data fork only, using `ATTR_FILE_DATAALLOCSIZE` through `fgetattrlist` on an opened
regular-file descriptor. It excludes resource-fork allocation and is not a guarantee of exclusive
physical-device storage. This is distinct from B109's raw `st_blocks` count in 512-byte units; the
API does not assert that the two values have a fixed conversion. The query reads no file contents,
accepts no arbitrary URL, starts no security scope, and does not change portable file behavior.
Apple lists `fgetattrlist` in the File Timestamp required-reason API category, so the host must
declare an applicable approved reason in `PrivacyInfo.xcprivacy` for actual use. See the
[B244 plan](../../PLAN_IOS_FILE_DATA_FORK_ALLOCATED_SIZE.md).

`IosFiles::regular_file_resource_fork_allocated_size_snapshot` returns the filesystem-reported
bytes allocated to the resource fork only, using `ATTR_FILE_RSRCALLOCSIZE` through `fgetattrlist`
on an opened regular-file descriptor. It excludes data-fork allocation and does not read, enumerate,
or grant access to resource-fork contents. A zero value is only the reported size and does not
establish that a resource fork is absent. Filesystem support can vary, and the value is a
point-in-time snapshot. This iOS-only query does not change portable file behavior. Apple lists
`fgetattrlist` in the File Timestamp required-reason API category, so the host must declare an
applicable approved reason in `PrivacyInfo.xcprivacy` for actual use. See the
[B248 plan](../../PLAN_IOS_FILE_RESOURCE_FORK_ALLOCATED_SIZE.md).

`IosFiles::regular_file_resource_fork_size_snapshot` returns the filesystem-reported logical byte
length for the resource fork only, using `ATTR_FILE_RSRCLENGTH` through `fgetattrlist` on an opened
regular-file descriptor. It does not enumerate or read fork contents; zero is the reported length
and does not prove that a resource fork is absent. Filesystem support can vary, and the value is a
point-in-time snapshot. The iOS-only query does not change portable file behavior. Apple lists
`fgetattrlist` in the File Timestamp required-reason API category, so the host must declare an
applicable approved reason in `PrivacyInfo.xcprivacy` for actual use. See the
[B251 plan](../../PLAN_IOS_FILE_RESOURCE_FORK_SIZE.md).

`IosFiles::regular_file_document_id_snapshot` returns `IosFileDocumentIdSnapshot`; its
`document_id()` accessor maps XNU's invalid zero value to `None` and returns nonzero IDs as
`Some(u32)`. The query uses `ATTR_CMN_DOCUMENT_ID` with `FSOPT_ATTR_CMN_EXTENDED` through
`fgetattrlist` on an opened regular-file descriptor. XNU describes this ID as a kernel-assigned
document value that tracks data across moves and stays sticky to its assigned path across safe
saves. The API makes no inode, content-hash, clone-ID, link-ID, cross-volume, or durable-identity
guarantee; filesystem support can vary. A filesystem that omits the requested value yields
`Unsupported`. This iOS-only query reads no content, accepts no arbitrary URL, starts no security
scope, and does not change portable file behavior. The host must declare an applicable approved
File Timestamp reason in `PrivacyInfo.xcprivacy` for actual use. See the [B266 plan](../../PLAN_IOS_FILE_DOCUMENT_ID.md).

`IosFiles::regular_file_allocated_blocks_512` returns the filesystem-reported `st_blocks` count
in 512-byte units for one regular file. This differs from logical file size; sparse files may
report fewer allocated blocks than their logical size implies. It is not a promise of exact
physical-device usage or exclusive allocation. The query uses one no-follow metadata lookup,
rejects final symlinks/directories/special entries, reads no contents, and changes no portable
`FileBackend` behavior. See the [B109 plan](../../PLAN_IOS_FILE_ALLOCATED_BLOCKS.md).

## Regular file clone

`IosFiles::clone_regular_file(source, destination)` creates a native copy-on-write clone of one
regular sandbox file at a destination that must not exist. Both paths use `AppPath` and the
retained semantic roots. The source opens with `O_NOFOLLOW` and must pass a regular-file check;
the destination parent uses no-follow descriptor traversal, and `fclonefileat` receives only its
validated final component with `CLONE_NOFOLLOW_ANY | CLONE_RESOLVE_BENEATH`. Existing destination
entries are never replaced. The syscall is expected to publish a complete clone atomically or
create no destination, per XNU's `clonefile(2)` contract; this does not promise crash durability.

The separate `IosFiles::volume_clone_support_snapshot(directory)` reports Foundation's cached
optional `NSURLVolumeSupportsFileCloningKey` Boolean for one retained app-directory volume. It can
help a caller decide whether to attempt a clone, but it is only a volume-level hint; it does not
establish that a particular pair of paths, clone flags, or future call will succeed. The clone
syscall's result remains authoritative. The snapshot is read at `IosFiles::new`, returns `None`
when Foundation cannot provide a Boolean, and is available from iOS 10.0.

The clone may share data blocks at first, but later writes to either file are private. A later
overwrite can still fail with `ENOSPC`, so the API makes no speed, storage, or future-write promise.
Native attributes and extended attributes follow XNU clone semantics; without `CLONE_ACL`, the
destination inherits ACLs from its parent. Concurrent source writes are not serialized. Source
and destination on separate filesystems return the mapped native error. The API does not inspect
or preflight the volume-cloning resource key. It needs no usage-description key, permission, or
entitlement. It is an iOS-only operation outside
`framework-files::FileBackend`; it accepts no arbitrary URL, provider path, or security-scoped URL,
and keeps the existing concurrent opened-parent directory-rename limit. The public function is
available from iOS 10.0 in the SDK; the no-follow/beneath flags are passed on every call, with no
weaker fallback. See the [B196 plan](../../PLAN_IOS_FILE_CLONING.md#b196-implementation).

## Regular-file extended flags

`IosFiles::regular_file_extended_flags_snapshot(path)` returns raw `ATTR_CMNEXT_EXT_FLAGS` for one
regular sandbox file through `IosFileExtendedFlags`. It opens the final entry with `O_NOFOLLOW`,
checks the opened descriptor is regular, then uses `fgetattrlist` on that descriptor. The raw `u64`
preserves unnamed bits. Named SDK flags include `EF_MAY_SHARE_BLOCKS`, `EF_SHARES_ALL_BLOCKS`, and
`EF_IS_SPARSE`; the first two report whether this file may share blocks or shares all blocks with
another file, not whether it shares with a particular `clone_regular_file` source. They are not
stable sharing or allocation guarantees. `EF_IS_SPARSE` reports a sparse-region flag and is distinct
from allocated-block count. `EF_NO_XATTRS` reports no extended attributes; `EF_IS_PURGEABLE` means
the filesystem may delete the file when asked to free space, not that it will or should.

The descriptor binds the metadata query to the opened inode; no URL or path lookup occurs after
open. Unsupported filesystem attributes return `Unsupported`. The query reads no file contents,
accepts no arbitrary URL, starts no security scope, and does not change portable `FileBackend`
behavior. See the [B208 plan](../../PLAN_IOS_FILE_EXTENDED_FLAGS.md).

`IosFiles::regular_file_clone_id_snapshot(path)` returns the opaque 64-bit `ATTR_CMNEXT_CLONEID`
value for an opened regular file. XNU documents equal clone IDs as a way to find pure clones that
share a data stream. This value is not a path or peer identity, current block-sharing proof,
content hash, persistent identifier, or change token. The query uses the same no-follow opened-file
descriptor path and returns `Unsupported` when the filesystem does not support clone IDs. See the
[B211 plan](../../PLAN_IOS_FILE_CLONE_ID.md).

`IosFiles::regular_file_full_clone_count_snapshot(path)` returns the current `u32` count of full
clones reported for one opened regular file. XNU defines each counted clone as sharing all of its
blocks with this file. The count omits partial block-sharing peers and does not expose clone paths,
IDs, or durable identity; it can change as files are cloned or removed. This is distinct from
B208's per-file flags and B211's per-file clone ID. The query uses the same no-follow descriptor
path and returns `Unsupported` when the filesystem lacks the attribute. `fgetattrlist` is listed
by Apple under the File Timestamp required-reason API category, so a host app must declare an
applicable approved reason in `PrivacyInfo.xcprivacy` for actual use. This operation adds no
permission, usage-description key, or entitlement. See the
[B214 plan](../../PLAN_IOS_FILE_FULL_CLONE_COUNT.md).

`IosFiles::regular_file_private_size_snapshot(path)` returns the current `ATTR_CMNEXT_PRIVATESIZE`
byte count for one opened regular file. XNU defines this as bytes not trapped in a clone or snapshot
that would be freed immediately if the file were deleted. This is distinct from allocated size and
B214's count of full clone peers; it is not a space reservation or a guarantee for a later delete
or write. The method uses the no-follow descriptor path and returns `Unsupported` when the
filesystem lacks the attribute. Like other `fgetattrlist` calls, the host app must declare an
applicable approved File Timestamp required-reason API entry in `PrivacyInfo.xcprivacy` for actual
use. See the [B223 plan](../../PLAN_IOS_FILE_PRIVATE_SIZE.md).

`IosFiles::regular_file_link_id_snapshot(path)` returns the opaque `ATTR_CMNEXT_LINKID` value for
one opened regular-file entry. XNU scopes uniqueness to the mounted volume; it describes persistent
values only on volumes that support `VOL_CAP_FMT_PERSISTENTOBJECTIDS`. On HFS+ and APFS, hard-link
entries can have distinct link IDs, so this is not the same identity as `(st_dev, st_ino)` and is
not a content or clone identity. The API does not query persistent-ID volume support, so callers
must limit comparison to the current mount. `fgetattrlist` also requires an applicable approved
File Timestamp reason in the host `PrivacyInfo.xcprivacy`. See the
[B226 plan](../../PLAN_IOS_FILE_LINK_ID.md).

## Single-entry kind

`IosFiles::entry_kind` classifies one validated sandbox `AppPath` with the same `FileKind` values
used by `read_directory`: regular files are `File`, directories are `Directory`, and symbolic links
or special entries are `Other`. It inspects only the final entry with `fstatat` and
`AT_SYMLINK_NOFOLLOW`, so it reads no file contents and does not follow a final symlink. A missing
entry returns `NotFound`; parent traversal and metadata errors use the existing POSIX mapping. The
answer is point-in-time and does not reserve the path for a later operation. This is an iOS-only
helper, not a new portable `FileBackend` operation; it grants no URL or security-scope access and
keeps the existing concurrent parent-directory rename limit. See the [B93 plan](../../PLAN_IOS_ENTRY_KIND.md).

`IosFiles::entry_object_kind` gives a more detailed iOS-only classification through the same
no-follow `fstatat` lookup. It returns `File`, `Directory`, `Symlink`, `Fifo`, `Socket`,
`BlockDevice`, or `CharacterDevice`; an unrecognized mode type is `Unknown(raw_type_bits)`. Unlike
`entry_kind`, it does not group symlinks and special files into `FileKind::Other`. The method does
not open the entry, so it can classify a FIFO without blocking, and it does not follow a final
symlink. This is only point-in-time metadata, not a later-operation guarantee; it keeps B1's
concurrent opened-parent directory-rename limit and adds no portable `FileBackend` behavior. See
the [B296 plan](../../PLAN_IOS_ENTRY_OBJECT_KIND.md). Apple lists `fstatat` in the File Timestamp
required-reason API category; the host app must declare an applicable approved reason in its
`PrivacyInfo.xcprivacy` for actual use.

## Direct directory-entry count

`IosFiles::directory_entry_count` counts direct names in one validated sandbox directory, except
`.` and `..`. It counts all entry kinds and byte names, including symbolic links and names that
`read_directory` cannot represent as UTF-8. The method scans with `readdir` but does not build a
`Vec<DirectoryEntry>`, copy each name to a `String`, or run a kind lookup for every entry. The
directory stream itself may allocate libc memory, and time cost remains proportional to the entry
count. Concurrent namespace changes can affect the observed count; it is not an atomic snapshot,
reservation, or safe basis to remove the directory. It adds no portable `FileBackend` operation,
file-content access, URL support, or security-scope access. See the [B128 plan](../../PLAN_IOS_DIRECTORY_ENTRY_COUNT.md).

`IosFiles::directory_is_empty` stops once it sees the first direct name other than `.` or `..`,
so it can avoid a full scan when the directory is nonempty. It uses the same no-follow path rules
and may use libc stream memory. Concurrent namespace changes can affect the result; `true` does not
reserve the directory or guarantee a later removal. `remove_directory` remains authoritative. This
iOS-only query adds no portable `FileBackend` operation, file-content access, URL support, or
security-scope access. See the [B131 plan](../../PLAN_IOS_DIRECTORY_EMPTY_CHECK.md).

`IosFiles::directory_entry_kind_counts` returns fixed-width counts for regular files, directories,
and `Other` entries. It applies the same no-follow `fstatat` classification as `read_directory` but
does not decode or copy names or build a `Vec<DirectoryEntry>`. A name can disappear or change
between `readdir` and its kind lookup; in that case the query may return the mapped POSIX error
rather than partial counts. Concurrent namespace mutation also means the result is not an atomic
snapshot or delete guard. The method adds no portable `FileBackend` operation, file-content
access, URL support, or security-scope access. See the [B134 plan](../../PLAN_IOS_DIRECTORY_KIND_COUNTS.md).

`IosFiles::directory_allocated_size_snapshot(path)` returns the filesystem's point-in-time on-disk
allocation for the directory object itself. It does not sum child files or descendant directories,
and it is not an app quota, reservation, or performance signal. The query uses the no-follow opened
directory descriptor and returns `Unsupported` when the filesystem does not provide
`ATTR_DIR_ALLOCSIZE`. As with other `fgetattrlist` calls, host use requires an applicable approved
File Timestamp reason in `PrivacyInfo.xcprivacy`. See the
[B235 plan](../../PLAN_IOS_DIRECTORY_ALLOCATED_SIZE.md).

## Volume available capacity

`IosFiles::volume_available_capacity_bytes` reports `f_bavail * f_bsize` from `fstatfs` on the
retained root descriptor for a semantic app directory. It is volume-wide free space available to
non-superusers, not an app-specific quota, reservation, or guarantee that a later write will
succeed. Apple classifies `fstatfs` as a required-reason Disk Space API. The app or SDK that uses
it must declare `NSPrivacyAccessedAPICategoryDiskSpace` in its privacy manifest with an approved
reason that matches actual behavior. For display to the person, Apple lists reason `85F4.1` and
limits off-device transfer; reason `E174.1` applies only when the app checks whether space is
sufficient/low and changes user-observable behavior. The library selects no reason on behalf of a
host app. See the [B137 plan](../../PLAN_IOS_VOLUME_CAPACITY.md).

`IosFiles::volume_total_capacity_bytes` reports `f_blocks * f_bsize` from the same retained root
descriptor. This is total data capacity reported for that mounted volume, not physical device
capacity or an app-container limit. It is another point-in-time volume value, not a storage
reservation. It has the same required-reason Disk Space privacy-manifest obligation as B137 and
does not add an Info.plist key or entitlement. See the [B146 plan](../../PLAN_IOS_VOLUME_TOTAL_CAPACITY.md).

`IosFiles::volume_is_read_only` checks whether `fstatfs` reports the `MNT_RDONLY` mount flag for
the volume that contains a retained semantic app-directory root. This is a volume-mount property,
not an effective-write-access check: a writable mount does not bypass sandbox policy, directory
permissions, file protection, file flags, quotas, or low space. The result can change after the
snapshot, and a real write remains authoritative. The query has the same required-reason Disk
Space privacy-manifest obligation as B137; it adds no Info.plist key or entitlement. See the
[B152 plan](../../PLAN_IOS_VOLUME_READ_ONLY.md).

`IosFiles::volume_optimal_io_block_size_bytes` returns `f_iosize` from `fstatfs` for the volume
containing a retained semantic app-directory root. Apple describes this as the filesystem's
optimal transfer block size; callers may use it as a sizing hint for their own I/O, but it is not
an alignment requirement, buffer-size mandate, or performance guarantee. A nonpositive native
value maps to `InvalidInput`. This query has the same required-reason Disk Space privacy-manifest
obligation as B137 and adds no Info.plist key or entitlement. See the
[B155 plan](../../PLAN_IOS_VOLUME_IO_SIZE.md).

`IosFiles::volume_rename_support_snapshot` returns the two Foundation rename-option values cached
by `IosFiles::new` for a semantic app-directory root. Its `IosVolumeRenameSupportSnapshot` methods
return `Some(true)` or `Some(false)` when Foundation supplied an `NSNumber`; `None` means the
resource value could not be read or was not a number. The values report only volume support for
`RENAME_EXCL` and `RENAME_SWAP`; they do not promise that a later operation will succeed. The
existing `FileBackend` still treats an unreported value as unsupported and uses its conservative
fallback. This iOS-only query adds no portable `FileBackend` operation or permission requirement.
See the [B172 plan](../../PLAN_IOS_VOLUME_RENAME_SUPPORT.md).

`IosFiles::volume_name_support_snapshot` returns Foundation's cached case-sensitive and
case-preserved name values for one semantic app-directory volume. `Some(false)` is a reported
negative; `None` means the key query did not produce a Boolean `NSNumber`. These volume-level
values may help a caller choose filename-comparison behavior, but they do not define Unicode
normalization/collation, prevent name races, or guarantee a later create/rename result. The
portable `AppPath` comparison and validation rules remain unchanged. See the
[B175 plan](../../PLAN_IOS_VOLUME_NAME_SUPPORT.md).

Other Foundation volume values were audited without adding standalone support snapshots. B178 declines
`NSURLVolumeMaximumFileSizeKey` because Apple's online reference describes the returned
`NSNumber` as Boolean while the SDK header describes a byte count; the binding does not resolve
the numeric representation, so a Boolean must not become a false byte limit. B181 declines
`NSURLVolumeSupportsHardLinksKey` because the file facade has no hard-link creation operation and
B105 already reports the link count for one regular file. See the
[B178 audit](../../PLAN_IOS_VOLUME_MAXIMUM_FILE_SIZE.md) and
[B181 audit](../../PLAN_IOS_VOLUME_HARD_LINK_SUPPORT.md). B184 declines the volume symbolic-link support flag because the facade has no symlink operation; its no-follow policy remains in force. B187 declines the advisory-locking support flag because the facade has no lock or open-handle operation. See the [B184 audit](../../PLAN_IOS_VOLUME_SYMLINK_SUPPORT.md) and [B187 audit](../../PLAN_IOS_VOLUME_ADVISORY_LOCKING.md). B190 declines `NSURLVolumeSupportsSparseFilesKey` because the facade has no sparse-file create, hole, or extent operation, and allocated-block count does not prove sparseness. See the [B190 audit](../../PLAN_IOS_VOLUME_SPARSE_FILES.md).
B193 identified `fclonefileat` as a concrete operation; B196 implements the bounded regular-file clone API, and B199 exposes Foundation's volume-level clone-support hint separately. The hint does not preflight or guarantee a clone call. See the [B193/B196 plan](../../PLAN_IOS_FILE_CLONING.md) and [B199 plan](../../PLAN_IOS_VOLUME_CLONING_SUPPORT.md).

`IosFiles::app_directory_name_max_bytes` queries `_PC_NAME_MAX` on the retained descriptor for
one semantic app-directory root. The result applies to direct child filename components in that
root, in bytes; it is not a total path limit and does not establish limits for nested directories.
`None` represents the POSIX no-limit sentinel. A returned limit does not guarantee
that a later create or rename succeeds. This query adds no portable `FileBackend` operation. See
the [B164 plan](../../PLAN_IOS_APP_DIRECTORY_NAME_MAX.md).

## Entry modification time

`IosFiles::entry_modification_time` returns the raw POSIX seconds and nanoseconds from one
`fstatat(..., AT_SYMLINK_NOFOLLOW)` call. It reports the final entry itself, including a symlink
rather than its target, and accepts regular files, directories, symlinks, and special entries. The
seconds are relative to the Unix epoch; the nanoseconds field is in `0..1_000_000_000`, though the
filesystem may have coarser precision. Callers may set timestamps, so this is not a content
version, reliable change token, or durability proof. A missing entry returns `NotFound`; other
lookup errors use the existing POSIX mapping. The result does not reserve the path or change
portable `FileBackend` semantics. See the [B96 plan](../../PLAN_IOS_FILE_MODIFICATION_TIME.md).

## Entry status-change time

`IosFiles::entry_status_change_time` returns the raw POSIX seconds and nanoseconds from
`st_ctime`/`st_ctime_nsec` in one `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup. Apple defines this
as the last file-status change time, which can change for metadata operations as well as writes;
it is distinct from B96's data-modification time. A final symlink returns `InvalidInput` because
Apple documents that `lstat` does not provide timestamps belonging to the link itself. The value
is a point-in-time snapshot with filesystem-dependent precision, not a content version or reliable
change token. It reads no contents, grants no URL or security-scope access, and changes no
portable `FileBackend` semantics. See the [B107 plan](../../PLAN_IOS_FILE_STATUS_CHANGE_TIME.md).

## Entry access time

`IosFiles::entry_access_time` returns the raw POSIX seconds and nanoseconds from
`st_atime`/`st_atime_nsec` in one `fstatat(..., AT_SYMLINK_NOFOLLOW)` lookup. Apple defines this
as the time file data was last accessed, but it can also be set explicitly; the query cannot prove
that every data access refreshed it. A final symlink returns `InvalidInput` because Apple does not
define timestamps belonging to the link itself. Treat the value as a filesystem-reported
diagnostic, not an access log, content version, or reliable change token. It reads no target
contents, grants no URL or security-scope access, and changes no portable `FileBackend` semantics.
See the [B112 plan](../../PLAN_IOS_FILE_ACCESS_TIME.md).

## Entry added-to-directory time

`IosFiles::entry_added_time` returns the raw seconds and nanoseconds from Apple's
`ATTR_CMN_ADDEDTIME` for one opened regular file. XNU defines this field as the time the object was
created or renamed into its containing directory, and warns that values may be inconsistent for
hard-linked items. It is not a reliable creation-time or path-history record; B101's
`st_birthtime` no-go remains distinct. A final symlink or non-regular entry returns `InvalidInput`,
and a filesystem that omits the attribute returns `Unsupported`. This iOS-only query reads no file
contents, accepts no arbitrary URL, starts no security scope, and does not change portable
`FileBackend` semantics. The host must declare an applicable approved File Timestamp reason in
`PrivacyInfo.xcprivacy` for actual use. See the [B272 plan](../../PLAN_IOS_FILE_ADDED_TIME.md).

## Stored backup-time marker

`IosFiles::entry_stored_backup_time` reads the filesystem's stored `ATTR_CMN_BKUPTIME` marker for
one opened regular file or directory and returns its seconds/nanoseconds pair. XNU documents the
field for backup utilities but says the filesystem stores it without interpreting it. This result
is not evidence that iOS or iCloud backup completed, included the object, or has a current copy;
the method does not set the marker or perform a backup. Filesystem support and timestamp precision
may vary. The query opens the final entry without following symlinks, reads no file contents, and
retains B1's concurrent opened-parent directory-rename limit. Because it uses `fgetattrlist`, the
host must declare an applicable approved File Timestamp reason in `PrivacyInfo.xcprivacy` for actual
use. See the [B301 plan](../../PLAN_IOS_FILE_BACKUP_TIME_MARKER.md).

## File data-generation snapshot

`IosFiles::regular_file_data_generation_snapshot` returns an `IosFileDataGenerationSnapshot`
from one opened regular-file descriptor. Its `identity()` pair comes from `fstat`; the optional
`generation_count()` comes from `fgetattrlist` on that same descriptor. These are separate
syscalls, so the method makes no atomic cross-field snapshot claim. XNU documents equality
comparison only for the same filesystem object; zero is invalid and is returned for memory-mapped
files, so the accessor returns `None` for zero. The `(st_dev, st_ino)`-style pair is a point-in-time
identity, not a persistent identifier or protection from inode reuse. Do not treat the generation
count as a general content-change token or compare it across different identity pairs. Filesystems may
omit the attribute and then return `Unsupported`. This iOS-only query reads no content, accepts no
arbitrary URL, starts no security scope, and does not change portable `FileBackend` semantics. The
host must declare an applicable approved File Timestamp reason in `PrivacyInfo.xcprivacy` for
actual use. See the [B275 plan](../../PLAN_IOS_FILE_DATA_GENERATION.md).

## Raw data-protection-class code

`IosFiles::entry_data_protection_class_code` returns an `IosFileDataProtectionClassCode` for an
opened regular file or directory. Its `raw_value()` is the `u32` from
`ATTR_CMN_DATA_PROTECT_FLAGS`. Apple documents that field as a data-protection class, but does not
publish a numeric mapping to named levels. Callers may preserve or display the raw value only; do
not map it to a named level, compare it across objects or OS versions, or infer current/future data
access or security guarantees. The value is not an object identity or a cross-call stable-object
snapshot. The query opens the final entry without following a symlink, reads no file contents, and
may fail before the attribute query due to ordinary access or file-state rules. It uses
`fgetattrlist`, so the host must declare an applicable approved File Timestamp reason in
`PrivacyInfo.xcprivacy` for actual use. See the [B293 plan](../../PLAN_IOS_FILE_DATA_PROTECTION_CLASS_CODE.md).

## BSD file flags

`IosFiles::entry_bsd_file_flags` returns the raw `st_flags` bits for one non-symlink entry. The
`IosBsdFileFlags` value exposes known Apple masks such as `UF_IMMUTABLE`, `UF_APPEND`, and
`UF_HIDDEN`, preserves unknown bits through `bits()`, and supports mask checks with `contains()`.
Flags may explain a restriction or display hint, but are not a full access check; the query does
not change flags or add portable `FileBackend` behavior. See the [B115 plan](../../PLAN_IOS_BSD_FILE_FLAGS.md).

## Entry owner IDs

`IosFiles::entry_owner_ids` returns the numeric POSIX `st_uid`/`st_gid` pair for one non-symlink
entry. The values are fixed-width `u32`s, not account names or stable user identities, and do not
establish effective access. The query uses one no-follow metadata lookup, reads no contents, and
adds no portable `FileBackend` behavior. See the [B118 plan](../../PLAN_IOS_FILE_OWNER_IDS.md).

## Entry identity snapshot

`IosFiles::entry_identity_snapshot` returns `st_dev` and `st_ino` from one
`fstatat(..., AT_SYMLINK_NOFOLLOW)` call. The pair describes the final entry itself, including a
symlink rather than its target, and accepts files, directories, symlinks, and special entries. It
can help compare entries observed at about the same time, including hard-link names, but it is not
a persistent ID or open handle; inode values may be reused after removal, and separate queries can
race path mutation. The method reads no contents, starts no security scope, and does not reserve the
path for a later operation. See the [B99 plan](../../PLAN_IOS_FILE_IDENTITY.md).

## POSIX permission bits

`IosFiles::entry_posix_permission_bits` returns `st_mode & 0o7777` for one validated path, using
`fstatat(..., AT_SYMLINK_NOFOLLOW)`. The `u16` includes owner/group/other read, write, and execute
bits plus set-user-ID, set-group-ID, and sticky bits. A symlink returns its own raw bits. This is
metadata useful for diagnostics such as inspecting stored creation modes; it does not determine
effective access or guarantee that a later read/write will succeed. Callers must rely on the actual
operation result. The method reads no contents and adds no portable `FileBackend` operation. See
the [B103 plan](../../PLAN_IOS_FILE_PERMISSION_BITS.md).

`IosFiles::entry_effective_access` requests Apple's `ATTR_CMN_USERACCESS` value for one validated
entry. It reports the current process's effective-UID read, write, and execute/search mask through
`IosEntryEffectiveAccess::{allows_read, allows_write, allows_execute_or_search}`; `bits()` retains
the raw mask, including unnamed bits. For directories, read means list, write means add a child,
and execute means search. The final symlink is not followed; the query does not report access to
its target. Some volume formats do not support this attribute and return `Unsupported`. This is a
point-in-time OS permission report, not a full access check or guarantee of a later read, write, or
traversal; app sandbox policy, file protection, mount state, namespace races, and other checks may
still affect an operation. The query reads no contents and adds no portable `FileBackend`
operation, arbitrary URL support, permission prompt, or security-scope access. See the
[B170 plan](../../PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md).

## Regular-file hard-link count

`IosFiles::regular_file_hard_link_count` returns the `st_nlink` count for one regular file from
one `fstatat(..., AT_SYMLINK_NOFOLLOW)` call. It rejects a final symlink, directory, or special
entry as `InvalidInput`. A count greater than one reports multiple hard links at that instant, but
does not list aliases or prove that the count will remain unchanged; a later link, unlink, or path
replacement may race. This is a best-effort diagnostic, not an exclusivity guard for a write. It
reads no contents and changes no portable `FileBackend` behavior. See the [B105 plan](../../PLAN_IOS_FILE_LINK_COUNT.md).

`read` rejects a final symlink, `read_directory` lists a symlink as `FileKind::Other`, and
`exists` reports a final symlink as present without following it. `remove_file` unlinks a final
symlink itself; `remove_directory` does not follow one. Create-or-replace atomically replaces a
final symlink rather than following it. Filesystem names that contain a backslash or bytes that are
not valid UTF-8 cannot be represented by the portable `DirectoryEntry` contract and cause
`InvalidInput` in `read_directory`.

## Writes and errors

Create-or-replace writes to a same-directory temporary file and commits with POSIX `renameat`.
Create-new uses Darwin `renameatx_np(RENAME_EXCL)`. Replace-existing uses
`renameatx_np(RENAME_SWAP)` so a missing target is not created and the old regular file is replaced
at one atomic visibility point. Replace-existing requires an existing regular file: a missing
target returns `NotFound`; a final symlink, directory, or special entry returns `InvalidInput`.
The atomic path checks the final entry with `fstatat(..., AT_SYMLINK_NOFOLLOW)` before staging and
again immediately before `RENAME_SWAP`. The final type check and rename are separate syscalls, so
same-path native mutation through another handle is not serialized and can race the check. The
backend queries `NSURLVolumeSupportsExclusiveRenamingKey` and
`NSURLVolumeSupportsSwapRenamingKey` for each root before it accepts these extended rename modes.
If a volume reports no support, `RequireAtomic` returns `Unsupported` before target mutation;
`AllowNonAtomic` uses exclusive direct creation or open/truncate/write and reports
`NotGuaranteed`. Such a failed fallback write may leave a new partial file or partially changed
bytes, as the portable contract allows for non-atomic writes.
The `ReplaceExisting` fallback validates the opened descriptor as a regular file before truncation,
but concurrent native replacement or removal of the same path is not serialized; after such a race,
the descriptor can refer to an inode no longer named by that path.

Successful staged writes report `Atomic`; this means readers see the whole old file or whole new
file, not that bytes or directory metadata are durable after a crash. The backend does not call
`fsync`. After a successful `RENAME_SWAP`, it unlinks the old version at the staging name on a
best-effort basis; a cleanup failure does not undo or turn the committed write into an error and
can leave a hidden `.ios-files-*` entry. New files and staged replacement files pass mode `0600` to
`openat`; new directories pass mode `0700` to `mkdirat`. The process `umask` may remove permission
bits from either mode. A staged replacement installs the new staged inode and does not preserve the
replaced file's metadata. Parent directories are not created implicitly.

POSIX `errno` is preserved as `PlatformErrorCode`; common values map to the portable
`NotFound`, `AlreadyExists`, `PermissionDenied`, `InvalidInput`, `Unsupported`, or
`ResourceExhausted` category. Foundation directory lookup errors map to `Platform` and preserve
`NSError.code`; its error domain is not represented by the current portable error type.

## Cost and execution behavior

Methods are synchronous and can block on filesystem I/O. They start no callback, worker thread, or
async task; dropping `IosFiles` does not cancel a call already on the stack. `read` copies file
bytes into its returned `Vec`; `write` passes the caller's slice to POSIX writes without an
intermediate Rust byte buffer. Path components are copied into temporary C strings. Directory
entry names are copied from `readdir` into owned Rust `String` values. The sandbox `Files` facade
exposes no native root path or descriptor escape handle.

The crate uses `objc2` 0.6.5 and `objc2-foundation` 0.3.2. The sandbox backend uses Foundation error,
search-path, URL, value, and volume-metadata APIs plus private `libc` POSIX calls. Bookmark resolution
also uses `NSData`, `NSProcessInfo`, and typed `NSURL` bookmark APIs. The integrated crate enables
`block2`, `ios-runtime`, and Foundation file-coordination APIs for the separate `IosFileCoordinator`
extension; these are not needed by the sandbox methods. `IosResolvedBookmark` exposes only its
resolved caller URL. A minimal consumer of both app-data crates, built against the integrated
crate, was checked for device and simulator
imports. `otool -L` showed Foundation, CoreFoundation, `libobjc.A`, `libSystem.B`, and `libiconv.2`;
neither binary imports UIKit, Network, Swift, Python, or another capability framework. This link
probe used Xcode 26.6 with the iOS 26.5 SDK, below the planned Xcode 27.x baseline. It proves link
imports only; no simulator launch or live sandbox operation was performed.

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

`IosFiles::regular_file_allocated_blocks_512` returns the filesystem-reported `st_blocks` count
in 512-byte units for one regular file. This differs from logical file size; sparse files may
report fewer allocated blocks than their logical size implies. It is not a promise of exact
physical-device usage or exclusive allocation. The query uses one no-follow metadata lookup,
rejects final symlinks/directories/special entries, reads no contents, and changes no portable
`FileBackend` behavior. See the [B109 plan](../../PLAN_IOS_FILE_ALLOCATED_BLOCKS.md).

## Single-entry kind

`IosFiles::entry_kind` classifies one validated sandbox `AppPath` with the same `FileKind` values
used by `read_directory`: regular files are `File`, directories are `Directory`, and symbolic links
or special entries are `Other`. It inspects only the final entry with `fstatat` and
`AT_SYMLINK_NOFOLLOW`, so it reads no file contents and does not follow a final symlink. A missing
entry returns `NotFound`; parent traversal and metadata errors use the existing POSIX mapping. The
answer is point-in-time and does not reserve the path for a later operation. This is an iOS-only
helper, not a new portable `FileBackend` operation; it grants no URL or security-scope access and
keeps the existing concurrent parent-directory rename limit. See the [B93 plan](../../PLAN_IOS_ENTRY_KIND.md).

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

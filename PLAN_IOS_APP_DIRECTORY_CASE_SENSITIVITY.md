# B431: App-Directory Case-Sensitivity Snapshot

## Status

Implemented in `ios-files` as an iOS-only, read-only query on one retained semantic app-directory root. This does not add a portable `framework-files` operation or alter `AppPath` comparison/validation.

## API and contract

`IosFiles::app_directory_case_sensitivity(&self, directory: AppDirectory) -> Result<Option<bool>, FileError>` calls `fpathconf(root_fd, _PC_CASE_SENSITIVE)` on the retained Documents, Caches, Temporary, or Application Support descriptor.

- `0` returns `Some(false)`; `1` returns `Some(true)`.
- `-1` with `errno` unchanged returns `Some(true)`, not `None`. Legacy FSKit pathconf Boolean encoding uses `0` for false and `-1` for true. This selector is Boolean, not a numeric limit with a no-limit sentinel.
- `-1` with `EINVAL` returns `None`: Darwin pathconf documents `EINVAL` when an implementation does not associate a valid property with the selected file. XNU's rename path explicitly handles an unsupported `_PC_CASE_SENSITIVE` selector as an error.
- Other native errors use the existing POSIX error mapper. Any result other than `-1`, `0`, or `1` is `InvalidInput`.
- An `AppDirectory` that is not one of the four retained roots returns `Unsupported`.

The value describes the filesystem's case-sensitivity property at query time. It does not define Unicode normalization/collation, guarantee whether a particular pair of names collides, reserve a name, or guarantee a later create/rename. It does not establish support for case-sensitive names; B175 separately reports Foundation's cached `NSURLVolumeSupportsCaseSensitiveNamesKey` value.

## Evidence

- The installed iPhoneOS 26.5 SDK `sys/unistd.h` defines public `_PC_CASE_SENSITIVE` as `11` under Darwin extension declarations. It declares `long fpathconf(int, int)` without an explicit availability annotation. Locked `libc` 0.2.190 binds `_PC_CASE_SENSITIVE` and `fpathconf`; the package deployment floor remains iOS 10.0.
- Apple XNU's [`vfs_syscalls.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/vfs/vfs_syscalls.c#L9008-L9015) calls `VNOP_PATHCONF` and treats a nonzero `_PC_CASE_SENSITIVE` result as case-sensitive; an unsupported selector is handled as an error. The same source repeats that policy for rename at [lines 9111-9118](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/vfs/vfs_syscalls.c#L9111-L9118).
- Apple's open-source HFS implementation [`hfs_vnops.c`](https://github.com/apple-oss-distributions/hfs/blob/main/core/hfs_vnops.c#L5734-L5779) returns `1` when `HFS_CASE_SENSITIVE` is set, `0` otherwise, and `EINVAL` for an unrecognized pathconf selector.
- The legacy FSKit PathConf protocol documented Boolean pathconf values as `0` for false and `-1` for true. This legacy encoding is retained here as a valid true result. The installed current FSKit `FSVolume.PathConfOperations` header no longer exposes case sensitivity through that protocol; current `FSVolume.caseFormat` is a separate enum. FSKit is not a dependency of `ios-files`.
- Apple's [`vnop_advlock_args` documentation](https://developer.apple.com/documentation/kernel/vnop_advlock_args) identifies `_PC_CASE_SENSITIVE` as the filesystem case-sensitivity property. Apple's [`FSVolume.CaseFormat` documentation](https://developer.apple.com/documentation/fskit/fsvolume/caseformat) defines case-sensitive names as treating upper- and lowercase characters as distinct.
- Apple's archived [iOS `pathconf(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pathconf.2.html) documents the open-descriptor query, `-1`/`errno` distinction, and `EINVAL` for an unsupported property association. It does not list the newer `_PC_CASE_SENSITIVE` selector, so its generic no-limit sentinel is not used to interpret this Boolean selector.
- The query reads no file contents, performs no path re-resolution, and adds no framework, permission, Info.plist key, entitlement, or required-reason privacy API.

## Validation

Non-test validation is limited to locked/offline host, iOS device, and iOS Simulator checks; strict library Clippy; rustdoc; formatting; docs-check; and scoped diff-check. No tests, runtime filesystem queries, consumers, or link probes are part of this slice.

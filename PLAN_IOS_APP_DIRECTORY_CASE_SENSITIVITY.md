# B431: App-Directory Case-Sensitivity Snapshot Feasibility

## Disposition

GO for a narrow iOS-only, read-only query of the case-sensitivity property on one retained semantic app-directory root. This is a feasibility report only; no source API has been added.

## Proposed contract

`IosFiles::app_directory_case_sensitivity(&self, directory: AppDirectory) -> Result<Option<bool>, FileError>` would call `fpathconf(root_fd, _PC_CASE_SENSITIVE)` on the already-open Documents, Caches, Temporary, or Application Support descriptor.

- Return `Some(true)` for `1` and `Some(false)` for `0`.
- Return `None` when `fpathconf` returns `-1` without setting `errno`, preserving an indeterminate result.
- Map `EINVAL` for this valid property name to `Unsupported`; map other native failures through the existing POSIX error mapper. Map unexpected values to `InvalidInput`.
- The result describes the filesystem's case-sensitivity property for the retained root at query time. It does not define Unicode normalization/collation, guarantee that a specific pair of names collides, reserve a name, or guarantee a later create/rename. It does not change portable `AppPath` comparison or validation.
- This is distinct from B175's cached Foundation `NSURLVolumeSupportsCaseSensitiveNamesKey`, which reports whether a volume supports case-sensitive names. The proposed `fpathconf` query asks for the filesystem's case-sensitivity property; it must not be substituted with or inferred from B175's support value.

## Evidence

- The installed iPhoneOS 26.5 SDK `sys/unistd.h` defines public `_PC_CASE_SENSITIVE` as `11` under the Darwin extension declarations. `unistd.h` declares `long fpathconf(int, int)` without an explicit availability annotation. The package's existing iOS 10.0 deployment floor remains the applicable floor.
- Locked `libc` 0.2.190 binds `_PC_CASE_SENSITIVE` as `c_int` and exposes `fpathconf`; no dependency change is needed.
- Apple's [`vnop_advlock_args` documentation](https://developer.apple.com/documentation/kernel/vnop_advlock_args) describes the property as asking whether a filesystem is case-sensitive. Apple's [`FSVolume.CaseFormat` documentation](https://developer.apple.com/documentation/fskit/fsvolume/caseformat) defines case-sensitive names as treating upper- and lowercase characters as distinct. These references establish the property meaning, not a guarantee about any later namespace operation.
- Apple's archived [iOS `fpathconf(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fpathconf.2.html) documents use with an open file descriptor and the `-1`/`errno` distinction. The archived manual does not list this newer Darwin property; its result mapping must therefore be checked against the installed SDK/runtime contract before implementation.
- B175's [volume case-name support plan](PLAN_IOS_VOLUME_NAME_SUPPORT.md) covers Foundation support values cached at `IosFiles::new`; its contract is not a current `fpathconf` observation.
- `fpathconf` introduces no framework, permission, Info.plist key, entitlement, or required-reason privacy API. It reads no file contents and performs no path re-resolution.

## Implementation gate

Before source changes, confirm the Darwin `_PC_CASE_SENSITIVE` return-value convention and unsupported-filesystem behavior against an authoritative libc/kernel source or a compile-time API oracle that does not execute a query. Keep the implementation limited to the retained descriptor and the tri-state/error contract above; do not add a portable contract or alter `AppPath` semantics.

No tests, builds, runtime queries, consumers, or probes were run for this feasibility report.

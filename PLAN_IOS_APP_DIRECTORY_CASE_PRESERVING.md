# B439: App-Directory Case-Preservation Snapshot Audit

## Disposition

NO-GO for a second `ios-files` API. The descriptor query is technically bounded, but it would repeat B175's case-preserved-name volume Boolean without adding a distinct operation or caller contract.

## Candidate scope

The candidate was an iOS-only `IosFiles::app_directory_case_preserving_names(AppDirectory)` method that calls `fpathconf(root_fd, _PC_CASE_PRESERVING)` on the retained Documents, Caches, Temporary, or Application Support descriptor. Darwin defines this as a filesystem path property for case preservation during name creation and rename. A narrow implementation could return an optional Boolean and leave portable `AppPath` rules unchanged.

The query would be synchronous, read-only, descriptor-bound, and require no path re-resolution, Foundation object, permission prompt, entitlement, Info.plist key, or required-reason privacy API. Any returned property would remain point-in-time metadata, not a promise that a later create or rename succeeds.

## Evidence and overlap

- The installed iPhoneOS 26.5 SDK `sys/unistd.h` defines public Darwin `_PC_CASE_PRESERVING` as `12` under the same extension guard as `_PC_CASE_SENSITIVE`. `fpathconf` has no explicit availability annotation; the existing `ios-files` iOS 10.0 deployment floor applies. Locked `libc` exposes both the selector and `fpathconf`.
- Apple's open-source HFS [`hfs_vnops.c`](https://github.com/apple-oss-distributions/hfs/blob/main/core/hfs_vnops.c#L5734-L5779) returns `1` for `_PC_CASE_PRESERVING` and `EINVAL` for an unknown pathconf selector. XNU's [`vfs_syscalls.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/vfs/vfs_syscalls.c#L9170-L9180) identifies `_PC_CASE_PRESERVING` as a distinct filesystem property used with `_PC_CASE_SENSITIVE` for case-insensitive, case-preserving rename handling.
- Apple's [`NSURLVolumeSupportsCasePreservedNamesKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportscasepreservednameskey?language=objc) already gives a read-only Boolean for whether the volume supports case-preserved names. B175 reads and caches this value for each of the same four retained app-directory roots.
- A fresh `fpathconf` observation could differ in freshness from B175's construction-time cache, but no current `ios-files` operation consumes that distinction. The roots are app-container directories; callers already receive the same case-preservation support Boolean from B175, and a second query would not change path handling or operation guarantees.

## Result

Do not add source or alter the portable contract. Keep B175 as the case-preserved-name support snapshot. Reconsider only if a future operation needs the live pathconf property for a distinct, specified decision and can explain why the cached Foundation support value is insufficient.

No tests, builds, runtime filesystem queries, consumers, or probes were run for this audit.

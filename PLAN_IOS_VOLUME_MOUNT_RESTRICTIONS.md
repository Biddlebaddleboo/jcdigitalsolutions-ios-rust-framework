# B161: No-Go for Additional iOS Volume Mount-Restriction Flags

## Disposition

Do not add public `ios-files` booleans for `MNT_NOEXEC`, `MNT_NOSUID`, `MNT_NODEV`, `MNT_DONTBROWSE`, `MNT_QUOTA`, or other `f_flags` bits beyond B152's narrow read-only mount snapshot. The remaining flags report filesystem mount policy; they do not establish what this app may do with a particular sandbox path

## Candidate and limits

- A `fstatfs` snapshot is bound to the retained app-directory root and reports exported mount flags. It does not replace the platform's code-signing policy, sandbox access checks, file permissions, file protection, or the result of an attempted operation.
- `MNT_NOEXEC` is not an app-level executable-content permission query. A false bit does not authorize code loading or execution; a true bit does not describe all platform execution constraints.
- `MNT_NOSUID` and `MNT_NODEV` do not provide a useful application file-management capability for ordinary sandbox entries, and do not describe the permissions or behavior of an individual path.
- `MNT_DONTBROWSE` concerns whether a mounted filesystem is appropriate to expose in browsing contexts; it is not a property of the selected file or a user-visible access guarantee for the app's semantic directory.
- `MNT_QUOTA` only indicates a filesystem quota mount policy. It does not report this app's quota or remaining allowance; B137 is explicitly not an app quota value.

## Decision

Keep the additional bits private to the backend. Do not return the raw `f_flags` word or use these flags to predict whether a file operation will succeed. The only exposed mount-property query remains B152's `volume_is_read_only`, which has a direct, narrowly documented interpretation; actual file operations remain authoritative for app-level access.

## Platform evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) describes `f_flags` as exported mount flags and lists `MNT_RDONLY`, `MNT_NOEXEC`, `MNT_NOSUID`, `MNT_NODEV`, `MNT_DONTBROWSE`, and `MNT_QUOTA` with filesystem-level meanings.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` declares these constants. Their availability in the header establishes bit definitions, not an effective per-path or per-app authorization contract.
- This is a no-go report only. It adds no source/API, framework, dependency, permission, privacy-manifest reason, deployment-floor claim, or capability-matrix status change.

# B149: iOS Volume File-Node Counts No-Go

## Disposition

Do not expose `statfs.f_files` or `statfs.f_ffree` as app-data capacity methods. Apple defines them as total and free file nodes for the mounted filesystem, not as a per-app or per-container limit. The values do not establish whether a sandbox app can create another file

## Candidate and evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) defines `f_files` as total file nodes in the filesystem and `f_ffree` as free file nodes in the filesystem.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` and locked `libc` 0.2.189 Apple binding expose both fields as `u64` in `statfs`.
- `ios-files` opens app sandbox directories on their host volume. Its API does not create a separate filesystem, request inode quota, or control filesystem node allocation. The public `statfs(2)` contract does not say these volume-wide counters predict a sandbox app's file-creation result.
- `fstatfs` is already covered by B137's required-reason Disk Space privacy-manifest note; exposing raw node counters would add no separate user-facing operation.

## Why no implementation

The counts are filesystem diagnostics with no supported app-specific limit or actionable operation in this facade. A caller could misread `f_ffree` as a count of files its app can still create. Actual file creation remains the authoritative result; B149 adds no API or source claim

## Closure criteria

Revisit only if Apple documents a stable, app-visible per-container file-node budget or another supported user-facing operation that uses these volume-wide counts without implying a creation guarantee

## Validation

- No source, dependency, package, portable contract, guide API claim, or shared aggregate file changed.
- Apple's iOS `statfs(2)` manual, iPhoneOS 26.5 `sys/mount.h`, and locked `libc` 0.2.189 bindings were inspected.
- No tests, builds, probes, consumers, or live filesystem calls were run for this no-go

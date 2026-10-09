# B115: iOS BSD File-Flag Snapshot

## Status

`IosFiles::entry_bsd_file_flags` returns the raw `st_flags` bit set for one non-symlink entry.
Known Apple masks are named; unknown bits remain available, and no effective-access claim is made

## Objective

Expose a read-only iOS query for BSD file flags that can explain a stored file restriction or hint
without changing flags or interpreting the full access policy

## Contract

- `entry_bsd_file_flags(&self, path: AppPath<'_>) -> Result<IosBsdFileFlags, FileError>` uses the
  existing semantic `AppDirectory`, path validation, root descriptor, and descriptor-relative
  parent traversal.
- The final entry is inspected once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. Final symlinks return
  `InvalidInput`; regular files, directories, and special entries return that entry's `st_flags`.
- `IosBsdFileFlags::bits()` returns all raw `u32` bits. `contains()` checks a named mask. The type
  names Apple masks `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, `UF_OPAQUE`, `UF_COMPRESSED`,
  `UF_HIDDEN`, `SF_ARCHIVED`, `SF_IMMUTABLE`, and `SF_APPEND`.
- Flags are metadata, not an effective-access result. Some known flags can restrict changes or act
  as display hints, but POSIX mode bits, ACLs, sandbox policy, filesystem behavior, and races also
  affect whether an operation succeeds. B115 does not set, clear, or change flags.
- Missing final entries map through the existing POSIX error mapper to `NotFound`; parent,
  permission, and metadata errors preserve the existing mapped category and native code.
- The query is synchronous, reads no contents, retains no descriptor, follows no final symlink,
  accepts no absolute URL, starts no security scope, and grants no access. It has the existing
  concurrent native directory-rename containment limit.
- This is an iOS-only `IosFiles` API. It adds no flags concept to the portable
  `FileBackend`/`Files` contract and adds no dependency, framework, or native symbol.

## Static evidence

- Apple's iOS [`stat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html)
  defines `st_flags` as user-defined file flags and points to `chflags(2)` for the bit list. It
  also says `lstat` does not report file flags that belong to a symbolic link, so B115 rejects
  final symlinks.
- Apple's iOS [`chflags(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fchflags.2.html)
  describes `UF_NODUMP`, `UF_IMMUTABLE`, `UF_APPEND`, `UF_OPAQUE`, `UF_HIDDEN`, `SF_ARCHIVED`,
  `SF_IMMUTABLE`, and `SF_APPEND`; the installed iPhoneOS 26.5 `sys/stat.h` also defines
  `UF_COMPRESSED`.
- The SDK declares `struct stat.st_flags` as `__uint32_t` and the locked `libc` Apple binding
  exposes `stat.st_flags: u32` plus matching constants for each named mask.
- `fstatat` is available from iOS 8.0; the existing `ios-files` package floor is iOS 10.0. B115
  adds no deployment-floor requirement, permission prompt, Info.plist key, entitlement, dependency,
  framework, or native symbol.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy on both targets with `-- -D warnings` and
  `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check` and scoped `git diff --check`.
- No tests, live sandbox operation, link/import probe, consumer, or runtime action was run.

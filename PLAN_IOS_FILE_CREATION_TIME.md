# B101: iOS `st_birthtime` Creation-Time Snapshot — No-Go

## Disposition

No `st_birthtime`-based creation-time API is added. Darwin exposes a birth-time field in
`struct stat`, but that field does not have a reliable per-entry availability signal. Apple
documents that on filesystems without birth-time support, `st_birthtime` contains `ctime` instead.
Returning that field as creation time would mislabel a file-status-change time as a creation time

## Audited surface

- The installed iPhoneOS 26.5 SDK header
  `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/usr/include/sys/stat.h`
  declares `st_birthtimespec` in the 64-bit `stat` layout and comments it as the time of file
  creation/birth. `fstatat` is declared iOS 8.0+; the current `ios-files` package floor is iOS 10.0.
- The locked `libc` Apple binding exposes `stat.st_birthtime` and `stat.st_birthtime_nsec` in
  `libc-0.2.189/src/unix/bsd/apple/mod.rs`. Binding availability proves only that Rust can read the
  fields, not that every entry's filesystem supplies a true birth time.
- Apple's archived iOS [`lstat(2)` documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/lstat.2.html)
  defines `st_birthtime` as creation time, then states that on filesystems where birth time is
  unavailable the field holds `ctime`. The `struct stat` result has no separate support bit that
  lets this adapter distinguish those cases.
- `fstatat(..., AT_SYMLINK_NOFOLLOW)` could obtain the final entry's own pair under B1's path rules,
  but it cannot resolve the semantic ambiguity. Relabeling the field as “birth time or ctime” would
  not provide a dependable creation-time contract and would add little beyond the existing
  modification-time snapshot.

## Closure criteria

The `st_birthtime` no-go remains. B359 adds a separate `ATTR_CMN_CRTIME` query that first checks
`ATTR_VOL_ATTRIBUTES.validattr.commonattr` on the same opened file descriptor and returns
`Unsupported` when the volume does not advertise that attribute. This qualified attrlist path does
not infer from `st_birthtime`; its returned value is read/write metadata, a point-in-time
observation, and not immutable proof of the real-world creation event. See
[`PLAN_IOS_FILE_CREATION_TIME_ATTRIBUTE.md`](PLAN_IOS_FILE_CREATION_TIME_ATTRIBUTE.md)

## Scope and checks

This report adds no Rust API, package dependency, framework, root workspace/lockfile change, matrix
edit, or aggregate-doc update. No tests, builds, probes, consumers, or live filesystem calls were
run. `git diff --check` and `cargo +1.94.1 xtask docs-check` were used as static documentation gates

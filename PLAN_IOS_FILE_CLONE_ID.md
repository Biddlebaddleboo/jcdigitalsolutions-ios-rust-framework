# B211: iOS Regular-File Clone-ID Snapshot

## Scope

B211 adds `IosFiles::regular_file_clone_id_snapshot(AppPath) -> Result<IosFileCloneIdSnapshot, FileError>`.
The method opens one regular file through the backend's validated descriptor-relative path walk,
then requests `ATTR_CMNEXT_CLONEID` with `fgetattrlist` on the open descriptor. It returns the
opaque 64-bit value in a caller-owned Rust type. This is distinct from B208's file-level may-share
and full-share flags; it adds no portable `FileBackend` method and does not change B196 clone
behavior.

## API and binding evidence

The iPhoneOS 26.5 SDK defines `ATTR_CMNEXT_CLONEID` in the public `sys/attr.h` `forkattr` group and
declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability. The request uses
`FSOPT_ATTR_CMN_EXTENDED`, which is already bound with `ATTR_CMNEXT_CLONEID` by locked `libc`
0.2.190. The SDK provides no separate availability annotation for the extended-common attribute,
so no additional runtime floor is claimed.

Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2)
defines `ATTR_CMNEXT_CLONEID` as a `u_int64_t` uniquely identifying the associated file data stream
and says equal IDs are useful for finding pure clones. It also defines the attribute-buffer layout
as a leading `u_int32_t` byte length followed by the requested value, aligned to four bytes. The
implementation uses a fixed 12-byte buffer for this single attribute.

## Contract limits

- The method opens the final component with `O_NOFOLLOW | O_NONBLOCK`, checks the open descriptor
  is a regular file, then queries that descriptor. Later path replacement or unlink does not
  retarget the attribute query. The established concurrent rename of an already-open parent
  directory can still move that directory outside the selected root before the open.
- The ID may be compared as a current XNU clone-ID report. This API makes no persistence, across-
  write, cross-volume, content-hash, or change-token guarantee. Equality does not prove current
  block sharing or identify a particular B196 source path.
- `EINVAL` and `ENOTSUP` map to `Unsupported` when the volume does not support the attribute; the
  native code remains in `FileError`. A short or unexpected buffer length returns `InvalidInput`.
- The query reads no file contents, accepts no arbitrary URL, starts no security scope, and needs
  no usage-description key, permission, or entitlement. No dependency or root manifest change is
  needed.

## Validation

- Passed device and arm64 Simulator checks:
  `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy:
  `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
  and `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed rustdoc:
  `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 fmt --package ios-files -- --check`,
  `target/debug/xtask docs-check`, and tracked diff whitespace checks.
- No tests, linked probes, consumer execution, runtime queries, or live file operations are part
  of this slice.

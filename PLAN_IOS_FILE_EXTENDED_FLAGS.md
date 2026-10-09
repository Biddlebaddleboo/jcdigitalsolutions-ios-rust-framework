# B208: iOS Regular-File Extended-Flags Snapshot

## Scope

B208 adds `IosFiles::regular_file_extended_flags_snapshot(AppPath) -> Result<IosFileExtendedFlags, FileError>`.
The method opens one regular file through the backend's validated descriptor-relative path walk,
then requests `ATTR_CMNEXT_EXT_FLAGS` with `fgetattrlist` on the open file descriptor. It returns the
raw `u64` and named public SDK masks for clone sharing, extended-attribute presence, sparse regions,
and purgeability. Unknown bits remain available. This is an iOS-only point-in-time metadata query;
it adds no portable `FileBackend` method and does not change B196 clone behavior.

## API and binding evidence

The iPhoneOS 26.5 SDK declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability. Its public
`sys/attr.h` defines `ATTR_CMNEXT_EXT_FLAGS` in the `forkattr` bit field and
`FSOPT_ATTR_CMN_EXTENDED`; the request sets `forkattr` to `ATTR_CMNEXT_EXT_FLAGS` and passes the
extended-common option. The public `sys/stat.h` defines the documented `EF_*` masks; those macros
are not bound as Rust constants in locked `libc` 0.2.190, so the crate uses local Rust constants
with the exact SDK values. The same `libc` version binds `fgetattrlist`, `ATTR_CMNEXT_EXT_FLAGS`,
and `FSOPT_ATTR_CMN_EXTENDED`.

Apple's [XNU `getattrlist(2)` source documentation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2)
defines the descriptor form, attribute-buffer layout, and extended-flag meanings:

- `EF_MAY_SHARE_BLOCKS`: this file may share blocks with another file.
- `EF_SHARES_ALL_BLOCKS`: this file shares all blocks with another file; it is a full clone and
  implies `EF_MAY_SHARE_BLOCKS`.
- `EF_IS_SPARSE`: this file has at least one sparse region.
- `EF_NO_XATTRS`: this file has no extended attributes.
- `EF_IS_PURGEABLE`: the filesystem may delete this file when asked to free space.

The SDK gives no separate availability annotation for `ATTR_CMNEXT_EXT_FLAGS` or these masks, so
no additional runtime floor is claimed beyond the public iOS 3.0 `fgetattrlist` declaration.
Filesystem support is checked by the operation; `EINVAL` and `ENOTSUP` map to `Unsupported`.

## Contract limits

- The source is opened with `O_NOFOLLOW | O_NONBLOCK`, then checked as a regular file. The
  descriptor keeps the query bound to that inode if another task later replaces or unlinks its
  path. The existing concurrent rename of an already-open parent directory can still move that
  directory outside the selected root before the open; this API does not change that backend
  limit.
- The fixed buffer contains the documented leading `u32` length and one `u64` flag value. A
  truncated or unexpected size returns `InvalidInput`.
- The clone bits describe this file only. They do not identify a particular peer or prove that it
  shares blocks with a specific B196 source. Sharing can change after the snapshot; no stable
  allocation, space-saving, or performance claim is made.
- Sparse and purgeable bits are filesystem reports. They are not an allocated-size measure, a
  promise of future purge, or an instruction to delete data. The no-xattr bit does not add an xattr
  API.
- The query reads no contents, accepts no arbitrary URL, starts no security scope, and needs no
  usage-description key, permission, or entitlement. No dependency or root manifest change is
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

# B359: iOS Stored Creation-Time Attribute Snapshot

## Scope

B359 adds `IosFiles::regular_file_creation_time(&self, path: AppPath<'_>) -> Result<IosFileCreationTime, FileError>` as an iOS-only, descriptor-bound metadata query. It adds no creation-time concept to the portable `FileBackend`

The B101 no-go remains specific to reading `st_birthtime` from `struct stat`: on filesystems without birth-time support, Apple documents that field as containing `ctime`, and `stat` has no per-entry support bit. B359 does not read or infer from `st_birthtime`

## Support gate and value query

The method validates `AppPath`, traverses parents with the existing no-follow descriptor helper, and opens the final component with `O_NOFOLLOW`. It requires the opened descriptor to identify a regular file

On that descriptor, the first `fgetattrlist` request asks only for `ATTR_VOL_INFO | ATTR_VOL_ATTRIBUTES`. `ATTR_VOL_INFO` has no payload. The result buffer begins with its `u32` byte length, followed by `vol_attributes_attr_t`; the first `u32` in that payload is `validattr.commonattr` because `attribute_set_t.commonattr` is its first field. The parser checks the exact buffer and returned lengths before reading that mask. The method requires `validattr.commonattr & ATTR_CMN_CRTIME != 0`; if not, it returns `Unsupported`

The second `fgetattrlist` request asks for `ATTR_CMN_CRTIME` on the same descriptor. The existing checked timespec parser requires the exact result length, preserves signed seconds and the nanosecond field, maps an omitted value to `Unsupported`, and rejects malformed lengths or nanoseconds outside `0..1_000_000_000`

The volume support query and value query are separate calls, not one atomic metadata snapshot. The open descriptor pins the entry and volume across both requests, but the filesystem timestamp can change between them

## API semantics and limits

- XNU defines `ATTR_CMN_CRTIME` as a `timespec` for the time the filesystem object was created. The SDK marks this common attribute read/write through `setattrlist`; the result is mutable filesystem metadata, not immutable proof of the real-world creation event
- The value is point-in-time and may have coarser precision than one nanosecond. The method does not claim content identity, durability, or a reliable change token
- Missing entries map to `NotFound`; malformed paths to `InvalidPath`; final symlinks or non-regular entries to `InvalidInput`; absent volume support or omitted result to `Unsupported`; other errors use the existing POSIX mapping and retain native codes
- The method reads no target contents, accepts no arbitrary URL, starts no security scope, and retains B1's limit under a concurrent native rename of an already-open parent directory
- No permission prompt, Info.plist key, entitlement, framework, dependency, or native handle is added
- Apple lists `fgetattrlist` in the File Timestamp required-reason API category. A host app must declare an applicable approved reason in its final `PrivacyInfo.xcprivacy` for actual use

## SDK and binding evidence

- The installed iPhoneOS 26.5 SDK defines public `ATTR_CMN_CRTIME` as `0x00000200`, `ATTR_VOL_INFO` as `0x80000000`, and `ATTR_VOL_ATTRIBUTES` as `0x40000000` in `sys/attr.h`; it defines `vol_attributes_attr_t.validattr` and `attribute_set_t.commonattr`
- The SDK declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability and gives no separate availability annotation for these attribute bits. The `ios-files` deployment floor remains iOS 10.0
- Locked `libc` 0.2.190 binds `ATTR_CMN_CRTIME`, `ATTR_VOL_INFO`, `ATTR_VOL_ATTRIBUTES`, `vol_attributes_attr_t`, `attribute_set_t`, and `fgetattrlist`
- XNU `getattrlist(2)` describes `validattr` as the set of attributes supported by the volume-format implementation and defines `ATTR_CMN_CRTIME` as the object's creation-time `timespec`. XNU `vfs_attrlist.c` routes `fgetattrlist(fd, ...)` through the common attrlist implementation and returns volume attributes for the descriptor's mount

## Validation

- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 clippy --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios -- -D warnings`
- Passed `cargo +1.94.1 clippy --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim -- -D warnings`
- Passed `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
- Passed `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`
- Passed `cargo +1.94.1 fmt -p ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and `git diff --check`
- No tests, runtime filesystem calls, consumer builds, or link/import probes were run

## Sources

- [XNU `getattrlist(2)`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2)
- [XNU `vfs_attrlist.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/vfs/vfs_attrlist.c)
- [Apple `fgetattrlist` required-reason API category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype)

# B275: iOS Regular-File Data Generation Snapshot

## Scope

B275 adds `IosFiles::regular_file_data_generation_snapshot(&self, path: AppPath<'_>) -> Result<IosFileDataGenerationSnapshot, FileError>` for one regular sandbox file

The snapshot combines an `(st_dev, st_ino)` identity pair from `fstat` with an optional XNU data-generation count from `fgetattrlist` on the same opened descriptor. It adds no portable `FileBackend` method

## API and evidence

`IosFileDataGenerationSnapshot::identity()` returns `IosFileIdentitySnapshot`, with the device ID and inode number from the open descriptor. `generation_count()` returns a nonzero `u32`, or `None` when XNU reports zero

The installed iPhoneOS 26.5 SDK defines public `ATTR_CMN_GEN_COUNT` as `0x00080000` and `FSOPT_ATTR_CMN_EXTENDED` as `0x00000020` in `sys/attr.h`. Locked `libc` 0.2.190 binds both constants and `fgetattrlist`; its Apple `MetadataExt` binding supplies `st_dev` and `st_ino`. The SDK declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability and gives no separate attribute availability annotation, so B275 adds no higher API floor

Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_CMN_DEVID` as a `dev_t` equivalent to `st_dev`, `ATTR_CMN_FILEID` as a `u_int64_t` equivalent to `st_ino`, and `ATTR_CMN_GEN_COUNT` as a `u_int32_t` generation count for file data. B275 gets the identity pair from `MetadataExt` on the opened descriptor and requests only the generation count through `fgetattrlist`. The manual says an unchanged nonzero generation count for the same filesystem object indicates identical data, while the numeric value itself has no independent meaning. It also says zero is invalid and is returned for memory-mapped files. `ATTR_CMN_GEN_COUNT` requires `FSOPT_ATTR_CMN_EXTENDED`, which reinterprets `forkattr` bits as extended common attributes

The fixed 8-byte buffer contains the leading 4-byte length and 4-byte generation count. XNU skips unsupported attributes by default; a length-only response maps to `Unsupported`, while any other unexpected length maps to `InvalidInput`. The identity read and generation query use the same open descriptor, but are separate syscalls and make no atomic cross-field snapshot claim

## Contract limits

- The method validates `AppPath`, walks parents with the existing no-follow descriptor helper, opens the final entry with `O_NOFOLLOW | O_NONBLOCK`, and accepts regular files only. A final symlink or other entry kind returns `InvalidInput`
- The device ID and file ID form a point-in-time identity report for the opened file, not a persistent ID or protection from inode reuse. A path replacement after `openat` does not retarget either descriptor-based lookup
- Compare generation counts only when both snapshots have a nonzero count and their identity pairs match. XNU's same-object equality rule does not make this a general content-change token, durable version, or cross-volume identifier
- A zero generation count maps to `None`; XNU specifically reports zero for a memory-mapped file. Filesystem support may vary
- `EINVAL`, `ENOTSUP`, and omitted requested attributes map to `Unsupported`; malformed or oversized output maps to `InvalidInput`. Other native errors preserve the mapped `FileError` category and POSIX code
- The operation reads no contents, accepts no arbitrary URL, starts no security scope, and preserves the existing concurrent rename of an already-open parent-directory limit
- Apple lists `fgetattrlist` in the File Timestamp required-reason category. A host app that uses this method must declare an applicable approved reason in its final `PrivacyInfo.xcprivacy`
- No permission prompt, Info.plist key, entitlement, dependency, framework, workspace, or lock change is needed

## Validation

Passed these non-test checks

- `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 fmt --package ios-files -- --check`
- `target/debug/xtask docs-check`
- Scoped `git diff --check`

No tests, linked probes, consumer execution, runtime queries, or live file operations are in scope

# B266: iOS Regular-File Document ID Snapshot

## Scope

B266 adds `IosFiles::regular_file_document_id_snapshot(&self, path: AppPath<'_>) -> Result<IosFileDocumentIdSnapshot, FileError>` as an iOS-only file metadata query

The method opens one regular file through the validated descriptor-relative `AppPath` walk, then requests `ATTR_CMN_DOCUMENT_ID` through `fgetattrlist` on the open descriptor with `FSOPT_ATTR_CMN_EXTENDED`

## API and evidence

`IosFileDocumentIdSnapshot::document_id()` returns `Some(u32)` for a nonzero ID and `None` for XNU's invalid zero value

The iPhoneOS 26.5 SDK defines public `ATTR_CMN_DOCUMENT_ID` as `0x00100000` and `FSOPT_ATTR_CMN_EXTENDED` as `0x00000020` in `sys/attr.h`. Locked `libc` 0.2.190 binds both constants and `fgetattrlist`. The SDK declares `fgetattrlist` in `unistd.h` with iOS 3.0 availability and gives no separate availability annotation for this attribute, so no later attribute floor is claimed

Apple's [XNU `getattrlist(2)` manual](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/getattrlist.2) defines `ATTR_CMN_DOCUMENT_ID` as a `u_int32_t` assigned by the kernel to a document, used to track data regardless of moves, and sticky to its assigned path across safe saves. The manual says zero is invalid and that the request requires `FSOPT_ATTR_CMN_EXTENDED`. That option reinterprets `forkattr` bits as extended common attributes

The request uses a fixed 8-byte buffer: the 4-byte attribute-buffer length followed by the 4-byte document ID. XNU says unsupported attributes are skipped by default; if this sole value is omitted and the result contains only the length field, the method maps that result to `Unsupported`

## Contract limits

- The method opens the final component with `O_NOFOLLOW | O_NONBLOCK`, checks the open descriptor as a regular file, then queries that descriptor. A later path replacement or unlink does not retarget the query
- The current value is a point-in-time document/path token, not an inode, content hash, clone ID, link ID, cross-volume ID, or durable identity. XNU documents safe-save stickiness but the API makes no broader persistence or filesystem-support claim
- `EINVAL`, `ENOTSUP`, and an omitted attribute map to `Unsupported`; other native errors preserve the mapped `FileError` category and POSIX code. An unexpected buffer length maps to `InvalidInput`
- The operation reads no file contents, accepts no arbitrary URL, starts no security scope, and adds no permission, entitlement, or usage-description key. It does not alter portable `FileBackend` behavior
- The existing concurrent rename of an already-open parent-directory limit remains: another native handle can move that directory outside the selected root before the final open
- Apple lists `fgetattrlist` in the File Timestamp required-reason category. A host app that uses this query must declare an applicable approved reason in its final `PrivacyInfo.xcprivacy`
- No dependency, workspace, or lock change is needed

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

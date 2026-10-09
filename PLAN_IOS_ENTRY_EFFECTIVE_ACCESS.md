# B170: iOS Entry Effective-Access Snapshot

## Status

`IosFiles::entry_effective_access` reads Apple's `ATTR_CMN_USERACCESS` value for one no-follow
`AppPath`. It reports a point-in-time permission mask for the calling process's effective UID; it
does not predict whether a later operation will succeed

## API

- `IosFiles::entry_effective_access(&self, path: AppPath<'_>) -> Result<IosEntryEffectiveAccess, FileError>`
  uses the existing path validation and opens the parent path by descriptor-relative, no-follow
  traversal. It passes the final validated component to `getattrlistat` with `FSOPT_NOFOLLOW`.
- `IosEntryEffectiveAccess::allows_read`, `allows_write`, and `allows_execute_or_search` test the
  returned mask with `R_OK`, `W_OK`, and `X_OK`. `bits()` preserves the raw `u32`, including bits
  without a named accessor.
- The result describes the process's effective UID at query time. For a directory, the reported
  read bit means list, write means add a child entry, and execute means search. A final symlink is
  not followed; its mask does not describe the target.
- The result is not a full access decision or a promise that an operation will succeed. Sandbox
  policy, file protection, mount state, path mutation, and other OS checks may still affect a later
  operation. The existing open-parent concurrent-directory-rename limit remains.
- Some volume formats do not support `ATTR_CMN_USERACCESS`. The fixed request maps `EINVAL` to
  `Unsupported`; `ENOTSUP`/`EOPNOTSUPP` use the existing unsupported mapping. Missing entries and
  other lookup errors use the existing POSIX mapping. A malformed returned buffer maps to
  `InvalidInput`.
- This is an iOS-only metadata helper. It reads no file contents, accepts no arbitrary URL, starts
  no security scope, and adds no portable `FileBackend` operation, dependency, framework,
  permission, Info.plist key, entitlement, or required-reason privacy API.

## Platform evidence

- Apple's iOS [`getattrlist(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html)
  defines `ATTR_CMN_USERACCESS` as the effective permissions of the calling process's effective
  UID, and says to test read/write/execute with `R_OK`, `W_OK`, and `X_OK`. It documents
  `FSOPT_NOFOLLOW` for a final path component and a leading `u_int32_t` buffer length followed by
  requested attributes in documented order. It also says not all volumes support every attribute
  and documents `EINVAL` for an unsupported attribute.
- The installed iPhoneOS 26.5 SDK's `sys/attr.h` declares `ATTR_CMN_USERACCESS` and
  `FSOPT_NOFOLLOW`; `unistd.h` declares `getattrlistat` with iOS 8.0 availability. The package
  baseline is iOS 10.0, so this adds no higher deployment-floor requirement.
- Locked `libc` 0.2.190 exposes `libc::attrlist`, `ATTR_BIT_MAP_COUNT`, `ATTR_CMN_USERACCESS`,
  `FSOPT_NOFOLLOW`, `R_OK`/`W_OK`/`X_OK`, and `getattrlistat` for Apple targets.
- The method does not request volume attributes or use `fstatfs`; it adds no Disk Space
  required-reason API. No prompt, capability entitlement, or usage-description key applies.

## Validation

- Passed `cargo fmt --package ios-files -- --check` and `git diff --check`.
- Passed locked offline `cargo +1.94.1 check -p ios-files` and strict Clippy with `-- -D warnings`
  for `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`
  and `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`.
- No tests, consumers, probes, or live filesystem operations were run. Checks used Rust/Cargo
  1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5; local Xcode is below
  the repo's required Xcode 27.x baseline.

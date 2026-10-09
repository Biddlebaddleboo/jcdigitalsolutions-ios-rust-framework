# B296: iOS Detailed Entry Object Kind

## Status

`IosFiles::entry_object_kind` returns a detailed, point-in-time classification of one validated
sandbox entry. It does not open or follow the final entry, and it does not expand portable
`FileKind`

## Contract

- `entry_object_kind(&self, path: AppPath<'_>) -> Result<IosEntryObjectKind, FileError>` uses the
  existing `AppDirectory`, `AppPath` validation, root descriptor, and descriptor-relative parent
  traversal.
- The method reads `st_mode & S_IFMT` once with `fstatat(..., AT_SYMLINK_NOFOLLOW)`. It maps
  regular files to `File`, directories to `Directory`, symbolic links to `Symlink`, and POSIX FIFO,
  socket, block-device, and character-device entries to their matching variants. Any other type
  preserves the raw mode-type bits in `Unknown(u32)`.
- It does not open the final entry, so FIFO classification does not block on a FIFO open. It reads
  no file contents and does not follow a final symbolic link.
- The result is a point-in-time observation, not a reservation or guarantee for a later operation.
  It retains B1's limit: a concurrent rename of an opened parent directory can weaken path
  containment. Parent traversal, missing-entry, permission, and metadata errors use the existing
  `FileError` mapping.
- This is an inherent iOS `IosFiles` API. It does not alter portable `FileKind`,
  `FileBackend`/`Files`, `AppPath`, or `read_directory` semantics. It adds no dependency, framework,
  permission prompt, usage-description key, entitlement, URL access, or security scope.

## Platform and privacy evidence

- In the installed iPhoneOS 26.5 SDK, `sys/_types/_s_ifmt.h` defines `S_IFMT`, `S_IFIFO`,
  `S_IFCHR`, `S_IFDIR`, `S_IFBLK`, `S_IFREG`, `S_IFLNK`, and `S_IFSOCK`; it also defines the
  obsolete `S_IFWHT`, which remains represented by `Unknown(raw_type_bits)`. `sys/stat.h:393`
  declares `fstatat` available from iOS 8.0. The existing `ios-files` package floor is iOS 10.0,
  so B296 adds no deployment-floor requirement.
- The locked `libc` Apple binding exposes `fstatat`, `AT_SYMLINK_NOFOLLOW`, and the POSIX mode
  constants used by this mapping. The implementation reuses the existing no-follow metadata
  helper and adds no native symbol or crate dependency.
- Apple lists `fstatat` in the [File Timestamp required-reason API
  category](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype).
  A host app that uses this method must declare an applicable approved reason in its final
  `PrivacyInfo.xcprivacy`; this library does not choose a reason for the host.

## Validation

- Do not add or run tests, use a live app container, execute consumers, or run link/import probes.
- Passed `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
  and the same command for `aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 clippy --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios -- -D warnings`
  and the same command for `aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
  and the same command for `aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 fmt -p ios-files -- --check`, `cargo +1.94.1 xtask docs-check`,
  `git diff --check`, and a trailing-whitespace scan of this new plan.
- No tests, live filesystem call, consumer, link/import probe, or runtime action was run. The
  installed SDK is iPhoneOS/iPhoneSimulator 26.5 under Xcode 26.6 build `17F113`; that Xcode is
  below the repository's Xcode 27.x baseline.

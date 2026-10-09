# iOS app-directory long-name truncation

## Scope and result

B446 adds `IosFiles::app_directory_truncates_long_names(AppDirectory)` and a guard in the
existing `AppPath` walk. The property is descriptor-bound to the retained Documents, Caches,
Temporary, or Application Support root. No portable `FileBackend` type or operation changes.

Apple's `fpathconf(2)` page defines `_PC_NO_TRUNC` as `1` when names longer than
`KERN_NAME_MAX` are truncated and `0` when they are not. This follows the Darwin selector's
documented behavior, not an inference from the selector name. Apple's current
`FSVolume.PathConfOperations.truncatesLongNames` describes the same true-means-truncate property.
The SDK constant is `_PC_NO_TRUNC = 8`; locked `libc` 0.2.190 binds it.

`app_directory_truncates_long_names` returns `Some(true)` for a native result of `1` and for the
legacy FSKit Boolean `true` encoding of `-1` with unchanged `errno`; it returns `Some(false)` for
`0`, `None` for `EINVAL` when the filesystem does not associate the property with the descriptor,
and maps other errors. This is a point-in-time query, not a guarantee about later namespace or
filesystem state.

## Path-operation guard

Before each `openat` component and final `AppPath` leaf operation, `ios-files` queries
`_PC_NO_TRUNC` on that already-open parent descriptor:

- `0` means no truncation is reported; the backend proceeds and lets the actual syscall return its
  native `ENAMETOOLONG` result for an overlong component.
- `1`, or the legacy errno-free `-1` Boolean-true encoding, means truncation may occur; the backend
  then queries `_PC_NAME_MAX` on the same descriptor and returns `ENAMETOOLONG` if this component
  exceeds that byte limit.
- An unsupported `_PC_NO_TRUNC` association returns `Unsupported` because the backend cannot
  preserve the exact component name. If truncation is reported but `_PC_NAME_MAX` is unsupported,
  it also returns `Unsupported` because it cannot determine whether this component would change.
- Other pathconf errors preserve the existing POSIX error mapping.

This check applies to every component as the traversal advances through retained/opened directory
descriptors, including the final leaf. It does not normalize names, define Unicode collation, add
symlink following, or remove the existing concurrent opened-parent rename limitation. A native
operation remains authoritative for all other failures and races. The public snapshot reports only
the retained root; the path guard re-queries the specific opened parent for nested paths.

The selector and `fpathconf` add no Info.plist key, entitlement, or required-reason privacy API.
The operation reads no file contents and accepts no arbitrary URL.

## Evidence

- Installed Xcode iPhoneOS 26.5 SDK: `usr/include/sys/unistd.h` defines `_PC_NO_TRUNC` as `8`.
- Locked binding: `libc-0.2.190/src/unix/bsd/apple/mod.rs` defines `libc::_PC_NO_TRUNC` as `8`.
- Apple archived [`fpathconf(2)`](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fpathconf.2.html) documents `1` for truncating names and `0` for names preserved with the native overlong-name error. It documents `EINVAL` when a filesystem has no association for the selector.
- Apple's current [FSVolume.PathConfOperations.truncatesLongNames](https://developer.apple.com/documentation/fskit/fsvolume/pathconfoperations/truncateslongnames) defines `true` as truncation to `maximumNameLength` and `false` as an `ENAMETOOLONG` error.
- `crates/framework-files/src/lib.rs` validates lexical path form but does not cap component length. `ios-files` passes each UTF-8 component unchanged to descriptor-relative `openat`, `mkdirat`, `unlinkat`, and related operations. The guard prevents a filesystem that reports truncation from redirecting an overlong component.

## Validation

Static-only gates; no tests, runtime filesystem calls, link probes, or consumer binaries are run.
The following focused commands passed:

- `cargo +1.94.1 check --locked --offline -p ios-files`
- `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --lib -p ios-files --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --lib -p ios-files --target aarch64-apple-ios-sim -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo +1.94.1 doc --locked --offline --lib -p ios-files --target aarch64-apple-ios --no-deps`
- `cargo +1.94.1 fmt --package ios-files -- --check`
- `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`
- `git diff --check -- platform/ios/ios-files/src/lib.rs PLAN_IOS_APP_DATA.md PLAN_IOS_APP_DIRECTORY_NAME_TRUNCATION.md docs/ios/files.md`

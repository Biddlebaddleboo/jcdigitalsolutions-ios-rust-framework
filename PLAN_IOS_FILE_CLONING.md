# B193/B196: iOS Regular-File Clone

## B193 feasibility finding

`fclonefileat` provides a bounded native operation that creates one copy-on-write clone from an
open source descriptor into a destination directory. It is useful for a caller that needs a
second sandbox file without a Rust payload-sized `Vec` or byte stream. B193 found the public
operation and its `AppPath` path for a narrow iOS-only extension. B196 implements that extension;
this adds no portable `FileBackend` operation or matrix status.

Apple's iPhoneOS 26.5 `sys/clonefile.h` declares `fclonefileat` from iOS 10.0 and defines
`CLONE_NOFOLLOW_ANY` (`0x0008`) and `CLONE_RESOLVE_BENEATH` (`0x0010`). Locked `libc` exposes the
`fclonefileat` declaration but not those public header macros, so the iOS crate defines local Rust
constants with the exact values from `sys/clonefile.h`. Apple's [XNU `clonefile(2)` source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/man/man2/clonefile.2)
documents copy-on-write data sharing, destination absence, descriptor source semantics, path flags,
expected all-or-none atomic creation, `ENOTSUP` for unsupported filesystems, and `EXDEV` for files
on different filesystems. Apple's [APFS API guide](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/ToolsandAPIs/ToolsandAPIs.html)
lists the public clone calls.

## B196 implementation

`IosFiles::clone_regular_file(source, destination)` takes two validated `AppPath` values. It
resolves each parent by the existing component-by-component no-follow descriptor traversal. It
opens the source final component with `O_NOFOLLOW | O_NONBLOCK`, verifies the open descriptor is a
regular file, and passes that descriptor plus the destination-parent descriptor to `fclonefileat`.
The destination argument is one validated final component. The call always passes
`CLONE_NOFOLLOW_ANY | CLONE_RESOLVE_BENEATH`; it has no weaker fallback. The destination must not
exist and is never replaced. A final source symlink is not followed; a non-regular source returns
`InvalidInput`.

On success the new file has independent file identity, with data blocks that may be shared with
the source by copy-on-write. Later writes remain private to each file. A later overwrite can still
fail with `ENOSPC`. The syscall is expected to create the full destination or no destination; this
is not a crash-durability promise. No performance, storage-space, or future-write guarantee is
made.

Native file attributes and extended attributes follow XNU clone semantics. The destination uses
the destination parent ACL because `CLONE_ACL` is not set. Native owner handling applies, and the
kernel clears setuid/setgid bits on regular files. The operation is not a byte-only copy and does
not serialize concurrent source writes. It takes no URL, provider path, security-scoped URL, or
file-provider lifecycle.

`ENOTSUP` is `Unsupported`, `EEXIST` is `AlreadyExists`, `ENOENT` is `NotFound`, and `EXDEV` plus
other native codes use the existing `FileError` POSIX mapping with the code preserved. The call
does not preflight `NSURLVolumeSupportsFileCloningKey`; a volume property is not a promise for a
particular clone request. Distinct `AppDirectory` roots are allowed; the syscall decides whether
the two opened entries share a filesystem.

The operation inherits the backend's established race limit: a concurrent native rename can move
an already-open parent directory outside the selected root while its descriptor remains valid.
Source and destination namespace changes are not serialized. The API adds no dependency,
Info.plist key, permission, or entitlement. `fclonefileat` has an iOS 10.0 header floor. The SDK
header gives no separate availability annotation for the two flag macros; the implementation
always passes them and surfaces a native error if the OS rejects them. No runtime floor for those
flags is asserted by the compile-only checks.

## Validation

- Passed device and arm64 Simulator checks:
  `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy for device and Simulator:
  `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
  and `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed rustdoc for device and Simulator:
  `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`.
- Passed `rustfmt --edition 2024 --check platform/ios/ios-files/src/lib.rs`,
  `target/debug/xtask docs-check`, and tracked diff whitespace checks.
- No tests, linked probes, consumer execution, runtime queries, or live file operations were run.

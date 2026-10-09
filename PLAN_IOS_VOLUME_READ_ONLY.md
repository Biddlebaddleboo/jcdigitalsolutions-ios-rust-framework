# B152: iOS Volume Read-Only Mount Snapshot

## Disposition

Add `IosFiles::volume_is_read_only` as a one-shot query of the volume mount flag for one retained semantic app-directory root. It is useful as diagnostic context when a filesystem reports that its mount is read-only. It is not an effective app write-access or path-writability claim

## API

- `volume_is_read_only(&self, directory: AppDirectory) -> Result<bool, FileError>` calls `fstatfs` on the already-open descriptor for Documents, Caches, Temporary, or Application Support.
- Return whether `f_flags & MNT_RDONLY` is nonzero. The method uses libc's Apple `statfs` binding and `MNT_RDONLY` constant; it does not resolve a URL or inspect file contents.
- Return `Unsupported` for an unsupported semantic directory and use the existing POSIX error mapper if `fstatfs` fails.
- The value describes whether the containing volume is mounted read-only at query time. A `false` result does not mean this app can write a path: sandbox policy, directory permissions, file protection, BSD file flags, quota, or available space can still reject a write. The result is not a reservation or guarantee, and an actual write remains authoritative.
- This is an additive iOS-only API. It adds no portable `FileBackend` operation, framework, dependency, permission, usage-description key, entitlement, or explicit deployment-floor requirement.

## Privacy and platform evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) documents `fstatfs` and `struct statfs`.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` describes `f_flags` as the exported mount flags and defines `MNT_RDONLY` as `0x00000001` (“read only filesystem”). The declaration has no explicit `__IOS_AVAILABLE` annotation, so this slice adds no explicit API-floor requirement above `ios-files`' iOS 10.0 baseline.
- Locked `libc` 0.2.189 exposes Apple `statfs::f_flags: u32`, `fstatfs`, and `MNT_RDONLY: c_int = 0x00000001` through its BSD Apple target bindings.
- Apple lists `fstatfs` in the required-reason Disk Space category. The host app or SDK must declare a privacy-manifest reason matching actual behavior and comply with the reason's limits; B137 records the supported reason guidance. This read-only-mount diagnostic does not select a reason for the host app.

## Validation

- Do not add or run tests, execute consumers or probes, or perform live filesystem queries.
- Run `cargo fmt --package ios-files -- --check`.
- Run locked offline `cargo +1.94.1 check` and strict Clippy for `ios-files` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Run iOS-target rustdoc, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.

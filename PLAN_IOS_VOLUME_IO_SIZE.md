# B155: iOS Volume Optimal-I/O-Size Hint

## Disposition

Expose the filesystem-reported `f_iosize` for one retained semantic app-directory root as a fixed-width byte count. Apple documents it as the optimal transfer block size, which can inform caller-owned buffered I/O sizing. It is only a hint; it does not promise alignment, a workload-specific optimum, or better performance

## API

- `IosFiles::volume_optimal_io_block_size_bytes(&self, directory: AppDirectory) -> Result<u32, FileError>` calls `fstatfs` on the already-open descriptor for Documents, Caches, Temporary, or Application Support.
- Convert the signed `f_iosize` to `u32`. Return `InvalidInput` if it is zero or negative; map a failed `fstatfs` call through the existing POSIX error mapper and return `Unsupported` for an unsupported semantic directory.
- The value describes the containing volume at query time. It is not an alignment constraint, required transfer size, guarantee for any file, or claim of faster I/O when used. Callers must still handle ordinary short reads/writes and errors.
- This additive API reads no file contents, accepts no arbitrary URL, starts no security scope, and adds no portable `FileBackend` operation, framework, dependency, permission, usage-description key, entitlement, or explicit deployment-floor requirement.

## Privacy and platform evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) defines `f_iosize` as the optimal transfer block size.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` declares `fstatfs` and exposes `f_iosize` as a signed 32-bit field in `struct statfs`; the declaration has no explicit `__IOS_AVAILABLE` annotation. This adds no explicit API-floor requirement above `ios-files`' iOS 10.0 baseline.
- Locked `libc` 0.2.189 exposes Apple `statfs::f_iosize: i32` and `fstatfs` for iOS targets.
- Apple lists `fstatfs` in the required-reason Disk Space category. The host app or SDK must declare a privacy-manifest reason matching actual behavior and comply with its limits; B137 records the host reason guidance. This method does not select a reason for the host.

## Validation

- Do not add or run tests, execute consumers or probes, or perform live filesystem queries.
- Run `cargo fmt --package ios-files -- --check`.
- Run locked offline `cargo +1.94.1 check` and strict Clippy for `ios-files` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Run iOS-target rustdoc, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.

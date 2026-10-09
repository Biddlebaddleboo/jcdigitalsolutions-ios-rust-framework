# B146: iOS Total Volume-Capacity Snapshot

## Disposition

Add `IosFiles::volume_total_capacity_bytes` as the checked filesystem-reported byte capacity for the mounted volume that holds one retained semantic app-directory root. It can pair with B137 for an on-device volume-capacity display. It is not physical device capacity or an app-container quota

## API

- `volume_total_capacity_bytes(&self, directory: AppDirectory) -> Result<u64, FileError>` calls `fstatfs` on the already-open descriptor for Documents, Caches, Temporary, or Application Support.
- Return checked `f_blocks * f_bsize`. Apple defines `f_blocks` as total data blocks in the filesystem and `f_bsize` as the fundamental filesystem block size. Reject a zero block size as `InvalidInput`; map POSIX errors through the existing mapper; report product overflow as `ResourceExhausted`.
- The result is a filesystem-reported snapshot for one mounted volume. It does not describe physical device capacity, this app's quota, this directory's allocated bytes, or bytes reserved for this app. It is not guaranteed to stay in sync with a later B137 available-space query.
- Query the retained root descriptor; do not re-resolve a URL. The call is synchronous, reads no file contents, accepts no arbitrary URL, starts no security scope, and adds no portable `FileBackend` operation.
- Apple lists `fstatfs` in the required-reason Disk Space category. The host app/SDK privacy manifest must contain a valid reason matching actual use, as detailed in [B137](PLAN_IOS_VOLUME_CAPACITY.md). This package does not add or select the host manifest reason.

## Utility and limits

An app that displays a capacity bar for the mounted volume can pair the total `f_blocks * f_bsize` snapshot with B137's non-superuser-available `f_bavail * f_bsize` snapshot. The values can race independently, are not app-specific, and must not be labeled as total physical device storage or application quota.

## Platform evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) defines `f_blocks` as total data blocks in the filesystem and `f_bsize` as its fundamental block size.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` declares `fstatfs(int, struct statfs *)` without an explicit `__IOS_AVAILABLE` annotation; its `statfs` layout exposes `f_blocks`, `f_bsize`, and `f_bavail`.
- Locked `libc` 0.2.189 exposes the Apple `statfs` fields as `f_blocks: u64` and `f_bsize: u32`, plus `fstatfs`.
- Apple's [`NSPrivacyAccessedAPIType` documentation](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype) lists `fstatfs` in `NSPrivacyAccessedAPICategoryDiskSpace`; the B137 plan records approved reasons and host obligations.
- No new framework, dependency, permission, usage-description key, entitlement, portable contract, or deployment-floor requirement is added.

## Validation

- Do not add or run tests, execute a consumer, run a linked probe, or make a live filesystem capacity query.
- Run device and arm64 Simulator `cargo +1.94.1 check --locked --offline -p ios-files`.
- Run strict Clippy for the same targets with `-- -D warnings`.
- Run iOS-target rustdoc with `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Run `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.

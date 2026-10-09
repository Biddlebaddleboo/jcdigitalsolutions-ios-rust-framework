# B137: iOS Volume Available-Capacity Snapshot

## Disposition

Add `IosFiles::volume_available_capacity_bytes` as a read-only snapshot of the filesystem-wide bytes represented by `f_bavail * f_bsize` for one retained semantic app-directory root. This is useful for on-device storage display or a caller's low-space policy; it is not app-specific capacity and does not reserve space

## API

- `volume_available_capacity_bytes(&self, directory: AppDirectory) -> Result<u64, FileError>` calls `fstatfs` on the already-open descriptor for Documents, Caches, Temporary, or Application Support.
- Return the checked product of `f_bavail` and `f_bsize`. Apple defines `f_bavail` as free blocks available to non-superusers and `f_bsize` as the fundamental filesystem block size. Reject zero block size as `InvalidInput`; map POSIX failures through the existing error mapper; report multiplication overflow as `ResourceExhausted`.
- The result is a point-in-time volume snapshot. It is not an app-container quota, per-directory allocation, reservation, estimate of a particular file's future cost, or guarantee that a later write will succeed. Other processes and system activity can change it before use.
- Query the retained root descriptor rather than re-resolve a URL. This ties the snapshot to the same opened root volume used by other methods and avoids a new path lookup.
- The method is synchronous, reads no file contents, accepts no arbitrary URL, starts no security scope, and adds no portable `FileBackend` operation.

## Required privacy reason

Apple lists `fstatfs` as a required-reason API in the `NSPrivacyAccessedAPICategoryDiskSpace` category. This package cannot choose one reason for every host app. The final app or SDK privacy manifest must include a reason that matches actual use:

- `85F4.1` is for displaying disk-space information to the person. Apple says information accessed for this reason, or derived from it, may not be sent off-device.
- `E174.1` is for a sufficient/low-space check where app behavior changes observably to the person. Apple says the information may not be sent off-device, with a narrow exception for avoiding a server download when space is insufficient.
- Apple also lists `7D9E.1` for disk-space information in an optional bug report chosen by the person and `B728.1` for the specified health-research app case. The library does not claim either use.

The app/SDK integration owner must add the appropriate `PrivacyInfo.xcprivacy` entry. A missing reason is not a permission prompt, Info.plist usage key, or entitlement issue. Do not collect the value for fingerprinting or transmit it beyond the selected reason's limits

## Platform evidence

- Apple's iOS [`statfs(2)` manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/statfs.2.html) defines `f_bsize` as the fundamental filesystem block size and `f_bavail` as free blocks available to non-superusers.
- The installed iPhoneOS 26.5 SDK `sys/mount.h` declares `fstatfs(int, struct statfs *)` without an explicit `__IOS_AVAILABLE` annotation; its `statfs` layout exposes the same capacity fields. This adds no explicit deployment-floor requirement above `ios-files`' iOS 10.0 baseline.
- Locked `libc` 0.2.189 exposes Apple `fstatfs` and `statfs` with `f_bsize: u32` and `f_bavail: u64`.
- Apple's [`NSPrivacyAccessedAPIType` documentation](https://developer.apple.com/documentation/bundleresources/app-privacy-configuration/nsprivacyaccessedapitypes/nsprivacyaccessedapitype) lists `fstatfs` in the Disk Space required-reason category and defines approved reasons. Apple's [required-reason API guide](https://developer.apple.com/documentation/bundleresources/describing-use-of-required-reason-api) says apps must declare covered API use in a privacy manifest; App Store Connect rejects submissions without required-reason declarations.
- Apple's [`NSURLVolumeAvailableCapacityKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumeavailablecapacitykey) also classifies the Foundation capacity key as a required-reason API. B137 uses `fstatfs` on an already-open descriptor instead; this avoids URL re-resolution but does not avoid the Disk Space privacy-manifest requirement.
- No new framework, dependency, permission, usage-description key, entitlement, portable contract, or root manifest change is made by this slice.

## Validation

- Do not add or run tests, execute a consumer, run a linked probe, or make a live filesystem capacity query.
- Run device and arm64 Simulator `cargo +1.94.1 check --locked --offline -p ios-files`.
- Run strict Clippy for the same targets with `-- -D warnings`.
- Run iOS-target rustdoc with `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Run `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`, and scoped `git diff --check`.

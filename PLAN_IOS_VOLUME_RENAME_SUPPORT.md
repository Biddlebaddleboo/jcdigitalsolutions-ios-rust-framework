# B172: iOS Volume Rename-Option Support Snapshot

## Status

`IosFiles::volume_rename_support_snapshot` exposes the Foundation rename-option support values
that `IosFiles::new` already reads for each retained semantic app-directory root. It preserves an
unreported value as `None`; it does not promise that a future rename will succeed

## API

- `IosFiles::volume_rename_support_snapshot(AppDirectory)` returns an
  `IosVolumeRenameSupportSnapshot` for Documents, Caches, Temporary, or Application Support.
- `IosVolumeRenameSupportSnapshot::exclusive_rename_supported()` returns `Some(true)` or
  `Some(false)` when Foundation provides `NSURLVolumeSupportsExclusiveRenamingKey`; `None` means
  the key query failed, returned no value, or returned a value that was not an `NSNumber`.
- `IosVolumeRenameSupportSnapshot::swap_rename_supported()` preserves the same three states for
  `NSURLVolumeSupportsSwapRenamingKey`.
- The values are captured when `IosFiles::new` resolves its roots. This method returns that cache;
  it does not query Foundation again. A caller can recreate `IosFiles` to obtain a fresh snapshot.
- Apple defines the keys as read-only Boolean `NSNumber` values for volume support of
  `renamex_np(2)`'s `RENAME_EXCL` and `RENAME_SWAP` options. A true value only reports the volume
  capability; it does not establish path permissions, serialize namespace changes, or guarantee a
  later operation. The actual write/rename result remains authoritative.
- The existing `FileBackend` path maps `None` to its prior conservative `false` behavior. This
  additive iOS-only API changes no portable contract, file operation, dependency, framework,
  permission, Info.plist key, entitlement, or required-reason privacy API.

## Platform evidence

- Apple's [`NSURLVolumeSupportsExclusiveRenamingKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportsexclusiverenamingkey?language=objc)
  defines the value as volume support for `renamex_np(2)` with `RENAME_EXCL`, returned as a
  read-only Boolean `NSNumber`.
- Apple's [`NSURLVolumeSupportsSwapRenamingKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportsswaprenamingkey?language=objc)
  defines the value as volume support for `renamex_np(2)` with `RENAME_SWAP`, returned as a
  read-only Boolean `NSNumber`.
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` declares both keys available from iOS
  10.0. The package's existing iOS 10.0 baseline is unchanged.
- The existing `objc2-foundation` dependency and `NSURL` feature already provide the URL resource
  query used by `IosFiles::new`; B172 adds no dependency or native API import.

## Validation

- Passed `cargo fmt --package ios-files -- --check` and `git diff --check`.
- Passed locked offline `cargo +1.94.1 check -p ios-files` and strict Clippy with `-- -D warnings`
  for `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`
  and `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`.
- No tests, consumers, probes, or live filesystem operations were run. Checks used Rust/Cargo
  1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5; local Xcode is below
  the repo's required Xcode 27.x baseline.

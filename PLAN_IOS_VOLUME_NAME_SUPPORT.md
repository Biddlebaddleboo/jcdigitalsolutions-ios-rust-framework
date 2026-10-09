# B175: iOS Volume Case-Name Support Snapshot

## Status

`IosFiles::volume_name_support_snapshot` exposes Foundation's cached case-sensitive and
case-preserved name support values for each retained semantic app-directory root. It preserves an
unreported value as `None` and does not change portable path comparison

## API

- `IosFiles::volume_name_support_snapshot(AppDirectory)` returns an
  `IosVolumeNameSupportSnapshot` for Documents, Caches, Temporary, or Application Support.
- `IosVolumeNameSupportSnapshot::case_sensitive_names_supported()` returns `Some(true)` or
  `Some(false)` when Foundation supplies `NSURLVolumeSupportsCaseSensitiveNamesKey`; `None` means
  the resource query failed, returned no value, or returned a value that was not an `NSNumber`.
- `IosVolumeNameSupportSnapshot::case_preserved_names_supported()` preserves the same three states
  for `NSURLVolumeSupportsCasePreservedNamesKey`.
- The values are queried when `IosFiles::new` resolves each root and are cached in that
  `IosFiles` instance. They describe Foundation's volume-level support report, not a fresh query.
- A reported case-sensitive value says whether case variants are distinct names. A reported
  case-preserved value says whether the volume preserves case in stored names. Neither value
  defines Unicode normalization/collation, reserves a name, prevents concurrent namespace
  changes, or guarantees a later create/rename operation. The actual filesystem operation remains
  authoritative.
- This is an iOS-only metadata helper. It changes no portable `AppPath` comparison/validation or
  `FileBackend` behavior, and adds no dependency, framework, permission, Info.plist key,
  entitlement, or required-reason privacy API.

## Platform evidence

- Apple's [`NSURLVolumeSupportsCaseSensitiveNamesKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportscasesensitivenameskey?language=objc)
  defines a read-only Boolean `NSNumber` for whether a volume supports case-sensitive names.
- Apple's [`NSURLVolumeSupportsCasePreservedNamesKey` documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportscasepreservednameskey?language=objc)
  defines a read-only Boolean `NSNumber` for whether a volume supports case-preserved names.
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` declares both keys available from iOS
  4.0. The package's existing iOS 10.0 baseline is unchanged.
- Locked `objc2-foundation` 0.3.2 exposes both constants under its existing `NSURL` feature; the
  existing `NSURL` resource-value API and `NSNumber` type handle their values. B175 adds no
  dependency or native API import.

## Validation

- Passed `cargo fmt --package ios-files -- --check` and `git diff --check`.
- Passed locked offline `cargo +1.94.1 check -p ios-files` and strict Clippy with `-- -D warnings`
  for `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`
  and `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`.
- No tests, consumers, probes, or live filesystem operations were run. Checks used Rust/Cargo
  1.94.1, Xcode 26.6 build `17F113`, and iPhoneOS/iPhoneSimulator SDK 26.5; local Xcode is below
  the repo's required Xcode 27.x baseline.

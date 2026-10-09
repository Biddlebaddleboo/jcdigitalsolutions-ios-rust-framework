# B178: No-Go for Volume Maximum File Size

## Disposition

Do not add a Rust `volume_maximum_file_size_bytes` API yet. The public SDK header describes a
maximum file size in bytes, but Apple's current online reference also labels the returned
`NSNumber` as Boolean. The value's numeric type is not stated in the generated binding. A Rust
adapter must not turn a Boolean `NSNumber` into a plausible byte limit

## Candidate and evidence

- The candidate is Foundation's `NSURLVolumeMaximumFileSizeKey`, queried from one retained
  semantic app-directory URL during `IosFiles::new` and exposed as an optional `u64` snapshot.
  Such a value could help a caller reject a file size above a filesystem-reported limit before a
  write. It would not be an app quota, available-space value, or guarantee of write success.
- Apple's current [`NSURLVolumeMaximumFileSizeKey` reference](https://developer.apple.com/documentation/foundation/urlresourcekey/volumemaximumfilesizekey)
  describes the semantic value as the largest file size supported by the volume in bytes, or nil
  when it cannot be determined, but also says the value is a Boolean `NSNumber`.
- The installed iPhoneOS 26.5 SDK's `Foundation/NSURL.h` instead comments that the key returns the
  largest file size in bytes or nil, with value type `NSNumber`; the header gives no signedness,
  integer encoding, or Boolean exclusion.
- Locked `objc2-foundation` 0.3.2 exposes only the `NSURLVolumeMaximumFileSizeKey` resource-key
  constant. `NSURL.getResourceValue_forKey_error` returns a dynamically typed object; `NSNumber`
  can represent Boolean and numeric values, and the generated binding does not constrain this
  particular key to an integer encoding.
- The online reference and SDK header agree on the resource key's existence and broad byte-limit
  meaning, but conflict on whether the returned `NSNumber` is Boolean. Without a documented
  integer value type, converting `NSNumber` to `u64` could turn a Boolean into `0` or `1` byte.
  The no-runtime-probe scope leaves that behavior unverified.

## Closure criteria

- Apple documentation or SDK metadata must state that the value is an integer byte count and
  identify how to handle an unavailable value.
- The Objective-C binding must expose a safe representation consistent with that documented
  integer type, or a future scoped runtime study must establish the returned encoding on supported
  iOS targets.
- Any later implementation must report a filesystem maximum only, preserve unavailable state,
  and state that the result does not promise a write will succeed or bypass app quotas.

## Validation

- Source review used the installed iPhoneOS 26.5 SDK header and the locked
  `objc2-foundation` 0.3.2 generated binding; Apple documentation was checked at the cited URL.
- Passed `git diff --check` and `cargo +1.94.1 run --locked --offline -p xtask -- docs-check`.
- No tests, builds, runtime queries, probes, or live filesystem operations were run.

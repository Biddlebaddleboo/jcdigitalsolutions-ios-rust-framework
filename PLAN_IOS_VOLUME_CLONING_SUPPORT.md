# B199: iOS Volume Clone-Support Snapshot

## Scope

B199 adds `IosFiles::volume_clone_support_snapshot(AppDirectory) -> Result<Option<bool>, FileError>`.
At `IosFiles::new`, the backend reads Foundation's `NSURLVolumeSupportsFileCloningKey` for each
retained Documents, Caches, Temporary, and Application Support root. The call returns that cached
value for the requested semantic directory. It is a volume-level hint and does not alter B196's
clone operation or add a portable `FileBackend` method.

## API and binding evidence

The installed iPhoneOS 26.5 SDK declares `NSURLVolumeSupportsFileCloningKey` in Foundation's
`NSURL.h` as available from iOS 10.0. Apple's [NSURL resource-key documentation](https://developer.apple.com/documentation/foundation/urlresourcekey/volumesupportsfilecloningkey?language=objc)
defines it as a read-only Boolean `NSNumber` that reports whether the volume supports `clonefile(2)`.
The local `objc2-foundation` 0.3.2 generated binding exposes the symbol behind its existing
`NSString` feature; `ios-files` already enables that feature. No dependency or root manifest change
is needed.

## Contract and limits

- `Some(true)` and `Some(false)` preserve Foundation's reported Boolean. `None` means the resource
  query failed or returned no `NSNumber` Boolean.
- The value is queried once per retained root during `IosFiles::new`; it may be stale by later use.
- This is not a per-file, source/destination-pair, or flag-specific guarantee. In particular, it
  does not establish that `fclonefileat` with B196's flags will succeed. The native operation and
  error remain authoritative, including for cross-volume pairs.
- The query reads no file contents, accepts no arbitrary URL, starts no security scope, exposes no
  native object, and changes no sandbox-path or permission behavior. It needs no usage-description
  key, permission, or entitlement.
- The iOS availability floor is 10.0 for the key. `IosFiles::new` performs the Foundation resource
  queries synchronously; construction may block as documented for the backend.

## Validation

- Passed device and arm64 Simulator checks:
  `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 check --locked --offline --no-default-features -p ios-files --target aarch64-apple-ios-sim`.
- Passed strict Clippy:
  `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios -- -D warnings`
  and `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed rustdoc:
  `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios`
  and `cargo +1.94.1 doc --locked --offline --no-deps -p ios-files --target aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 fmt --package ios-files -- --check`,
  `target/debug/xtask docs-check`, and tracked diff whitespace checks.
- No tests, linked probes, consumer execution, runtime queries, or live file operations are part
  of this slice.

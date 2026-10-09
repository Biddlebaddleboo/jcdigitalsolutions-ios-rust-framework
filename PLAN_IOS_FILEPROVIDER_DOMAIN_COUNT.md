# B82: iOS File Provider registered-domain count snapshot

## Status

The additive API uses the existing read-only `getDomainsWithCompletionHandler:` callback and copies
only the returned array length to `u64`. The host/device/Simulator compile, strict Clippy, rustdoc,
format, docs, and package link/import gates pass. The Release link probes were built and inspected,
not executed; no tests or live provider queries were run. The existing B75 Boolean API and native
selector remain unchanged

## Scope

- Add `request_registered_domain_count() -> RegisteredDomainCountFuture` to `ios-file-provider`
- Return `RegisteredDomainCount::count() -> u64`; preserve `CountOutOfRange` if the native `usize`
  length cannot fit the fixed-width value
- Keep the result caller-owned and independent of native FileProvider objects
- Reuse `+[NSFileProviderManager getDomainsWithCompletionHandler:]`; no added prompt, permission,
  entitlement, or framework claim
- Do not expose IDs, display names, location/account data, URLs, items, or handles; do not infer
  enablement, connection, sync, or file access
- This is an additive read-only value in existing row 099; it does not add a capability row or
  complete FileProvider support
- Each API call starts an independent query. The callback queue remains unspecified, and dropping
  the returned future abandons Rust interest without cancelling the native request

## Evidence

- Installed iOS 26.5 SDK `NSFileProviderManager.h` declares
  `+getDomainsWithCompletionHandler:` and returns the calling app provider's `NSArray` of domains
- `objc2-file-provider` 0.3.2 exposes the typed manager call and `NSArray::count`; the count requires
  no `NSFileProviderDomain::identifier` or additional binding feature
- Apple describes [`getDomainsWithCompletionHandler:`](https://developer.apple.com/documentation/fileprovider/nsfileprovidermanager/getdomainswithcompletionhandler%28_%3A%29?language=objc)
  as returning all domains for the File Provider extension and [`NSFileProviderDomain`](https://developer.apple.com/documentation/fileprovider/nsfileproviderdomain?language=objc)
  as one domain of that extension
- The existing API floor is iOS 11.0. This count operation adds no newer symbol

## Validation

- No tests were added or run, and no provider request or consumer/probe binary was executed
- Passed `sh platform/ios/ios-file-provider/check.sh`. Its static checks, formatting, locked host,
  device, and Simulator checks, strict device/Simulator Clippy, iOS rustdoc, and Release link/import
  audit all passed
- The link/import audit built and inspected (but did not execute) the device and arm64 Simulator
  Release probes. Both contain only `FileProvider`, `Foundation`, `libSystem.B.dylib`, and
  `libobjc.A.dylib`; both contain `NSFileProviderManager` and
  `getDomainsWithCompletionHandler:`; device minos is iOS 11.0 and Simulator minos is iOS 14.0
- Passed the focused commands:
  `cargo +1.94.1 check --locked --offline -p ios-file-provider`,
  `cargo +1.94.1 check --locked --offline -p ios-file-provider --target aarch64-apple-ios`,
  `cargo +1.94.1 check --locked --offline -p ios-file-provider --target aarch64-apple-ios-sim`,
  `cargo +1.94.1 clippy --locked --offline --lib -p ios-file-provider --target aarch64-apple-ios -- -D warnings`,
  `cargo +1.94.1 clippy --locked --offline --lib -p ios-file-provider --target aarch64-apple-ios-sim -- -D warnings`,
  `cargo +1.94.1 doc --locked --offline --no-deps -p ios-file-provider --target aarch64-apple-ios`,
  `cargo fmt --manifest-path platform/ios/ios-file-provider/Cargo.toml -- --check`,
  `cargo +1.94.1 xtask docs-check`, and `git diff --check`

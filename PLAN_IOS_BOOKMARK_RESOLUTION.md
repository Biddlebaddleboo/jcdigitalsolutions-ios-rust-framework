# B83: iOS Bookmark Location Resolution

## Status

`B83` is a separate bookmark-location helper in `ios-files`, distinct from B82's FileProvider
registered-domain count snapshot. It does not change FileProvider behavior or app-data row counts.
`ios-files::IosResolvedBookmark::resolve_unscoped` resolves caller-supplied, non-security-scoped
Foundation bookmark data to a retained file URL and copies Foundation's stale bit. The method is
`unsafe` because the binding does not expose a way to verify bookmark scope; Apple says
`NSURLBookmarkResolutionWithoutImplicitStartAccessing` does not apply to security-scoped bookmark
data. The adapter checks for iOS 14.2 before use of that option, which prevents implicit start for
ephemeral security-scoped URLs only. Device and Simulator `cargo check`, strict Clippy, rustdoc,
format, and docs checks pass on Xcode 26.6 / iPhoneOS SDK 26.5 with Rust 1.94.1. No tests, live
bookmark calls, or consumer/probe binaries are run.

## Objective

Add one iOS-only location resolver for plain bookmark data. Keep it distinct from the existing
security-scope guard and file coordinator; make no document-provider readiness or file-I/O claim.

## Scope

- Input is caller-owned, non-security-scoped `NSData` bookmark data; output keeps one `NSURL` and a
  copied stale Boolean. The unsafe call's contract places this unverified condition on its caller.
- Resolution uses `WithoutUI`, `WithoutMounting`, and `WithoutImplicitStartAccessing`.
- The operation returns `Unsupported` before iOS 14.2 and rejects non-file URLs.
- The iOS header marks `NSURLBookmarkResolutionWithSecurityScope` unavailable on iOS. The
  `WithoutImplicitStartAccessing` option applies to ephemeral security-scoped URLs, not to
  security-scoped bookmark data. The helper cannot resolve the latter safely and cannot detect
  whether input carries that scope.
- No bookmark create or save, picker UI, grant/provenance check, security-scope start/stop, file
  coordination, `NSFilePresenter`, provider lifecycle, read/write, or path containment is in scope.
- The crate's iOS 10.0 floor and sandbox `IosFiles` contract do not change; this method has an
  operation-specific iOS 14.2 floor.

## API evidence

- The installed iOS 26.5 `Foundation.framework/Headers/NSURL.h` declares bookmark resolution from
  iOS 4.0 and `NSURLBookmarkResolutionWithoutImplicitStartAccessing` from iOS 14.2. The header says
  the latter disables implicit start for ephemeral security-scoped URLs and is not applicable to
  security-scoped bookmark data; `NSURLBookmarkResolutionWithSecurityScope` is unavailable on iOS.
- `objc2-foundation` 0.3.2 provides typed bookmark resolution, resolution options, the runtime
  `NSProcessInfo` version query, and `NSData`/`NSError` bindings.
- Apple's [bookmark resolution API](https://developer.apple.com/documentation/foundation/url/init%28resolvingbookmarkdata%3Aoptions%3ArelativeTo%3Abookmarkdataisstale%3A%29-3ic6f)
  documents the stale result and no-implicit-start option. The installed Objective-C header's
  availability and scope note govern this iOS binding; this slice does not claim external document
  bookmark or scope support.
- The separate [B82 FileProvider count plan](PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md) covers only a
  registered-domain count for the calling app's own extension; it is unrelated to bookmark data,
  bookmark resolution, or file access.

## Validation

- Do not add or run tests, make a live bookmark query, or execute a consumer/probe binary.
- Passed `cargo check --locked --offline -p ios-files --target aarch64-apple-ios`.
- Passed `cargo check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed `cargo clippy --locked --offline -p ios-files --target aarch64-apple-ios -- -D warnings`.
- Passed `cargo clippy --locked --offline -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed `cargo doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`.
- Passed `cargo fmt --package ios-files -- --check`, `cargo xtask docs-check`, `git diff --check`,
  and an AWK trailing-whitespace scan of new files.
- The existing link/import probe was not run because it builds consumer binaries. No test or probe
  binary was run.
- Toolchain evidence: Xcode 26.6 build 17F113, iPhoneOS SDK 26.5, Rust/Cargo 1.94.1. This is below
  the repository's required Xcode 27.x baseline.

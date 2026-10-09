# B85: iOS Plain Bookmark Data Creation

## Status

`IosPlainBookmarkData::create` accepts a caller-owned file `NSURL` and retains Foundation bookmark
data created with `NSURLBookmarkCreationWithoutImplicitSecurityScope`. The type has a private data
field and no constructor for arbitrary `NSData`; its `resolve` method can therefore call B83's unsafe
resolver with data whose creation options are known. Device/Simulator checks, strict Clippy, iOS
rustdoc, format, docs-check, and diff-check pass. No tests, live bookmark calls, or consumer/probe
binaries were run. The first device compile exposed the generated method's `NSArray` feature gate;
the package-local Foundation feature was enabled and both target checks then passed

## Objective

Add a narrow iOS-only way to create location bookmark data without embedding implicit ephemeral
security scope, while retaining provenance in an opaque Rust value for safe resolution. Do not add
access rights, picker UI, a document-provider flow, or a bookmark persistence policy

## Contract

- `IosPlainBookmarkData::create(&NSURL)` accepts file URLs only. It synchronously calls
  `bookmarkDataWithOptions:includingResourceValuesForKeys:relativeToURL:error:` with only
  `NSURLBookmarkCreationWithoutImplicitSecurityScope`, no resource-value keys, and no relative URL.
- The method may do file-system work and return a Foundation error. It does not start security-scoped
  access, prompt, request an entitlement, grant access, or prove the URL is accessible later.
- The wrapper retains the returned `NSData`; `data()` borrows it for caller-managed storage. The
  wrapper has no constructor from arbitrary `NSData`, and `resolve()` safely resolves only the
  wrapper's internally created data through B83's iOS 14.2+ no-implicit-start path.
- The type does not create security-scoped bookmark data: the selected option prevents implicit
  ephemeral-scope inclusion, while `NSURLBookmarkCreationWithSecurityScope` is unavailable on iOS.
- A caller that loads bookmark bytes from storage as an arbitrary `NSData` cannot forge this typed
  provenance and must use B83's unsafe `IosResolvedBookmark::resolve_unscoped` contract.
- Bookmark data names a location; it is not a file handle, access grant, sandbox-containment proof,
  security-scope lifetime, or guarantee that the file remains present or resolvable.
- This API does not save bookmark data, start or stop scope, use FileProvider, coordinate I/O,
  register `NSFilePresenter`, read or write file contents, or add the URL to `framework-files`.

## API and binding evidence

- Installed Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5 `Foundation.framework/Headers/NSURL.h`
  declares `bookmarkDataWithOptions:includingResourceValuesForKeys:relativeToURL:error:` from iOS
  4.0 and `NSURLBookmarkCreationWithoutImplicitSecurityScope` from iOS 5.0.
- The same header marks `NSURLBookmarkCreationWithSecurityScope` unavailable on iOS and describes
  `NSURLBookmarkCreationWithoutImplicitSecurityScope` as disabling implicit security-scope
  inclusion. It notes that the option does not apply to security-scoped bookmarks; this operation
  creates new data without either implicit scope or the unavailable explicit option.
- `objc2-foundation` 0.3.2 exposes the typed method
  `NSURL::bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error` and
  `NSURLBookmarkCreationOptions::WithoutImplicitSecurityScope`. `ios-files` already selects the
  needed `NSURL`, `NSArray`, `NSData`, `NSError`, and `NSString` features; no dependency or lock
  change is needed.
- Apple documents the [bookmark-data method](https://developer.apple.com/documentation/foundation/nsurl/bookmarkdata%28options%3Aincludingresourcevaluesforkeys%3Arelativeto%3A%29?language=objc)
  and the [`WithoutImplicitSecurityScope` option](https://developer.apple.com/documentation/foundation/nsurl/bookmarkcreationoptions/withoutimplicitsecurityscope?language=objc).
- B83 resolution still requires iOS 14.2 for
  `NSURLBookmarkResolutionWithoutImplicitStartAccessing`; B85 creation itself adds no floor above
  the `ios-files` package's iOS 10.0 minimum.

## Validation

- Do not add or run tests, issue a live bookmark query, execute consumer/probe binaries, or add Swift.
- Passed `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios` and
  `cargo +1.94.1 check --locked --offline -p ios-files --target aarch64-apple-ios-sim`.
- Passed `cargo +1.94.1 clippy --locked --offline -p ios-files --target aarch64-apple-ios -- -D warnings`
  and `cargo +1.94.1 clippy --locked --offline -p ios-files --target aarch64-apple-ios-sim -- -D warnings`.
- Passed `cargo +1.94.1 doc --locked --offline -p ios-files --no-deps --target aarch64-apple-ios`,
  `cargo fmt --package ios-files -- --check`, `cargo +1.94.1 xtask docs-check`, and
  `git diff --check`.
- Do not run the app-data link/import probe in this slice; it builds consumer binaries.
- Checks used Xcode 26.6 build `17F113`, iPhoneOS/iPhoneSimulator SDK 26.5, and Rust/Cargo 1.94.1;
  this is below the repository's required Xcode 27.x baseline.

## Sources and scope links

- [Apple `NSURL.bookmarkData` documentation](https://developer.apple.com/documentation/foundation/nsurl/bookmarkdata%28options%3Aincludingresourcevaluesforkeys%3Arelativeto%3A%29?language=objc)
- [Apple `NSURLBookmarkCreationWithoutImplicitSecurityScope` documentation](https://developer.apple.com/documentation/foundation/nsurl/bookmarkcreationoptions/withoutimplicitsecurityscope?language=objc)
- [B83 raw bookmark resolution](PLAN_IOS_BOOKMARK_RESOLUTION.md)
- [B82 FileProvider registered-domain count](PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md)
- [D1 app-data plan](PLAN_IOS_APP_DATA.md)

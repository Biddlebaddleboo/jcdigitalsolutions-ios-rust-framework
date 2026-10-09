# Background Assets read-only surfaces

`ios-background-assets` exposes one read-only, iOS-only snapshot:

```rust
let count = ios_background_assets::request_download_queue_count().await?;
let queued = count.count();
```

The backend calls `BADownloadManager.sharedManager()` and
`fetchCurrentDownloadsWithCompletionHandler:`. Apple defines the returned array as downloads
queued by the app or its extension. This API is available from iOS 16.1. The Rust result contains
only the array length, copied to `u64`; the backend does not read any `BADownload` property. This
avoids access to nonatomic download properties on the framework callback queue.

The request starts when `request_download_queue_count()` runs. Each call starts its own native
request. The count is a point-in-time queue snapshot from the callback, not a stable value. A
change by the app, extension, or system may make it stale at once. The query does not schedule or
cancel a download, fetch asset bytes, inspect an asset pack, or prove that Background Assets is
available, configured, or ready. A zero count means only that the returned queue had no entries.

This surface covers the unmanaged `BADownloadManager` queue only. It does not query managed asset
packs through `BAAssetPackManager`. A host that uses unmanaged Background Assets still needs its
app and downloader-extension targets, shared App Group, `BAManifestURL`, and other policy keys
that fit its download flow. The backend does not inspect or validate any host target, entitlement,
manifest, extension, profile, or App Store setup. A query with no valid host setup may return an
empty queue; that value must not be read as a support result.

## B110: inspect a local asset-pack manifest

`inspect_asset_pack_manifest(json, app_group_id)` parses caller-owned JSON with
`BAAssetPackManifest.initFromData:applicationGroupIdentifier:error:` and returns the count of the
parsed `assetPacks` set. This iOS 26.0+ API is a synchronous parse of a manual asset-pack manifest;
it is separate from the older unmanaged download queue above. It does not call
`BAAssetPackManager.sharedManager()`, so it does not opt the app into automatic management, and it
does not query local installation state, fetch a manifest, schedule a download, or read asset
bytes.

The app-group identifier is passed to Apple's parser for the manifest representation. The call
does not prove that the app group is present in the signed entitlement or shared by an extension.
Malformed JSON or a manifest Apple cannot parse returns `AssetPackManifestError::Native` with the
native error domain and code. `ApiUnavailable` means the runtime is below iOS 26.0. The count is
only the number of manifest entries; it is not a count of downloaded or available local packs.

The B110 native API is a safe Rust facade over the typed 0.3.2 binding: it owns the `NSData` and
`NSString` inputs for the call, creates the manifest through its documented initializer, and reads
the `readonly, copy` `assetPacks` set while the manifest remains retained. It returns only the
set's count and retains no Objective-C value.

The callback copies an `NSError` domain and native `NSInteger` code into
`NativeBackgroundAssetsError`. `ApiUnavailable` means the runtime iOS version is below 16.1;
`UnsupportedPlatform` marks a non-iOS build. Dropping the future detaches Rust interest but does
not cancel the native request. The callback may still complete its owned request state after the
future drops.

## Scope and validation

B97 adds no portable Background Assets trait, download operation, progress delegate, installed-file
access, Swift source, or Swift ABI. B110 adds only a count of caller-provided manifest entries; it
does not report asset-pack status. Neither slice adds a permission prompt or host configuration
tool. See [the focused implementation plan](../../PLAN_CAPABILITIES_BACKGROUND_ASSETS.md).

B97 and B110 compile, strict library Clippy, rustdoc, package format, and docs-check gates ran with
Rust 1.94.1 and Xcode 26.6 build 17F113 / iOS SDK 26.5. That Apple toolchain is below the repo's
Xcode 27.x baseline. These gates did not run tests, a probe, an app, a parser call, a device query,
or a Background Assets lifecycle.

## Apple references

- [Background Assets overview](https://developer.apple.com/documentation/backgroundassets)
- [`BADownloadManager`](https://developer.apple.com/documentation/backgroundassets/badownloadmanager)
- [`fetchCurrentDownloadsWithCompletionHandler:`](https://developer.apple.com/documentation/backgroundassets/badownloadmanager/fetchcurrentdownloads%28completionhandler%3A?language=objc)
- [Configure an unmanaged Background Assets project](https://developer.apple.com/documentation/backgroundassets/configuring-an-unmanaged-background-assets-project)
- [App Groups entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups)

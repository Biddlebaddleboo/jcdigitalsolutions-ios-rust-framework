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

## B113: owned asset-pack manifest metadata

`inspect_asset_pack_manifest_entries(json, app_group_id)` returns an owned Rust entry per pack in
the parsed manifest. Each entry has the unique `identifier`, `download_size_bytes`, and native
`version`. Entries sort by identifier, then version and download size, so the Rust vector order is
stable even though Apple's `NSSet` order is not. The native `NSInteger` download size is checked
for a nonnegative byte count and exposed as `u64`; version stays `isize` to preserve Apple's exact
`NSInteger` value.

The values describe only the supplied local JSON manifest. Apple notes that a `BAAssetPack` object
can become invalid when its server asset changes; B113 does not fetch server state or use the
manager, and copies all fields during the synchronous read. The returned strings and integers do
not establish current download, installation, freshness, entitlement, or extension status.

## B116: optional custom metadata bytes

Each B113 entry also exposes `user_info_json_bytes()`. It returns the optional custom metadata from
the manifest as owned raw bytes; Apple documents `BAAssetPack.userInfo` as JSON-encoded custom
information. The Rust facade does not parse or validate that JSON. `None` is preserved when Apple
returns no value; Apple notes the property is `nil` for Apple-hosted asset packs. This field does
not imply server freshness or any download state.

## B119 audit: no additional iOS 26.5 asset-pack field

No B119 API was added. The installed iOS 26.5 SDK and typed 0.3.2 binding expose no further
read-only `BAAssetPack` metadata beyond the B113 and B116 fields. Apple documents `language` as a
beta field for iOS 27, but it is absent from the installed SDK and binding. `download()` and
`allDownloads` create `BADownload` objects that can be scheduled, so they are not metadata-only
snapshots.

The callback copies an `NSError` domain and native `NSInteger` code into
`NativeBackgroundAssetsError`. `ApiUnavailable` means the runtime iOS version is below 16.1;
`UnsupportedPlatform` marks a non-iOS build. Dropping the future detaches Rust interest but does
not cancel the native request. The callback may still complete its owned request state after the
future drops.

## B124: essential queue-entry count

`request_essential_download_queue_count()` returns the number of entries in Apple's asynchronous
queue snapshot whose `BADownload.isEssential` value is `true`. The count describes only that
returned queue at the callback's observation point. It does not count installed assets, all
essential assets on the device, or downloads outside that queue. The API requires iOS 16.4.

The backend reads only the readonly atomic `isEssential` property on retained queue objects. It does
not read the nonatomic `state`, `identifier`, or `uniqueIdentifier` properties, and it does not
schedule or cancel downloads.

## B127: non-default-priority queue-entry count

`request_nondefault_priority_download_queue_count()` returns the number of entries in Apple's
asynchronous queue snapshot whose `BADownload.priority` differs from `BADownloaderPriorityDefault`.
It counts values above or below the default without assigning a schedule-time or download-state
meaning. The count covers only entries returned in that queue snapshot and requires iOS 16.1.

The backend reads only the readonly atomic `priority` property on retained queue objects. It does
not read the nonatomic `state`, `identifier`, or `uniqueIdentifier` properties, and it does not
schedule or cancel downloads.

## B130: install-time download allowance

`inspect_restricted_download_size_remaining(info)` copies the overall remaining byte allowance
from the `BAAppExtensionInfo` value Apple supplies to the downloader extension's
`downloads(for:manifestURL:extensionInfo:)` callback. It returns `None` when Apple reports that
downloads are unrestricted; otherwise it returns the remaining bytes that may be scheduled before
app launch. The helper does not construct extension info, read the manifest, or enqueue downloads.
It requires iOS 16.1.

## B133: essential install-time download allowance

`inspect_restricted_essential_download_size_remaining(info)` copies the remaining bytes under
Apple's essential download allowance from that same callback value. It preserves `None` when Apple
reports that downloads are unrestricted and requires iOS 16.4. Both helpers use the typed
`BAAppExtensionInfo` binding and copy Apple's `Int?` value into Rust-owned `Option<u64>`; a negative
native byte count returns `ExtensionDownloadAllowanceError::InvalidRemainingBytes`.

The allowance helpers only let Rust callback code inspect system-supplied scheduling context. They
do not implement `BADownloaderExtension`, choose a manifest policy, create downloads, call the
manager, validate an `Info.plist` allowance, or promise that a returned download will be scheduled.

## B136: classify the extension content-request value

`classify_extension_content_request(request)` maps the typed callback's `BAContentRequest` value to
`ExtensionContentRequestKind::{Install, Update, Periodic}`. It preserves any value not named by the
installed SDK as `Unknown(native_value)`, so a future request reason is not mislabeled. The iOS
26.5 SDK names install, update, and periodic requests; this conversion reads no Objective-C
object, parses no manifest, and schedules no downloads.

## B139 audit: no additional typed error classifier

No API was added. The SDK's `BAError.h` introduces `BAErrorDomain` in iOS 17.0, and the 0.3.2
binding exposes it only under the `BAError` feature. A safe mapping of a captured error must first
establish that its domain equals Apple's constant; the SDK header does not state the constant's
string value. Existing `NativeBackgroundAssetsError` already preserves arbitrary native domains
and codes without assigning Background Assets meaning to another framework's error. The new
classifier was deferred rather than infer a domain or add an iOS 17-only static-symbol path.

## B142 audit: no managed asset-pack status query

No API was added. The installed SDK's iOS 26.4 managed status methods are absent from the typed
0.3.2 `BAAssetPackManager` binding; the binding exposes only the older status query that may fetch
server state and is deprecated in favor of a typed method the binding does not expose. The local
status API also returns an empty value for both unknown and not-downloaded pack identifiers. Any
manager query would opt the host into managed asset-pack handling and requires the matching
managed downloader-extension protocol, so this unmanaged package does not expose it.

## Scope and validation

B97, B124, and B127 add only read-only queue counts; none adds download scheduling, a progress
delegate, installed-file access, Swift source, or Swift ABI. B110 adds a count of caller-provided
manifest entries; B113 adds owned manifest metadata only; B116 adds optional custom JSON bytes.
B130 and B133 copy allowance values from a system-supplied downloader-extension callback object;
B136 classifies its typed request value. These slices do not implement the extension protocol.
None reports asset-pack status or adds a permission prompt or host configuration tool. See [the
focused implementation plan](../../PLAN_CAPABILITIES_BACKGROUND_ASSETS.md).

B97, B110, B113, B116, B124, B127, B130, B133, and B136 compile, strict library Clippy, rustdoc,
package format, and docs-check gates ran with Rust 1.94.1 and Xcode 26.6 build 17F113 / iOS SDK 26.5. That
Apple toolchain is below the repo's Xcode 27.x baseline. These gates did not run tests, a probe, an
app, a parser call, a device query, or a Background Assets lifecycle.

## Apple references

- [Background Assets overview](https://developer.apple.com/documentation/backgroundassets)
- [`BADownloadManager`](https://developer.apple.com/documentation/backgroundassets/badownloadmanager)
- [`fetchCurrentDownloadsWithCompletionHandler:`](https://developer.apple.com/documentation/backgroundassets/badownloadmanager/fetchcurrentdownloads%28completionhandler%3A?language=objc)
- [Configure an unmanaged Background Assets project](https://developer.apple.com/documentation/backgroundassets/configuring-an-unmanaged-background-assets-project)
- [App Groups entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups)
- [`BAAssetPack`](https://developer.apple.com/documentation/backgroundassets/baassetpack)
- [`BAAssetPack.identifier`](https://developer.apple.com/documentation/backgroundassets/baassetpack/identifier)
- [`BAAssetPack.downloadSize`](https://developer.apple.com/documentation/backgroundassets/baassetpack/downloadsize)
- [`BAAssetPack.version`](https://developer.apple.com/documentation/backgroundassets/baassetpack/version)
- [`BAAssetPack.userInfo`](https://developer.apple.com/documentation/backgroundassets/baassetpack)
- [`BADownload.isEssential`](https://developer.apple.com/documentation/backgroundassets/badownload/isessential)
- [`BADownload.priority`](https://developer.apple.com/documentation/backgroundassets/badownload/priority-swift.property)
- [`BADownloaderPriorityDefault`](https://developer.apple.com/documentation/backgroundassets/badownload/priority-swift.struct/default)
- [`BAAppExtensionInfo`](https://developer.apple.com/documentation/backgroundassets/baappextensioninfo)
- [`restrictedDownloadSizeRemaining`](https://developer.apple.com/documentation/backgroundassets/baappextensioninfo/restricteddownloadsizeremaining)
- [`restrictedEssentialDownloadSizeRemaining`](https://developer.apple.com/documentation/backgroundassets/baappextensioninfo/restrictedessentialdownloadsizeremaining-5r8v0)
- [`downloads(for:manifestURL:extensionInfo:)`](https://developer.apple.com/documentation/backgroundassets/badownloaderextension-qwaw/downloads%28for%3Amanifesturl%3Aextensioninfo%3A%29)
- [`BAContentRequest`](https://developer.apple.com/documentation/backgroundassets/bacontentrequest)
- [`BAAssetPack.language` (beta)](https://developer.apple.com/documentation/backgroundassets/baassetpack/language)
- [Reducing download and storage demands with localized asset packs](https://developer.apple.com/documentation/backgroundassets/reducing-download-and-storage-demands-with-localized-asset-packs)
- [`objc2-background-assets` 0.3.2](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/)

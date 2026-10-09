# PLAN_CAPABILITIES_BACKGROUND_ASSETS.md — Workstream D72: Row 036 Feasibility Gate

## Status

D72 found no global BackgroundAssets support, authorization, or readiness query. Background Assets is a host-app asset delivery system, not an OS permission or service whose presence proves that an app can fetch or install assets.

One narrow value snapshot is technically supportable without performing a download: `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler` returns the current scheduled or in-flight queue for the app or its downloader extension. An API could report only the count of those queue entries. It must not call that count installed assets, completed downloads, available assets, or BackgroundAssets support. Keep capability row `036-background-assets-where-applicable` at its current status for this feasibility-only work; no implementation or matrix change is in scope.

An asset-pack-local-availability query also exists in the current Apple SDK, but `objc2-background-assets` 0.3.2's generated `BAAssetPackManager` API does not expose the iOS 26.4 local-status methods. Do not use an unverified raw Objective-C selector or claim this route is Rust-callable until typed binding support and its exact features are verified.

## Objective

Establish the actual app and asset lifecycle for Background Assets, the host configuration it requires, its iOS API floors, and whether a narrow non-mutating status/value facade can be honest without claiming download or install behavior.

## Framework scope and lifecycle

Apple defines Background Assets as a framework for additional assets associated with an app. It supports two broad configurations:

1. **Unmanaged asset downloads** use `BADownloadManager` and a `BADownloaderExtension`. The app and extension share an App Group. The host app declares Background Assets information-property-list configuration, including `BAManifestURL`; the system uses that manifest to notify the extension around install/update or periodic background events. The extension supplies download requests, and the system schedules or runs them. Essential assets can affect whether the app may launch during install. Periodic execution and download timing remain system-controlled.
2. **Managed asset packs** use `BAAssetPackManager` and a matching managed downloader extension. The first reference to `BAAssetPackManager.sharedManager` opts the app into automatic system management; Apple marks absence of the required matching extension protocol as a programmer error. Self-hosted managed packs use `BAManagedDownloaderExtension`; Apple-hosted managed packs use StoreKit's `SKDownloaderExtension`. The app and extension share an App Group and the host declares `BAAppGroupID` and `BAHasManagedAssetPacks`; Apple-hosted use also declares `BAUsesAppleHosting`. Asset-pack manifests define `essential`, `prefetch`, or `onDemand` policy. Apple-hosted packs require App Store Connect hosting and distribution through TestFlight or the App Store.

These are not equivalent surfaces. A queue snapshot from `BADownloadManager` is not an asset-pack manifest or a statement about local installation. A per-pack local status from `BAAssetPackManager` applies only to configured managed asset packs.

## Installed SDK and binding evidence

Inspected Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5. Public headers are under:

`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/BackgroundAssets.framework/Headers/`

Relevant declarations:

- `BADownloadManager`, `BADownload`, and `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler:` are available from iOS 16.1. The callback returns scheduled or in-flight `BADownload` objects queued by the app or extension. `BADownload.state` and `BADownload.identifier` are also iOS 16.1 APIs. The synchronous `fetchCurrentDownloads:` is iOS 16.4 and is documented as potentially blocking; a future value facade should use the asynchronous callback.
- `BAAssetPackManager` and `BAAssetPack` are available from iOS 26.0.
- `getStatusRelativeToAssetPack:completionHandler:`, `getLocalStatusOfAssetPackWithIdentifier:completionHandler:`, and `assetPackIsAvailableLocallyWithIdentifier:` are available from iOS 26.4. The local-status method checks only offline-determinable values and causes no network traffic or download/update/removal. It cannot establish whether the ID exists: an empty status for an unknown ID is indistinguishable from a known pack that is not downloaded.
- `BAAssetPackStatus` uses bit flags including `DownloadAvailable`, `UpdateAvailable`, `UpToDate`, `OutOfDate`, `Obsolete`, `Downloading`, and `Downloaded`.

The docs.rs listing for `objc2-background-assets` 0.3.2 exposes generated `BADownloadManager`, `BADownload`, and `BAAssetPackManager` bindings. Its `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler` needs crate features `BADownload` and `block2`; `BADownload.state` and `BADownload.identifier` need `BADownload`. The generated `BADownload.state` and `identifier` accessors are `unsafe`; their generated docs say the properties are nonatomic and might not be thread-safe. A future backend must define the allowed queue/thread and ownership behavior before it reads per-download properties.

The generated 0.3.2 `BAAssetPackManager` item list includes the older `getStatusOfAssetPackWithIdentifier:completionHandler:` but omits `getLocalStatusOfAssetPackWithIdentifier:completionHandler:` and `assetPackIsAvailableLocallyWithIdentifier:`. The 0.3.2 crate is not in this workspace or the inspected local Cargo source/cache, so no local compile or feature check was performed. The current Apple SDK has the APIs, but this binding gap blocks a verified typed-Rust asset-pack-local-status implementation on the inspected dependency.

## Host configuration and permissions

No Background Assets user authorization prompt or usage-description key appears in the inspected API/docs. No dedicated Background Assets entitlement was identified in Apple's setup docs. Host configuration is still required:

- Apple documents the App Groups capability on both app and downloader-extension targets. The signed App Groups entitlement key is `com.apple.security.application-groups`; both targets must share the same group.
- Unmanaged hosts declare `BAManifestURL` and, as required for their download policy, `BAInitialDownloadRestrictions`, `BAEssentialMaxInstallSize`, and `BAMaxInstallSize`.
- Managed hosts declare `BAAppGroupID` and `BAHasManagedAssetPacks`; Apple-hosted managed hosts also declare `BAUsesAppleHosting` and omit the other Background Assets plist keys as documented.
- A downloader extension target must use the protocol that matches its hosting mode. Extension principal-class and target framework-link setup are host responsibilities.

Do not claim that the App Groups entitlement alone grants Background Assets operation. The host must have the right extension, matching plist/manifest setup, valid app-group signing, and (for Apple hosting) an eligible upload/distribution configuration.

## Status/value facade feasibility

A narrow `current_download_queue_count()`-style API could honestly report the number of entries returned by `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler` at the callback's observation point. This operation only queries the queue; the documented method does not schedule or cancel a download. Define its value as a transient snapshot, not a stable count: another app/extension operation or the system may change the queue immediately after it is read. `fetchCurrentDownloadsWithCompletionHandler` reports errors and invokes its completion on the manager's completion queue. The docs also provide `performWithExclusiveControl` so the app and extension can avoid simultaneous manager operations; a future contract must decide whether and how queue snapshots use that coordination.

For an especially narrow v1 value, count the returned array without reading any `BADownload` properties. This avoids relying on the generated non-atomic, potentially non-thread-safe `state` or `identifier` getters. A later per-download snapshot needs a proven thread/queue rule and conservative mapping for future native states.

This value is not:

- a device or OS support query;
- proof that the host has an enabled/valid Background Download extension or signed App Group;
- a count of downloaded, installed, or usable asset files;
- a per-asset availability result;
- a promise about download timing, completion, install state, or future availability.

A separate managed-pack API could return local availability for one caller-supplied pack ID after iOS 26.4, but its empty status is ambiguous for unknown/not-downloaded packs and the current 0.3.2 typed binding omits the relevant methods. It should remain deferred until the binding and host baseline are validated.

## Feasibility result and next evidence

A read-only queue-count snapshot is an honest, bounded candidate if a later implementation explicitly scopes row 036 to app/extension queue observation and requires a configured Background Assets host. It does not support a generic portable `BackgroundAssetsAvailable` Boolean and does not imply any download/install operation. No source code is authorized by D72; row 036 is unchanged.

Before a future implementation:

1. Select unmanaged iOS 16.1 queue observation or managed iOS 26.4 per-pack local status; do not combine their semantics.
2. For the queue snapshot, verify 0.3.2 feature wiring and device/Simulator compile/link, callback block lifetime, error mapping, cancellation/drop behavior, and manager completion queue. Define app/extension concurrency and whether `performWithExclusiveControl` is required.
3. Avoid `BADownload.state` and other nonatomic properties unless a thread-safety proof exists; preserve unknown native state values if a later API exposes them.
4. Verify app-group entitlement/profile plus exact extension, plist, manifest, and hosting-mode setup in a host app. A crate compile cannot establish that configuration.
5. If using managed-pack local status, verify an updated typed binding for the iOS 26.4 methods, local-status behavior for downloaded/out-of-date/obsolete packs, unknown-ID ambiguity, and async completion ownership.
6. Validate actual background scheduling, installation, downloads, and asset reads only in a configured app and extension with representative packs; those behaviors are outside a status snapshot and must not be inferred from compile/link checks.

## Deferred work

- No capability matrix/status JSON, Cargo/workspace/lockfile, CI, aggregate plan, or shared index edit.
- No portable BackgroundAssets availability contract, iOS backend, downloader extension, app-group entitlement, Info.plist/manifest setup, download scheduling/cancel API, progress delegate, installation claim, or asset read API.
- No tests, probe execution, live download, or device lifecycle action.

## Apple and binding references

- [Background Assets overview](https://developer.apple.com/documentation/backgroundassets)
- [Configuring an unmanaged Background Assets project](https://developer.apple.com/documentation/backgroundassets/configuring-an-unmanaged-background-assets-project)
- [Downloading essential assets in the background](https://developer.apple.com/documentation/backgroundassets/downloading-essential-assets-in-the-background)
- [Downloading Apple-hosted asset packs](https://developer.apple.com/documentation/backgroundassets/downloading-apple-hosted-asset-packs)
- [Creating managed asset packs](https://developer.apple.com/documentation/backgroundassets/creating-managed-asset-packs)
- [BADownloadManager](https://developer.apple.com/documentation/backgroundassets/badownloadmanager)
- [BAAssetPackManager](https://developer.apple.com/documentation/backgroundassets/baassetpackmanager)
- [BAAssetPackStatus](https://developer.apple.com/documentation/backgroundassets/baassetpackstatus)
- [App Groups entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups)
- [`objc2-background-assets` 0.3.2 crate](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/)
- [`BADownloadManager` binding](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/struct.BADownloadManager.html)
- [`BADownload` binding](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/struct.BADownload.html)
- [`BAAssetPackManager` binding](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/struct.BAAssetPackManager.html)

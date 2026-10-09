# PLAN_CAPABILITIES_BACKGROUND_ASSETS.md — Workstream D72: Row 036 Feasibility Gate

## Status

D72 found no global BackgroundAssets support, authorization, or readiness query. Background Assets is a host-app asset delivery system, not an OS permission or service whose presence proves that an app can fetch or install assets. B97 later implements the unmanaged-queue count below; B124 and B127 add bounded entry-subset counts; B130 and B133 copy system-supplied extension allowance values; B136 classifies the typed callback reason; B139 and B142 record no-go audits for error decoding and managed-pack status; none changes that finding.

Narrow value snapshots are technically supportable without performing a download: `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler` returns the current scheduled or in-flight queue for the app or its downloader extension, and the system supplies `BAAppExtensionInfo` and `BAContentRequest` to the extension's `downloads(for:manifestURL:extensionInfo:)` callback. APIs report only queue counts, matching subsets, copied allowance values, or the classified callback reason. They do not mean installed assets, completed downloads, available assets, or BackgroundAssets support. D72 kept capability row `036-background-assets-where-applicable` at its then-current status; B97, B124, B127, B130, B133, and B136 remain scoped slices, not a broad support claim.

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

- `BADownloadManager`, `BADownload`, and `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler:` are available from iOS 16.1. The callback returns scheduled or in-flight `BADownload` objects queued by the app or extension. `BADownload.state`, `BADownload.identifier`, and `BADownload.priority` are iOS 16.1 APIs. `priority` is `readonly` and omits `nonatomic`, so its accessor is atomic by default; `BADownloaderPriorityDefault` is also available from iOS 16.1. `BADownload.isEssential` is iOS 16.4 and likewise atomic by default. The synchronous `fetchCurrentDownloads:` is iOS 16.4 and is documented as potentially blocking; queue snapshots use the asynchronous callback.
- `BAAssetPackManager` and `BAAssetPack` are available from iOS 26.0.
- `getStatusRelativeToAssetPack:completionHandler:`, `getLocalStatusOfAssetPackWithIdentifier:completionHandler:`, and `assetPackIsAvailableLocallyWithIdentifier:` are available from iOS 26.4. The local-status method checks only offline-determinable values and causes no network traffic or download/update/removal. It cannot establish whether the ID exists: an empty status for an unknown ID is indistinguishable from a known pack that is not downloaded.
- `BAAssetPackStatus` uses bit flags including `DownloadAvailable`, `UpdateAvailable`, `UpToDate`, `OutOfDate`, `Obsolete`, `Downloading`, and `Downloaded`.

The docs.rs listing for `objc2-background-assets` 0.3.2 exposes generated `BADownloadManager`, `BADownload`, and `BAAssetPackManager` bindings. Its `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler` needs crate features `BADownload` and `block2`; `BADownload.state`, `BADownload.identifier`, `BADownload.priority`, and `BADownload.isEssential` need `BADownload`. The generated `BADownload.state` and `identifier` accessors are `unsafe`; their generated docs say the properties are nonatomic and might not be thread-safe. The generated `priority(&self) -> BADownloaderPriority` and `isEssential(&self) -> bool` accessors have no nonatomic warning. B124 and B127 use only those readonly atomic getters on retained queue objects; B124 also has its iOS 16.4 guard, while B127 uses the iOS 16.1 queue floor.

The generated 0.3.2 `BAAssetPackManager` item list includes the older `getStatusOfAssetPackWithIdentifier:completionHandler:` but omits `getLocalStatusOfAssetPackWithIdentifier:completionHandler:` and `assetPackIsAvailableLocallyWithIdentifier:`. The 0.3.2 crate is not in this workspace or the inspected local Cargo source/cache, so no local compile or feature check was performed. The current Apple SDK has the APIs, but this binding gap blocks a verified typed-Rust asset-pack-local-status implementation on the inspected dependency.

`BAAppExtensionInfo.h` declares the class from iOS 16.1 and exposes readonly, strong, nullable `NSNumber *restrictedDownloadSizeRemaining`; `restrictedEssentialDownloadSizeRemaining` is available from iOS 16.4. Both properties omit `nonatomic`, so their getters are atomic by default. The generated 0.3.2 binding exposes both typed getters as `Option<Retained<NSNumber>>` under the `BAAppExtensionInfo` feature, which enables Foundation `NSValue`. Apple’s Swift documentation refines both to `Int?`, and documents `nil` when downloads are not restricted. The typed callback binding takes `&BAAppExtensionInfo` as the `extensionInfo` argument; the class has unavailable `init` and `new`, so the safe Rust facade accepts the object supplied by the system and does not construct one.

`BATypes.h` declares `BAContentRequest` as an `NSInteger` enum with `Install = 1`, `Update = 2`, and `Periodic = 3`; the iOS 16.1 `BADownloaderExtension` protocol passes it by value to its typed callback method. The 0.3.2 binding feature `BATypes` exposes the transparent typed enum and its three constants. The installed public header has no language-change enumerator; the Rust conversion preserves all unmatched `NSInteger` values as `ExtensionContentRequestKind::Unknown` rather than assuming an exhaustive native enum.

## Host configuration and permissions

No Background Assets user authorization prompt or usage-description key appears in the inspected API/docs. No dedicated Background Assets entitlement was identified in Apple's setup docs. Host configuration is still required:

- Apple documents the App Groups capability on both app and downloader-extension targets. The signed App Groups entitlement key is `com.apple.security.application-groups`; both targets must share the same group.
- Unmanaged hosts declare `BAManifestURL` and, as required for their download policy, `BAInitialDownloadRestrictions`, `BAEssentialMaxInstallSize`, and `BAMaxInstallSize`.
- Managed hosts declare `BAAppGroupID` and `BAHasManagedAssetPacks`; Apple-hosted managed hosts also declare `BAUsesAppleHosting` and omit the other Background Assets plist keys as documented.
- A downloader extension target must use the protocol that matches its hosting mode. Extension principal-class and target framework-link setup are host responsibilities.

Do not claim that the App Groups entitlement alone grants Background Assets operation. The host must have the right extension, matching plist/manifest setup, valid app-group signing, and (for Apple hosting) an eligible upload/distribution configuration.

## Status/value facade feasibility

A narrow `current_download_queue_count()`-style API could honestly report the number of entries returned by `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler` at the callback's observation point. This operation only queries the queue; the documented method does not schedule or cancel a download. Define its value as a transient snapshot, not a stable count: another app/extension operation or the system may change the queue immediately after it is read. `fetchCurrentDownloadsWithCompletionHandler` reports errors and invokes its completion on the manager's completion queue. The docs also provide `performWithExclusiveControl` so the app and extension can avoid simultaneous manager operations; a future contract must decide whether and how queue snapshots use that coordination.

For an especially narrow v1 value, B97 counts the returned array without reading any `BADownload` properties. B124 adds a count of objects with `isEssential == true`; B127 adds a count whose `priority` differs from `BADownloaderPriorityDefault`. Both avoid the generated non-atomic, potentially non-thread-safe `state`, `identifier`, and `uniqueIdentifier` getters. B130 and B133 read only the callback-supplied `BAAppExtensionInfo` readonly atomic allowance properties and copy their native `Int?` values. B136 maps the callback's typed enum and preserves unknown raw values. None calls the manager from that extension callback, constructs downloads, or implements the extension protocol.

This value is not:

- a device or OS support query;
- proof that the host has an enabled/valid Background Download extension or signed App Group;
- a count of downloaded, installed, or usable asset files;
- a per-asset availability result;
- a promise about download timing, completion, install state, or future availability.

A separate managed-pack API could return local availability for one caller-supplied pack ID after iOS 26.4, but its empty status is ambiguous for unknown/not-downloaded packs and the current 0.3.2 typed binding omits the relevant methods. It should remain deferred until the binding and host baseline are validated.

## D72 feasibility result and follow-up checks

A read-only queue-count snapshot was an honest, bounded candidate if a later implementation explicitly scoped row 036 to app/extension queue observation. D72 did not support a generic portable `BackgroundAssetsAvailable` Boolean and did not authorize source code; B97, B124, B127, B130, B133, and B136 are later, separately scoped implementations. B97 adds the unmanaged queue count, B124 adds an essential-entry subset count, B127 adds a non-default-priority subset count, B130/B133 copy the callback's general and essential allowance values, B136 classifies its typed request value, and B110/B113/B116 add caller-supplied manifest inspection and metadata to row 036 partial (`B`); none validates host setup or readiness.

At D72, the following checks remained before any implementation:

1. Select unmanaged iOS 16.1 queue observation or managed iOS 26.4 per-pack local status; do not combine their semantics.
2. For the queue snapshot, verify 0.3.2 feature wiring and device/Simulator compile/link, callback block lifetime, error mapping, cancellation/drop behavior, and manager completion queue. Define app/extension concurrency and whether `performWithExclusiveControl` is required.
3. Avoid `BADownload.state` and other nonatomic properties unless a thread-safety proof exists; preserve unknown native state values if a later API exposes them.
4. Verify app-group entitlement/profile plus exact extension, plist, manifest, and hosting-mode setup in a host app. A crate compile cannot establish that configuration.
5. If using managed-pack local status, verify an updated typed binding for the iOS 26.4 methods, local-status behavior for downloaded/out-of-date/obsolete packs, unknown-ID ambiguity, and async completion ownership.
6. Validate actual background scheduling, installation, downloads, and asset reads only in a configured app and extension with representative packs; those behaviors are outside a status snapshot and must not be inferred from compile/link checks.

## Still deferred after B97, B110, B113, B116, B119, B124, B127, B130, B133, B136, B139, and B142

- No capability matrix/status JSON, CI, aggregate plan, or shared index edit in this workstream.
- No portable BackgroundAssets availability contract or broad iOS backend; B97 adds the unmanaged queue count, B124 counts queue entries marked essential, B127 counts those whose priority differs from Apple's default, B110 counts entries in caller-supplied JSON, B113 copies selected metadata from those entries, B116 copies optional custom JSON metadata bytes, B130/B133 copy the allowance values from a caller-supplied callback object, and B136 classifies the typed request reason. B119 records why no further safe SDK 26.5 asset-pack field is available. No downloader-extension protocol implementation, app-group entitlement, Info.plist or host manifest setup inspection, download scheduling/cancel API, progress delegate, local-installation query, or asset read API.
- No tests, probe execution, live download, or device lifecycle action.

## B97 follow-up — unmanaged queue-count snapshot

B97 implements the D72 queue-count candidate in `platform/ios/ios-background-assets`. The public
Rust call `request_download_queue_count()` wraps
`BADownloadManager.fetchCurrentDownloadsWithCompletionHandler:` and returns only the callback
array length as `DownloadQueueCount`. It does not inspect `BADownload` objects or their nonatomic
properties. Apple lists the method at iOS 16.1 in the installed SDK header. The method reads the
unmanaged manager queue for the app or its extension; it does not query managed `BAAssetPack`
state.

The future starts native work when called. Its callback copies the count or the native error
domain and `NSInteger` code into Rust-owned values. A dropped future detaches Rust interest but
does not cancel the native request. A successful zero count means only that the callback's queue
snapshot contained no entries. It is not a support, host-configuration, asset-installation, or
asset-availability result. The backend neither schedules nor cancels downloads and reads no asset
bytes.

The new focused guide is [docs/ios/background-assets.md](docs/ios/background-assets.md). B97
requires iOS 16.1, B124 requires iOS 16.4, B127 requires iOS 16.1, B130 requires iOS 16.1, B133
requires iOS 16.4, B136 uses the callback's iOS 16.1 `BAContentRequest` value, and B110/B113/B116
require iOS 26.0; none makes a Swift ABI call. These
operations do not add portable Background Assets semantics, managed asset-pack status,
downloader-extension protocol implementation, entitlement or host-plist inspection, or a broad
capability-matrix edit in the workstream. Root marks row 036 `B` for B97's total queue count,
B124's essential-entry count, B127's non-default-priority count, and B110/B113/B116's
caller-supplied manifest count and metadata, plus B130/B133's copied callback allowance values
and B136's typed callback-reason classification;
no broad support or setup claim is made.

On Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, host/device/Simulator checks,
device/Simulator strict library Clippy, device/Simulator rustdoc, package formatting, and
`cargo +1.94.1 xtask docs-check` passed. Xcode 26.6 remains below the repo's Xcode 27.x baseline.
No tests, example probe, app, live queue query, or download lifecycle ran. See the focused API guide
for scope and Apple references.

## B110 follow-up — unmanaged manifest entry count

B110 adds `inspect_asset_pack_manifest(json, app_group_id)` to the B97 `ios-background-assets`
package. The iOS 26.0+ function parses caller-owned JSON with
`BAAssetPackManifest.initFromData:applicationGroupIdentifier:error:` and returns only the count of
its `assetPacks` set. It uses the generated `objc2-background-assets` 0.3.2 bindings with
`BAAssetPack` and `BAAssetPackManifest`; it does not use a raw selector or any Swift ABI.

The installed Xcode 26.6 build 17F113 / iOS SDK 26.5 `BAAssetPackManifest.h` marks the class and
initializer available from iOS 26.0, defines the initializer as a JSON-data-to-memory operation,
and declares `assetPacks` `readonly, copy`. The Rust wrapper copies bytes into `NSData`, converts
the group ID to `NSString`, holds the parsed manifest alive while it reads the returned set's safe
`count()`, then returns a fixed-width Rust value. The native parse error keeps its domain and
`NSInteger` code.

This is a manifest-description count, not a status or asset-availability result. It does not touch
`BAAssetPackManager.sharedManager()`, whose header says its first access opts the app into automatic
management and that a missing matching downloader-extension protocol is a programmer error. B110
does not query local state, access asset bytes, schedule downloads, check the app-group entitlement,
or validate a host target. The app-group ID is passed to Apple's parser for this manifest
representation. The focused guide is [docs/ios/background-assets.md](docs/ios/background-assets.md).

Rust 1.94.1 host/device/Simulator checks, strict library Clippy, and rustdoc passed for the package;
package formatting and `cargo +1.94.1 xtask docs-check` also passed. Xcode 26.6 build 17F113 / iOS
SDK 26.5 remains below the Xcode 27.x baseline. No tests or parser invocation ran; no app, probe,
device, or manager call ran. Exact static gates:

```sh
cargo +1.94.1 check --locked -p ios-background-assets
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-background-assets -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-assets/Cargo.toml -- --check
cargo +1.94.1 xtask docs-check
```

## B113 follow-up — owned manifest metadata

B113 adds `inspect_asset_pack_manifest_entries(json, app_group_id)` and
`AssetPackManifestEntry` in the same package. It parses the caller's JSON by the B110 route, reads
the parsed set's `BAAssetPack.identifier`, `downloadSize`, and `version`, then copies the values to
Rust-owned data. The output is sorted by identifier, version, and download size because Apple's
`NSSet` order is arbitrary. `downloadSize` is checked as a nonnegative byte count and exposed as
`u64`; `version` preserves the native `NSInteger` value as `isize`.

The installed iOS 26.5 `BAAssetPack.h` marks these fields readonly; the properties omit
`nonatomic`, so the declared Objective-C accessors are atomic by default. The class is
`NS_SWIFT_SENDABLE`. B113 invokes the generated typed 0.3.2 accessors only on retained pack objects
from the parsed, readonly-copy manifest set and copies all values in the same synchronous call.
Although Apple notes a server-provided `BAAssetPack` may become invalid after a server update, this
path does not create a manager, fetch server state, or retain those objects past the parse call.

B113 describes only the supplied manifest. It does not report local download/install state, server
freshness, entitlement, app-group validity, or extension setup. It does not schedule, update, or
remove a pack, read its bytes, use raw selectors, or call Swift ABI. The manifest parser and pack
class require iOS 26.0. Focused guide: [docs/ios/background-assets.md](docs/ios/background-assets.md).

Rust 1.94.1 host/device/Simulator checks, strict library Clippy, and rustdoc passed; package
formatting and `cargo +1.94.1 xtask docs-check` passed. Xcode 26.6 build 17F113 / iOS SDK 26.5 is
below the Xcode 27.x baseline. No tests, parser invocation, app, probe, device query, or manager
call ran. Exact B113 gates:

```sh
cargo +1.94.1 check --locked -p ios-background-assets
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-background-assets -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-assets/Cargo.toml -- --check
cargo +1.94.1 xtask docs-check
```

## B116 follow-up — optional custom JSON metadata bytes

B116 extends `AssetPackManifestEntry` with optional Rust-owned `user_info_json` bytes and exposes
them through `user_info_json_bytes()`. The installed iOS 26.5 SDK's `BAAssetPack.h` declares
`@property (nullable, readonly, copy) NSData* userInfo;` and documents the data as JSON-encoded
custom information; the header says the value is `nil` for Apple-hosted asset packs. Apple's
`BAAssetPack` documentation gives the same JSON-encoded contract. The declaration omits
`nonatomic`, so the Objective-C property is atomic by default, and the class is
`NS_SWIFT_SENDABLE`.

The available `objc2-background-assets` 0.3.2 generated binding exposes
`BAAssetPack::userInfo(&self) -> Option<Retained<NSData>>`; `objc2-foundation::NSData::to_vec()`
copies its contents to Rust-owned bytes. B116 uses that typed getter on the retained pack from the
parsed, readonly-copy local manifest set and copies the data before dropping the pack. `None` is
preserved. The facade does not parse or validate the JSON and does not infer server freshness,
download state, or local installation. It does not query a manager, use a raw selector, or call
Swift ABI. This manifest inspection requires iOS 26.0.

Rust 1.94.1 host/device/Simulator checks, strict library Clippy, and rustdoc passed; package
formatting and `cargo +1.94.1 xtask docs-check` passed. Xcode 26.6 build 17F113 / iOS SDK 26.5 is
below the Xcode 27.x baseline. No tests, parser invocation, app, probe, device query, or manager
call ran. Exact B116 gates:

```sh
cargo +1.94.1 check --locked -p ios-background-assets
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-background-assets -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-assets/Cargo.toml -- --check
cargo +1.94.1 xtask docs-check
```

## B119 audit — no additional SDK 26.5 asset-pack field

B119 is a no-go for another distinct typed `BAAssetPack` metadata field with the installed iOS
26.5 SDK and `objc2-background-assets` 0.3.2. The installed `BAAssetPack.h` exposes `identifier`,
`downloadSize`, `version`, and `userInfo` as readonly data properties; B113 and B116 already copy
all four into owned Rust metadata. The header has no `language` property, and the generated 0.3.2
`BAAssetPack` binding has no `language` getter.

Apple's `BAAssetPack.language` documentation is marked beta, and Apple's localized-assets article
states asset-pack language support begins in iOS 27 and Xcode 27. It is not available in the
installed iOS 26.5 SDK or the current typed binding, so it cannot be added through the permitted
safe API path here. The installed header's `download()` and manifest `allDownloads` methods create
`BADownload` objects that can be scheduled; they are not additional read-only metadata snapshots
and remain out of scope. No package change or runtime call was made for B119.

## B124 follow-up — essential queue-entry count

B124 adds `request_essential_download_queue_count()` to the B97 package. It uses the existing
asynchronous `BADownloadManager.fetchCurrentDownloadsWithCompletionHandler:` query and counts only
returned `BADownload` objects whose `isEssential` getter returns `true`. The result is a transient
count of matching entries in that callback snapshot, not a count of installed assets, every
essential asset on device, or downloads outside the returned queue. This API requires iOS 16.4;
B97's total queue count remains available from iOS 16.1.

The installed iOS 26.5 `BADownload.h` declares
`@property (readonly) BOOL isEssential API_AVAILABLE(... ios(16.4) ...)` and marks `BADownload`
`NS_SWIFT_SENDABLE`. The property omits `nonatomic`, so its getter is atomic by default. Generated
`objc2-background-assets` 0.3.2 exposes typed `BADownload::isEssential(&self) -> bool` under the
existing `BADownload` feature; the binding does not mark the getter nonatomic. The Rust callback
retains each array object during iteration and invokes only this getter after the iOS 16.4 guard.
It does not inspect the nonatomic `state`, `identifier`, or `uniqueIdentifier` properties, change
the queue, use raw selectors, or call Swift ABI.

Rust 1.94.1 host/device/Simulator checks, strict library Clippy, and rustdoc passed; package
formatting and `cargo +1.94.1 xtask docs-check` passed. Xcode 26.6 build 17F113 / iOS SDK 26.5 is
below the Xcode 27.x baseline. No tests, parser invocation, app, probe, device query, or runtime
manager call ran. Exact B124 gates:

```sh
cargo +1.94.1 check --locked -p ios-background-assets
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-background-assets -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-assets/Cargo.toml -- --check
cargo +1.94.1 xtask docs-check
```

## B127 follow-up — non-default-priority queue-entry count

B127 adds `request_nondefault_priority_download_queue_count()` to the B97 package. It uses the
same asynchronous queue snapshot and counts returned `BADownload` objects whose typed
`priority` getter differs from `BADownloaderPriorityDefault`. Apple documents priority as the
download execution priority; the default is the recommended value when priority does not matter.
The API counts only this exact subset, including values above or below the default; it does not
report a full per-download priority list or claim a particular schedule time.

The installed iOS 26.5 `BADownload.h` declares `@property (readonly) BADownloaderPriority
priority` and exposes `BADownloaderPriorityDefault` from iOS 16.1. The property omits `nonatomic`,
so its getter is atomic by default. Generated `objc2-background-assets` 0.3.2 exposes
`BADownload::priority(&self) -> BADownloaderPriority` and the typed default constant under the
existing `BADownload` feature; the generated accessor has no nonatomic warning. The queue array
iterator retains each object during the read. B127 does not inspect `state`, `identifier`, or
`uniqueIdentifier`, schedule or cancel downloads, use raw selectors, or call Swift ABI.

Rust 1.94.1 host/device/Simulator checks, strict library Clippy, and rustdoc passed; package
formatting and `cargo +1.94.1 xtask docs-check` passed. Xcode 26.6 build 17F113 / iOS SDK 26.5 is
below the Xcode 27.x baseline. No tests, parser invocation, app, probe, device query, or runtime
manager call ran. Exact B127 gates:

```sh
cargo +1.94.1 check --locked -p ios-background-assets
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-background-assets -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-background-assets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios
cargo +1.94.1 doc --locked --no-deps -p ios-background-assets --target aarch64-apple-ios-sim
cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-assets/Cargo.toml -- --check
cargo +1.94.1 xtask docs-check
```

## B130 follow-up — general extension allowance snapshot

B130 adds the iOS-only safe Rust facade
`inspect_restricted_download_size_remaining(&BAAppExtensionInfo) -> Result<Option<u64>, ExtensionDownloadAllowanceError>`.
The caller passes the object supplied by Apple's downloader-extension callback. `None` preserves
Apple's unrestricted signal; `Some(bytes)` is the remaining number of bytes that may be scheduled
before app launch. A negative native `Int` is rejected as `InvalidRemainingBytes`. The facade
does not instantiate `BAAppExtensionInfo`, parse the manifest, or schedule downloads.

The inspected SDK 26.5 `BAAppExtensionInfo.h` declares the class from iOS 16.1 and the property
`@property (readonly, strong, nullable) NSNumber *restrictedDownloadSizeRemaining`. The property
omits `nonatomic`, so the getter is atomic by default. Generated
`objc2-background-assets` 0.3.2 exposes the typed unsafe getter
`restrictedDownloadSizeRemaining(&self) -> Option<Retained<NSNumber>>` with the
`BAAppExtensionInfo` feature, whose feature dependencies include Foundation `NSValue`. The
binding's `BADownloaderExtension` typed method receives `&BAAppExtensionInfo`; the class header
marks `init` and `new` unavailable. Apple documents the property as `Int?` and `nil` when downloads
are not restricted. The Rust wrapper therefore borrows the live callback object, checks iOS 16.1,
calls the typed getter, reads `NSNumber.integerValue`, checks the signed value as `u64`, and returns
only owned Rust data.

## B133 follow-up — essential extension allowance snapshot

B133 adds the distinct iOS-only facade
`inspect_restricted_essential_download_size_remaining(&BAAppExtensionInfo) -> Result<Option<u64>, ExtensionDownloadAllowanceError>`.
It copies the essential remaining-byte value from the same callback object and uses the same nil
and nonnegative-value contract; its property is available from iOS 16.4. It does not infer a
download schedule or mutate the extension queue.

The installed SDK declares
`@property (readonly, strong, nullable) NSNumber *restrictedEssentialDownloadSizeRemaining`
with `API_AVAILABLE(... ios(16.4) ...)`; the property also omits `nonatomic`. The 0.3.2 typed
binding exposes `restrictedEssentialDownloadSizeRemaining(&self) -> Option<Retained<NSNumber>>`
under `BAAppExtensionInfo`. Apple's Swift reference calls it `Int?` and documents `nil` when
downloads are not restricted. The wrapper has an explicit iOS 16.4 guard and copies only the
`NSNumber.integerValue` into `Option<u64>`.

## B136 follow-up — typed extension request classification

B136 adds `classify_extension_content_request(BAContentRequest) -> ExtensionContentRequestKind`
behind the iOS target cfg. It maps the three SDK 26.5 names—install, update, periodic—to Rust
variants. Any unmatched value is returned as `Unknown(NSInteger)`, preserving a future system
value without claiming its meaning. The function is a typed enum conversion only; it sends no
Objective-C message and accepts no selector or raw pointer.

The iOS 26.5 SDK `BATypes.h` declares `BAContentRequest` as `NS_ENUM(NSInteger, ...)` with
`BAContentRequestInstall = 1`, followed by update and periodic values. `BADownloaderExtension.h`
declares the protocol from iOS 16.1 and passes the request value to
`downloadsForRequest:manifestURL:extensionInfo:`. Generated `objc2-background-assets` 0.3.2
exposes `BAContentRequest(pub NSInteger)` and `Install`, `Update`, and `Periodic` under `BATypes`;
the typed `BADownloaderExtension` method uses this value type. Although current Apple online docs
list a beta language-change request, that enumerator is absent from the installed SDK and 0.3.2
binding, so B136 preserves it as unknown rather than exposing it as a known case.

B130, B133, and B136 compile with Rust 1.94.1 for host, `aarch64-apple-ios`, and
`aarch64-apple-ios-sim`; strict library Clippy, rustdoc, package format, and
`cargo +1.94.1 xtask docs-check` passed. Xcode 26.6 build 17F113 / iOS SDK 26.5 remains below the
repository's Xcode 27.x baseline. No tests, extension callback, manager call, probe, app, device
query, or download lifecycle ran. None of these operations uses a raw selector or Swift ABI.

## B139 audit — no additional typed error classification

No API was added. The iOS 26.5 SDK `BAError.h` marks the global `BAErrorDomain` available from iOS
17.0. Generated `objc2-background-assets` 0.3.2 exposes this global and the typed `BAErrorCode`
only under feature `BAError`; its generated source declares the domain as an external static
`NSString`. Existing `NativeBackgroundAssetsError` copies the native domain and `NSInteger` code
from any `NSError`, so interpreting its code as a Background Assets code first requires an exact
domain match. The public header documents the domain symbol but does not give its string value.

Because this crate's queue surface starts at iOS 16.1 and must preserve arbitrary native error
domains, B139 does not invent a domain literal or add an iOS 17-only external static-symbol path
whose older-runtime weak-link behavior is not verified. The existing owned domain/code remain
the exact, forward-compatible result. This no-go adds no package feature and does not use runtime
error queries.

## B142 audit — no managed asset-pack status query

No API was added. The installed iOS 26.5 SDK declares
`BAAssetPackManager.getStatusRelativeToAssetPack:completionHandler:`,
`getLocalStatusOfAssetPackWithIdentifier:completionHandler:`, and
`assetPackIsAvailableLocallyWithIdentifier:` from iOS 26.4. The 0.3.2 generated manager binding
omits all three. It exposes only the older
`getStatusOfAssetPackWithIdentifier:completionHandler:`, which the SDK deprecates at iOS 26.4 in
favor of `getStatusRelativeToAssetPack:` and documents as potentially fetching latest server state.
The local-status method also cannot distinguish an unknown pack ID from a known but not-downloaded
pack, per the SDK documentation.

Calling `BAAssetPackManager.sharedManager()` opts the host into automatic managed-pack handling;
the generated binding docs and SDK header warn that not adopting the corresponding managed
downloader-extension protocol is a programmer error. This makes a generic managed status query
unsafe for this unmanaged package even apart from the missing current typed APIs. B142 leaves the
status query deferred until a current typed binding and host-specific managed-extension contract
are available. No manager call, app, or device query ran.

## Apple and binding references

- [Background Assets overview](https://developer.apple.com/documentation/backgroundassets)
- [Configuring an unmanaged Background Assets project](https://developer.apple.com/documentation/backgroundassets/configuring-an-unmanaged-background-assets-project)
- [Downloading essential assets in the background](https://developer.apple.com/documentation/backgroundassets/downloading-essential-assets-in-the-background)
- [Downloading Apple-hosted asset packs](https://developer.apple.com/documentation/backgroundassets/downloading-apple-hosted-asset-packs)
- [Creating managed asset packs](https://developer.apple.com/documentation/backgroundassets/creating-managed-asset-packs)
- [BADownloadManager](https://developer.apple.com/documentation/backgroundassets/badownloadmanager)
- [BADownload `isEssential`](https://developer.apple.com/documentation/backgroundassets/badownload/isessential)
- [BADownload `priority`](https://developer.apple.com/documentation/backgroundassets/badownload/priority-swift.property)
- [BADownloaderPriorityDefault](https://developer.apple.com/documentation/backgroundassets/badownload/priority-swift.struct/default)
- [BAAppExtensionInfo](https://developer.apple.com/documentation/backgroundassets/baappextensioninfo)
- [restrictedDownloadSizeRemaining](https://developer.apple.com/documentation/backgroundassets/baappextensioninfo/restricteddownloadsizeremaining)
- [restrictedEssentialDownloadSizeRemaining](https://developer.apple.com/documentation/backgroundassets/baappextensioninfo/restrictedessentialdownloadsizeremaining-5r8v0)
- [BADownloaderExtension downloads(for:manifestURL:extensionInfo:)](https://developer.apple.com/documentation/backgroundassets/badownloaderextension-qwaw/downloads%28for%3Amanifesturl%3Aextensioninfo%3A%29)
- [BAContentRequest](https://developer.apple.com/documentation/backgroundassets/bacontentrequest)
- [BAErrorDomain](https://developer.apple.com/documentation/backgroundassets/baerrordomain)
- [BAErrorCode](https://developer.apple.com/documentation/backgroundassets/baerrorcode)
- [BAAssetPackManager](https://developer.apple.com/documentation/backgroundassets/baassetpackmanager)
- [BAAssetPackManifest](https://developer.apple.com/documentation/backgroundassets/baassetpackmanifest)
- [BAAssetPackManifest `initFromData:applicationGroupIdentifier:error:`](https://developer.apple.com/documentation/backgroundassets/baassetpackmanifest/initfromdata%3Aapplicationgroupidentifier%3Aerror%3A)
- [BAAssetPackManifest `assetPacks`](https://developer.apple.com/documentation/backgroundassets/baassetpackmanifest/assetpacks)
- [BAAssetPack](https://developer.apple.com/documentation/backgroundassets/baassetpack)
- [BAAssetPack `identifier`](https://developer.apple.com/documentation/backgroundassets/baassetpack/identifier)
- [BAAssetPack `downloadSize`](https://developer.apple.com/documentation/backgroundassets/baassetpack/downloadsize)
- [BAAssetPack `version`](https://developer.apple.com/documentation/backgroundassets/baassetpack/version)
- [BAAssetPack `userInfo`](https://developer.apple.com/documentation/backgroundassets/baassetpack)
- [BAAssetPack `language` (beta)](https://developer.apple.com/documentation/backgroundassets/baassetpack/language)
- [Reducing download and storage demands with localized asset packs](https://developer.apple.com/documentation/backgroundassets/reducing-download-and-storage-demands-with-localized-asset-packs)
- [BAAssetPackStatus](https://developer.apple.com/documentation/backgroundassets/baassetpackstatus)
- [App Groups entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups)
- [`objc2-background-assets` 0.3.2 crate](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/)
- [`BADownloadManager` binding](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/struct.BADownloadManager.html)
- [`BADownload` binding](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/struct.BADownload.html)
- [`BAAssetPackManager` binding](https://docs.rs/objc2-background-assets/0.3.2/objc2_background_assets/struct.BAAssetPackManager.html)

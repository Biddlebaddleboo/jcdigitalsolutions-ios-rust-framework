# iOS MediaPlayer library authorization status

`ios-media-library-status` exposes one iOS-only snapshot:

```rust,ignore
let status = ios_media_library_status::media_library_authorization_status();
```

It calls `+[MPMediaLibrary authorizationStatus]` and maps the documented values to
`MediaLibraryAuthorizationStatus::{NotDetermined, Denied, Restricted, Authorized}`. An unknown
native integer remains available as `Unknown(i64)` and through `raw_value()`. Apple defines
`Restricted` separately from `Denied`: an app may access some library content in the restricted
state. The value is a snapshot; query again when current authorization matters.

The query is synchronous and does not request access, show the authorization UI, create a
`MPMediaLibrary` object, read library items, or contact Apple Music catalog or service endpoints.
It reports permission state only; it does not establish that a subscription, catalog, track, or
playback service is available. No async callback, cancellation, retained native object, or native
handle crosses the API. The wrapper makes no thread-safety claim.

The inspected Xcode 26.6 build 17F113 / iOS 26.5 SDK header
`MediaPlayer.framework/Headers/MPMediaLibrary.h` declares the enum and `authorizationStatus` from
iOS 9.3. The host must use a compatible deployment target before calling this function; this crate
sets no deployment target. The status-only query does not need an entitlement. This wrapper
neither requests access nor reads items. Apple requires a host app
that requests access to, or reads, the media library to provide the exact
`NSAppleMusicUsageDescription` key. This crate neither edits nor checks the host app's property
list.

The crate enables `objc2-media-player` 0.3.2 with default features off and only `MPMediaLibrary`.
That generated binding exposes `MPMediaLibrary::authorizationStatus()` and the four known status
constants. Its `requestAuthorization:` binding is gated by the disabled `block2` feature. No
portable facade or non-iOS function is provided.

Apple references: [authorization status values](https://developer.apple.com/documentation/mediaplayer/mpmedialibraryauthorizationstatus?language=objc), [`authorizationStatus`](https://developer.apple.com/documentation/mediaplayer/mpmedialibrary/authorizationstatus%28%29?language=objc), [`NSAppleMusicUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsapplemusicusagedescription), and [Media Player privacy requirements](https://developer.apple.com/documentation/mediaplayer/). See the [capability plan](../../PLAN_CAPABILITIES_MEDIA_LIBRARY_STATUS.md), [iOS plan](../../PLAN_IOS_MEDIA_LIBRARY_STATUS.md), and [validation plan](../../PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md).

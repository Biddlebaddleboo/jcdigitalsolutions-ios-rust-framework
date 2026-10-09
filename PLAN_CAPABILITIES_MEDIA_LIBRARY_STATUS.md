# PLAN_CAPABILITIES_MEDIA_LIBRARY_STATUS.md — Workstream D65: Media Library Authorization Snapshot

## Objective

Add one bounded iOS-only status query for the user's MediaPlayer library as a partial slice of row 090. Do not claim Apple Music catalog, subscription, service, item-read, or playback support.

## Write scope

- `PLAN_CAPABILITIES_MEDIA_LIBRARY_STATUS.md`
- `platform/ios/ios-media-library-status/**`
- `docs/ios/media-library-status.md`

Root owns workspace and lock integration, canonical capability rows, shared indexes, CI, and aggregate plans. Do not edit those paths in this slice.

## Contract boundary

- Expose `ios_media_library_status::media_library_authorization_status()` on iOS only.
- Preserve `NotDetermined`, `Denied`, `Restricted`, and `Authorized`; preserve any unknown native status in signed 64-bit form.
- Treat the result as a synchronous point-in-time snapshot.
- Provide no portable contract or non-iOS fallback.
- Do not request authorization, display a prompt, create a media-library object, read media items, contact Apple Music services, play content, or expose native handles.
- Make no thread-safety or runtime permission-state claim.

## SDK and binding evidence

- The inspected Xcode 26.6 build 17F113 / iOS 26.5 iPhoneOS SDK's `MediaPlayer.framework/Headers/MPMediaLibrary.h` marks both the authorization enum and `+[MPMediaLibrary authorizationStatus]` available from iOS 9.3.
- The host app must use an iOS deployment target compatible with that API; the crate sets no deployment target.
- `objc2-media-player = 0.3.2` exposes the query and four status constants under `MPMediaLibrary`. Its `requestAuthorization:` binding requires `block2`, which this crate does not enable.
- A host that requests library access or reads library items must provide `NSAppleMusicUsageDescription`; this package does neither and does not inspect the host property list.

## Acceptance

- Device and Simulator library checks compile the selected generated API.
- Strict Clippy, rustdoc, formatting, feature isolation, and `git diff --check` pass.
- No tests, link probes, permission request, prompt, media access, network request, or playback run as part of this slice.
- The root-owned row 090 may describe this as an iOS `B` partial only after integration; Apple Music service support remains unimplemented.

## Status

D65 is implemented in `platform/ios/ios-media-library-status` with its guide at `docs/ios/media-library-status.md`. Device and Simulator checks, strict Clippy, and iOS rustdoc passed in a temporary isolated Cargo workspace, then `sh platform/ios/ios-media-library-status/check.sh` passed under the integrated root lock on 2026-10-08. Root owns the `Cargo.lock` and workspace dependency integration.

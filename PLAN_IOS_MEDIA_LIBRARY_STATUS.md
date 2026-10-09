# PLAN_IOS_MEDIA_LIBRARY_STATUS.md — Workstream B71: MediaPlayer Library Status Adapter

## Objective

Implement D65's point-in-time MediaPlayer authorization-status query through the public iOS API only.

## Backend behavior

- Call `MPMediaLibrary::authorizationStatus()` through `objc2-media-player 0.3.2`.
- Map native values 0–3 to `NotDetermined`, `Denied`, `Restricted`, and `Authorized`; preserve other signed values as `Unknown(i64)`.
- Keep the method synchronous and object-free; no native object, pointer, or identifier escapes.
- Use the iOS 9.3 API floor from the inspected iOS 26.5 SDK header. The wrapper sets no deployment target and callers must not use the call below the API floor.

## Dependency and privacy boundary

- Enable only `objc2-media-player`'s `MPMediaLibrary` feature with default features disabled.
- Do not enable `block2`, `requestAuthorization:`, `defaultMediaLibrary`, item, query, or playback APIs.
- Do not request access, show a prompt, inspect media items, access a catalog, or send a service request.
- Apple requires `NSAppleMusicUsageDescription` for host behavior that requests access to or reads the media library. This wrapper does not do either or edit the host property list.
- Add no entitlement, Swift source, portable facade, runtime permission claim, or thread-safety promise.

## Write scope

- `PLAN_IOS_MEDIA_LIBRARY_STATUS.md`
- `platform/ios/ios-media-library-status/**`
- `docs/ios/media-library-status.md`

Root owns workspace/lock integration, canonical capability metadata, CI, shared indexes, and aggregate plans.

## Status

B71 is implemented as `media_library_authorization_status() -> MediaLibraryAuthorizationStatus`. The adapter confines unsafe code to the no-argument generated Objective-C class-method call and converts the `NSInteger` result to the fixed-width public raw value. Compile and lint evidence is recorded in [G65](PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md).

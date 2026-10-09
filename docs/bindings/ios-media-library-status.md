# Optional iOS MediaPlayer status C API

The opt-in `framework-c-api` Cargo feature `ios-media-library-status` enables
`bindings/c/include/framework_ios_media_library_status.h`. It exposes D65/B71's
point-in-time `+[MPMediaLibrary authorizationStatus]` query through one C scalar
output. The feature depends only on the existing `ios-media-library-status` crate.

## Result contract

`framework_ios_media_library_authorization_status` returns a
`FrameworkStatus` and writes the raw signed 64-bit MediaPlayer authorization status
to `out_raw_status`. Known values are 0 (`NOT_DETERMINED`), 1 (`DENIED`), 2
(`RESTRICTED`), and 3 (`AUTHORIZED`). Any other native signed value passes through
unchanged. The output is initialized to zero before work; read it only when the
function returns `FRAMEWORK_STATUS_OK`.

Null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`. On non-iOS, the C stub
returns `FRAMEWORK_STATUS_UNSUPPORTED` and leaves the output zero; there is no fake
authorization value or MediaPlayer dependency in the host feature graph. A Rust panic
is contained as `FRAMEWORK_STATUS_PANIC`.

## Platform and privacy limits

Apple declares `+[MPMediaLibrary authorizationStatus]` from iOS 9.3. Use an iOS 9.3 or
later deployment target or an explicit runtime availability guard before calling it.
The crate and C wrapper do not add a runtime guard. The focused link gate uses iOS
10.0 for the arm64 device target and iOS 14.0 for the arm64 Simulator target; those
are validation floors, not a change to the API's 9.3 floor.

The query does not request access, show a prompt, create an `MPMediaLibrary` object,
read media items, or contact Apple Music catalog/service APIs. It reports saved
authorization status only; it does not establish catalog, subscription, or playback
availability. The API makes no thread-safety promise and imposes no main-thread rule.
This wrapper does not need or inspect `NSAppleMusicUsageDescription`; a host that
requests authorization or reads library items has separate privacy requirements.

See [`docs/ios/media-library-status.md`](../ios/media-library-status.md) for B71's
typed Rust API, SDK evidence, and exact platform scope. F10's C ABI gate and evidence
are in `PLAN_BINDINGS_MEDIA_LIBRARY_STATUS.md`.

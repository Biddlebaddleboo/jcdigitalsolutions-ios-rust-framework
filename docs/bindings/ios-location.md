# iOS location C ABI

F15 adds the opt-in `ios-location` feature to `framework-c-api`. The C layer owns one capability-scoped operation handle and polls the existing B5 `IosLocationBackend`; it does not add a second Core Location implementation, global registry, or shared executor

## API and tags

Include `framework_ios_location.h`. The header exposes one availability query and three operation starts:

- `framework_ios_location_availability` reads Core Location service availability without requesting authorization
- `framework_ios_location_authorization_query_start` reads authorization without prompting
- `framework_ios_location_authorization_request_start` explicitly requests foreground authorization and may show Apple's permission UI
- `framework_ios_location_current_start` requests one current fix with a desired horizontal-accuracy target in meters; the target is a preference, not a guarantee

Each start returns one unique `FrameworkIosLocationOperation` handle. `framework_ios_location_operation_poll` returns pending or consumes one terminal result. `framework_ios_location_operation_cancel` drops a pending future. `framework_ios_location_operation_destroy` clears the original writable handle slot and drops the operation

Availability tags are fixed from 0 through 5. B5 currently reports `AVAILABLE` or `TEMPORARILY_UNAVAILABLE`; other tags are reserved for forward compatibility. Authorization tags are `UNKNOWN` 0, `NOT_DETERMINED` 1, `DENIED` 2, `RESTRICTED` 3, `FOREGROUND` 4, and `BACKGROUND` 5. An unknown future native authorization state maps to `UNKNOWN`. Operation-kind tags are authorization query 0, authorization request 1, and current request 2

## Result ABI

`FrameworkIosLocationResultV1` is output-only. On 64-bit iOS it has size 64 and alignment 8. Field offsets are `struct_size` 0, `abi_version` 4, `operation_kind` 8, `status` 12, `authorization` 16, `reserved` 20, `latitude_degrees` 24, `longitude_degrees` 32, `horizontal_accuracy_meters` 40, `timestamp_unix_millis` 48, `native_code` 56, and `reserved2` 60

The record is fully initialized on every valid poll call. Inspect semantic fields only when the API call returns `FRAMEWORK_STATUS_OK` and `out_ready` is 1. The record's `status` is the terminal operation result; an operation error is data in the result record, not a failure of the poll API call. A ready result is consumed once; a second poll returns `FRAMEWORK_STATUS_NOT_FOUND`

Pointer preconditions are caller obligations. Every output pointer must name valid storage with the required natural alignment and writable extent for the full call. The availability output must not overlap a live iOS operation handle; a start's output slot must not contain a live handle on entry or overlap live operation storage. Poll requires one live unique iOS handle and two aligned writable outputs that do not overlap each other or the handle. Cancel requires that same unique live handle. Destroy accepts null or the original writable handle slot returned by start; do not copy or alias a handle, or race any operation with destroy

Successful authorization results set only `authorization`. Successful current results set the WGS-84 coordinate, horizontal-accuracy estimate in meters, and Unix measurement timestamp in milliseconds. Failure results keep those value fields zero and preserve a representable nonzero native Core Location error code when present. `LocationError::Backend` maps through `FrameworkStatus::from_error`; invalid accuracy maps to `FRAMEWORK_STATUS_INVALID_ARGUMENT`

## Readiness callback and operation lifecycle

The required callback is a readiness notification only. It carries no result; poll the matching handle after the callback returns. The callback runs on the main thread and may run inline before a start function returns for an immediately-ready authorization query or result. It may also run during a poll if B5 wakes while that poll is active. The callback must not unwind or re-enter any F15 function; schedule a later poll on the main thread. Polling without waiting for the callback is valid

A start installs its non-null handle before it can issue a readiness notification; it returns `FRAMEWORK_STATUS_OK` for an accepted operation. Rejected starts leave the output handle null and take no callback/context ownership. Once accepted, the host keeps callback context live until the callback returns, a successful cancel, a poll that consumes a ready result, or destroy. The framework does not retain or free host memory

Cancel succeeds only while the operation is pending and before its readiness signal. It suppresses later callback/native-result delivery and drops the future on main; a later poll returns a ready result with `FRAMEWORK_STATUS_CANCELLED`. If readiness was already signaled or the result is terminal, cancel returns `FRAMEWORK_STATUS_NOT_FOUND`. Poll to consume a terminal result if it has not already been consumed; after the one ready result is consumed, further polls also return `FRAMEWORK_STATUS_NOT_FOUND`. Destroy can drop a pending operation without a later callback. Keep the original handle unique, do not alias or race its slot, and destroy it on main

The backend's Core Location future is lazy, so start polls it once to issue the native request. Each handle owns one pinned `async move` future that owns its `IosLocationBackend`; there is no self-referential borrow, cross-capability registry, executor, or task queue. B5's completion cell wakes on its operation-scoped Core Location delegate callback, which runs on the run loop where the manager was created. F15 creates that manager on main, so the readiness signal is main-thread-only. The host must poll again on main to move the result from the future into the C record

## Main thread, permission, and API floor

Every F15 call and handle operation must run on the iOS main thread, and the application's main run loop must continue to run. An otherwise-valid off-main call returns `FRAMEWORK_STATUS_UNAVAILABLE`; malformed pointers or input values may be rejected first. Operation destroy always returns unavailable off-main before reading or changing its pointer slot. A consuming app must provide `NSLocationWhenInUseUsageDescription` before it explicitly requests foreground authorization. The authorization request completes only after Core Location reports a raw status different from the status sampled before the request; it does not return an already-known status immediately. If no status change is reported, the operation may remain pending. Use `framework_ios_location_authorization_query_start` to read the current status without waiting for a change. Cancel/destroy abandons the result but cannot promise to dismiss a permission prompt already shown

Only `framework_ios_location_authorization_request_start` asks for permission. A current-location request does not prompt through F15; call the explicit authorization request when appropriate, then query the status or start current-location work. F15 never requests Always authorization, temporary full accuracy, background location, or continuous updates. If access is denied, the current operation returns the backend's permission error

The Core Location API floor is iOS 9.0 because `CLLocationManager.requestLocation()` is available from iOS 9.0. `requestWhenInUseAuthorization()` itself has an iOS 8.0 floor. The installed Rust arm64 iOS target builds objects with minos 10.0, so the focused device link gate uses minos 10.0; the arm64 Simulator gate uses minos 14.0. These are link/build checks only and do not establish full-library runtime support on iOS 9.0

## Non-iOS behavior and limits

Non-iOS builds validate required pointers and current-accuracy input, initialize outputs, and return `FRAMEWORK_STATUS_UNSUPPORTED`; they create no mock operation and import no Apple framework. A null handle slot passed to destroy is a no-op; a non-null unsupported pointer value is not dereferenced or dropped, and its slot remains unchanged

F15 adds no timeout, freshness limit, time-to-fix guarantee, requested-accuracy guarantee, physical location claim, permission-prompt dismissal guarantee, background mode, universal executor, registry, or runtime claim. Its exact build, lint, C/C++ link, symbol, framework-import, deployment-minimum, and host-feature-isolation checks are in `bindings/c/check-ios-location.sh`; the linked probes are never executed

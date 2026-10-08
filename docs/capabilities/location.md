# Location

**framework-location** defines a portable **no_std** contract for one-shot current-location requests. It contains framework-owned values only and does not itself probe a device, request permission, or create a global service. The B5 [iOS location guide](../ios/location.md) documents the separate Core Location backend.

## Rust API

~~~rust
use framework_location::{
    AccuracyTarget, Coordinate, Location, LocationBackend, LocationRequest,
};

async fn read_one_fix<B: LocationBackend>(
    location: &mut Location<B>,
) -> Result<framework_location::LocationFix, framework_location::LocationError> {
    let request = LocationRequest::new(AccuracyTarget::new(50.0)?);
    location.current(request).await
}
~~~

**Location<B>** owns the caller-supplied backend value. The concrete **LocationBackend** type is selected statically; associated future types use **core::future::Future**, with no boxed trait object, executor, **Send** requirement, registry, or hidden initialization. Availability is a non-prompting query. Authorization query is non-prompting; only an explicit **request_authorization** call may ask a native backend to prompt. That request is for foreground one-shot use and does not silently request background authorization.

**LocationAuthorization** is framework-owned and distinguishes unknown, not-determined, denied, restricted, foreground, and background grant status. A background grant also allows foreground use, but this value is status only: the portable contract exposes no background-location operation. It does not encode every permission detail offered by every platform.

## Coordinate, accuracy, timestamp, and ownership

**Coordinate** stores WGS-84 latitude and longitude in decimal degrees. Construction rejects NaN, positive or negative infinity, latitude outside -90 through 90 degrees, or longitude outside -180 through 180 degrees. Both endpoints are inclusive. The type does not normalize longitude or transform another geodetic reference system.

**AccuracyTarget** is a finite, non-negative desired horizontal accuracy in meters. Zero asks for the best available precision. It is a caller preference, not a guarantee: a backend may return a fix whose reported accuracy is worse than the target. **LocationFix::horizontal_accuracy_meters** is the backend-reported horizontal uncertainty estimate in meters, validated as finite and non-negative. It is not a confidence interval or guarantee of actual error; native accuracy estimates can differ in method and quality.

**LocationTimestamp** is an unsigned count of milliseconds since the Unix epoch, 1970-01-01T00:00:00Z. It is wall-clock measurement time, not monotonic time and not the time the backend returned the fix. A backend may return a cached fix; the timestamp reports when that measurement was made, but the contract sets no maximum age or freshness threshold. Callers that need a freshness limit must compare the timestamp under their own wall-clock policy.

Coordinates, requests, timestamps, fixes, authorization values, and errors are Rust-owned scalar values. A fix contains no strings, heap buffer, reference, or native object and is returned by value. The portable facade makes no allocation for a fix; a native adapter may need to copy or convert native result fields, and this contract makes no ABI layout or zero-copy promise.

## Operation and cancellation semantics

A current-location request starts on first poll. Dropping it before its first poll starts no backend work. Dropping a pending request asks the backend to cancel its native one-shot operation when its native API supports cancellation. A native completion racing with cancellation is ignored after the Rust future is dropped. If the native operation cannot be cancelled, the backend must detach its callback safely, discard its eventual result, and release callback state exactly once. Each started operation has one terminal success or error; if the future remains live through that terminal state, it yields that result once. If the caller drops the future first, the result is suppressed. Duplicate or late native completions are ignored.

Dropping an authorization-request future abandons interest in its result but cannot be assumed to dismiss a permission prompt already shown. A backend must keep any callback state safe until completion. Each completed authorization operation yields one result if its future remains live, and no result if that future has been dropped. A non-prompting authorization query must not change permission state. Errors preserve the framework **ErrorKind** and optional signed native code through **LocationError::Backend**.

**Location::backend** and **backend_mut** provide a narrow route to backend-specific controls or native escape hatches. Platform types do not enter the portable API; a concrete native backend may expose them only through its own platform-specific API.

## Scope and support

This portable contract covers one fix per request. It does not implement continuous updates, geofencing, visit monitoring, heading, speed, background location, or location history. The separate iOS backend implements only one-shot current location; neither layer claims prompt presentation, device sensor availability, indoor performance, fix freshness, time-to-fix, or that a requested accuracy target will be met.

Portable checks for this crate:

~~~sh
cargo fmt --all -- --check
cargo test -p framework-location
cargo check -p framework-location --no-default-features
git diff --check
~~~

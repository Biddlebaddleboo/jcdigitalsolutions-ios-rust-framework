# Optional iOS ClassKit deep-link marker C API

The opt-in `framework-c-api` Cargo feature `ios-classkit-deep-link` enables
`bindings/c/include/framework_ios_classkit_deep_link.h`. Its one function reads only the
`NSUserActivity.isClassKitDeepLink` marker through D81's `ios-system-services` API and writes the
result as `0` or `1` in a `uint8_t` output.

## Borrowed activity pointer

Pass a live `NSUserActivity` pointer whose owner retains the object for the full call. F14 borrows
it synchronously and does not retain, store, release, or consume it. Do not pass a stale pointer or
call outside the host activity lifecycle and thread rules. The wrapper does not hop queues or add a
main-thread requirement.

The output pointer is required, must not overlap the activity object, and is initialized to `0`
before input and runtime checks. A null activity/output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`.
On iOS before the iOS 11.3 API floor,
the call returns `FRAMEWORK_STATUS_UNAVAILABLE`; on non-iOS a non-null opaque pointer returns
`FRAMEWORK_STATUS_UNSUPPORTED` without dereference. On success the output is exactly `0` or `1`.
A caught Rust panic returns `FRAMEWORK_STATUS_PANIC` with the output still zero.

The marker only identifies the ClassKit deep-link activity kind. It does not read a context path,
ClassKit context, assignment data, or student identity, and does not authorize ClassKit data access.
F14 makes no assignment-data, service-readiness, or full-ClassKit claim.

The gate compiles and links C11/C++17 host/device/Simulator consumers, inspects the selector,
framework imports, exported symbol, and deployment metadata, and never executes a consumer. See
[`PLAN_BINDINGS_CLASSKIT.md`](../../PLAN_BINDINGS_CLASSKIT.md),
[`D81 ClassKit marker`](../ios/classkit-deep-link.md), and
[`ClassKit capability scope`](../../PLAN_CAPABILITIES_CLASSKIT.md).

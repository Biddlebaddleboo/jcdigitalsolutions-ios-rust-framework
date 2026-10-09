# Optional iOS preferences C API

The opt-in `framework-c-api` Cargo feature `ios-preferences` enables
`bindings/c/include/framework_ios_preferences.h`. The C API is an adapter over D1
`framework-preferences` and B1 `ios-preferences`; it uses the app's standard
`NSUserDefaults` domain on iOS and does not add a second storage implementation.

## Build and lifetime

Enable `ios-preferences` when building the `framework-c-api` static library. Include
`framework_ios_preferences.h` and link Foundation on iOS. A non-iOS build includes a
validating stub with no `ios-preferences`, Objective-C, or Apple framework dependency.
The consuming application owns the app privacy manifest and must declare the required
`NSUserDefaults` usage reason in `PrivacyInfo.xcprivacy`.

Create one opaque `FrameworkIosPreferences` handle for the standard defaults domain.
Destroy only the original handle slot once; destroy clears the slot before releasing
the handle. A handle must not be copied or used concurrently with another call or
destroy. Calls are synchronous and may use different threads if serialized per handle;
there is no main-thread requirement. `framework_ios_preferences_availability` reports
the backend's availability and does not predict whether a later operation will succeed.

## Keys, values, and operations

Keys are exact, case-sensitive UTF-8 byte strings that are non-empty and contain no
NUL byte. No normalization or key prefix is added. Values are opaque bytes represented
by Foundation `NSData`; an empty value is distinct from an absent key. Input spans are
borrowed for the call only and are not retained. Zero-length byte spans use `{NULL, 0}`.

`framework_ios_preferences_get` sets `out_found` to zero and `out_value` to the default
empty descriptor before validation. Absence returns `out_found == 0`. A present value,
including an empty value, returns `out_found == 1` and transfers an owned buffer that
must be released exactly once with `framework_owned_buffer_destroy`. The output buffer
must not contain a live allocation on entry.

`framework_ios_preferences_set` copies the borrowed bytes before return. The current
B1 backend reports `FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_NOT_GUARANTEED` for an
accepted `FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC` update. A
`FRAMEWORK_IOS_PREFERENCES_REQUIRE_ATOMIC` request returns
`FRAMEWORK_STATUS_UNSUPPORTED` before native mutation and leaves the output tag at
`FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN`. A later read through the same
backend sees a successful write unless another writer changes that key; no crash
durability or multi-key transaction is promised.

`framework_ios_preferences_remove` reports whether a value was visible in the
`NSUserDefaults` search list before the call. It removes from the app domain, so a
registered or global-domain fallback may appear on a later read.

Null required outputs, inconsistent pointer/length span shapes, oversized spans, invalid
UTF-8, empty or NUL-containing keys, and unknown atomicity tags map to
`FRAMEWORK_STATUS_INVALID_ARGUMENT`. Every non-null pointer must still satisfy the
header's readability, writability, alignment, lifetime, and non-aliasing preconditions;
the implementation cannot validate arbitrary memory addresses or lifetimes.
Backend errors map from `PreferenceError::kind()` through
`FrameworkStatus::from_error`. The current B1 path supplies no platform-native error
code, so this ABI exposes no native-code output. Status-returning exports contain Rust
panics as `FRAMEWORK_STATUS_PANIC`; destroy has no status return. On non-iOS targets,
valid calls return `FRAMEWORK_STATUS_UNSUPPORTED` after required output initialization
and shape validation.

This is non-secure settings storage, not Keychain, cross-device synchronization, or an
immediate durable-flush API. It does not promise that a read from another process sees
the value or that the defaults store has flushed it to persistent storage. See
[`docs/ios/preferences.md`](../ios/preferences.md) for the B1 backend boundary and
platform build evidence.

## Validation limits

The focused C ABI gate checks feature isolation, manifest/header symbols and tags,
C11/C++17 compile/link consumers, arm64 device and Simulator builds, linked imports,
and target deployment metadata. It does not execute any probe or claim live defaults
behavior. The consumer link floors are iOS 10.0 for device and iOS 14.0 for Simulator;
these are validation target floors, not a newly established minimum for
`NSUserDefaults` itself. Evidence is recorded in `PLAN_BINDINGS_PREFERENCES.md`.

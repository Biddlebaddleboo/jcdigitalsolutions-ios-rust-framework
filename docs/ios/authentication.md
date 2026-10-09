# iOS Local Authentication

`ios-auth` implements the portable one-shot local presence contract with Apple's public `LocalAuthentication` API. It has no global context, executor, registry, or process setup

## Use

Choose a policy for each explicit request

```rust
use framework_auth::{AuthenticationPolicy, AuthenticationRequest, Authenticator};
use ios_auth::IosAuthenticationBackend;

async fn verify(
    auth: &mut Authenticator<IosAuthenticationBackend>,
) -> Result<(), framework_auth::AuthenticationError> {
    let request = AuthenticationRequest::new(
        AuthenticationPolicy::BiometricsOnly,
        "Confirm this action",
    )?;
    auth.authenticate(request).await
}
```

`BiometricsOnly` maps to `LAPolicy::DeviceOwnerAuthenticationWithBiometrics` and does not permit a device-passcode fallback. `DeviceOwner` maps to `LAPolicy::DeviceOwnerAuthentication` on iOS 9+; Apple may use biometrics or the local device credential for that policy. Before iOS 9, `DeviceOwner` reports `Availability::Unsupported` and an explicit request returns `ErrorKind::Unsupported`

Call `availability(policy)` when a current query without a prompt is useful. Each call creates a fresh `LAContext` and invokes `canEvaluatePolicy_error`; the backend caches no result. Enrollment, lockout, passcode, and OS state can change after any query. The backend never calls `canEvaluatePolicy` from an `evaluatePolicy` reply block

Before first poll, the future holds the borrowed request. On first poll, it consumes that request, copies the reason into an owned `NSString`, and drops the request before native evaluation. It creates a fresh context and starts evaluation on that poll. The future holds its context, owned reason, and reply block until a result or drop

## Callback and cancellation

Apple documents the `evaluatePolicy` reply as a private queue with no fixed thread. The callback uses only an `Arc`-backed mutex cell and an `Arc<AtomicBool>`; it does not touch the future, `Rc`, `RefCell`, or `LAContext`. It catches Rust panics at the callback boundary. The cell takes one result, drops its lock, then wakes the task

The atomic flag rejects a duplicate native callback. The completion cell also accepts only one result. If a pending future is dropped, it detaches its result and waker, then calls `invalidate` on iOS 9+. On iOS 8, `invalidate` is unavailable; the backend releases its retained context, but makes no native-cancellation guarantee for that OS version. A callback that races with drop can finish its native work but cannot publish a Rust result or wake the detached task. The native callback state has no context reference, so it cannot form a context/block retain cycle

## Availability and error table

| Native result | Portable result |
| --- | --- |
| `canEvaluatePolicy` succeeds | `Availability::Available` |
| `kLAErrorPasscodeNotSet`, `kLAErrorBiometryNotAvailable`, `kLAErrorBiometryNotEnrolled`, or `kLAErrorBiometryLockout` | `Availability::TemporarilyUnavailable` |
| Unsupported policy or OS floor | `Availability::Unsupported` |
| Any other availability error or domain | `Availability::Unknown` |
| `kLAErrorUserCancel`, `kLAErrorUserFallback`, `kLAErrorSystemCancel`, or `kLAErrorAppCancel` | `ErrorKind::Cancelled` with its nonzero `i32` code when representable |
| `kLAErrorInvalidContext` | `ErrorKind::Internal` with its nonzero `i32` code when representable |
| Any other error in `kLAErrorDomain` | `ErrorKind::Platform` with its nonzero `i32` code when representable |
| Error from a different domain | `ErrorKind::Platform` without a native code |
| Failed callback with no `NSError` | `ErrorKind::Internal` |
| Empty or whitespace-only reason | `AuthenticationError::InvalidReason` / `ErrorKind::InvalidInput` |
| Unknown policy variant | `ErrorKind::Unsupported` |

`kLAErrorUserFallback` maps to cancellation: `BiometricsOnly` never changes to device-credential authentication. A successful result means only that the chosen system policy reported success at that time; it does not establish a person's identity or create a reusable credential

## Host app and data limits

An app that invokes Face ID must set `NSFaceIDUsageDescription` in its host app `Info.plist`. This crate does not edit that plist, request an entitlement, or prompt without an explicit request. It exposes no biometric modality, sample, enrollment record, or template, and has no `Security` or Keychain dependency

## API floor and dependency

The active Xcode 26.6 / iOS 26.5 SDK headers mark `LAContext`, `canEvaluatePolicy:error:`, `evaluatePolicy:localizedReason:reply:`, and `LAPolicyDeviceOwnerAuthenticationWithBiometrics` as iOS 8.0 APIs. `LAPolicyDeviceOwnerAuthentication` and `LAContext.invalidate` have an iOS 9.0 floor and runtime gates in this crate. `LAErrorDomain` has an iOS 8.3 symbol floor, so the backend checks the public `kLAErrorDomain` compile-time string constant instead; this preserves the iOS 8.0 biometrics floor

The iOS target selects `objc2-local-authentication` 0.3.2 with defaults off and only `LAContext`, `LAPublicDefines`, and `block2`. `block2` supplies the native reply block; `LAPublicDefines` supplies policy error constants and the domain string. The backend uses no Security APIs, private APIs, or Swift source

## Link/import probe

`sh platform/ios/ios-auth/check-link-imports.sh` builds a device and simulator probe binary, then checks each Mach-O load list with `otool -L` against the exact allowlist `LocalAuthentication.framework`, `Foundation.framework`, `libSystem.B.dylib`, and `libobjc.A.dylib`. It also scans undefined symbols with `nm -u` and rejects Swift runtime or Keychain symbols. The script does not execute either binary or show a prompt

Target checks validate Rust compilation and imports only. They do not exercise a live prompt, biometric sensor, enrollment state, or host-app plist

## Apple references

- [`LAContext`](https://developer.apple.com/documentation/localauthentication/lacontext)
- [`canEvaluatePolicy:error:`](https://developer.apple.com/documentation/localauthentication/lacontext/canevaluatepolicy%28_%3Aerror%3A%29)
- [`evaluatePolicy:localizedReason:reply:`](https://developer.apple.com/documentation/localauthentication/lacontext/evaluatepolicy%28_%3Alocalizedreason%3Areply%3A%29)
- [`invalidate`](https://developer.apple.com/documentation/localauthentication/lacontext/invalidate%28%29)
- [`LAError` codes](https://developer.apple.com/documentation/localauthentication/laerror-swift.struct/code)
- [`LAErrorDomain`](https://developer.apple.com/documentation/localauthentication/laerrordomain)
- [`NSFaceIDUsageDescription`](https://developer.apple.com/documentation/bundleresources/information_property_list/nsfaceidusagedescription)

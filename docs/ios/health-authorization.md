# iOS HealthKit availability and authorization

`ios-health-authorization` implements `framework-health-authorization` with public HealthKit APIs. It contains no Swift source and does not read or write health samples.

## Use

```rust
use framework_health_authorization::{
    HealthAuthorizationBackend, HealthAuthorizationRequest, HealthDataType, HealthDataTypeKind,
};
use ios_health_authorization::IosHealthAuthorizationBackend;

let backend = IosHealthAuthorizationBackend::new();
if backend.is_available() {
    let step_count = HealthDataType::new(
        HealthDataTypeKind::Quantity,
        "HKQuantityTypeIdentifierStepCount",
    )?;
    let read_types = [step_count];
    let request = HealthAuthorizationRequest::new(&read_types, &[])?;
    backend.request_authorization(&request, |completion| {
        // Ok(()) reports request-flow completion only, not permission.
        let _ = completion;
    })?;
}
```

The backend checks `HKHealthStore.isHealthDataAvailable` before any other HealthKit call and checks it again before each authorization request. If unavailable, the request returns `ErrorKind::Unavailable` and does not call its completion. A false availability result can mean unsupported OS/device conditions or restricted access; the API does not distinguish them. Store creation is lazy and occurs only after availability is true.

The backend resolves each type identifier with the matching `HKObjectType` factory, creates read and share `NSSet` values, and calls `requestAuthorizationToShareTypes:readTypes:completion:`. Unknown or OS-unavailable identifiers fail synchronously as `InvalidTypeIdentifier`. The iOS completion runs on HealthKit's arbitrary background queue. The Rust callback requires `Send`, is invoked at most once, and is protected from unwinding across the Objective-C block boundary. If the native callback reports an `NSError`, its nonzero code is preserved when representable as `i32`; a false callback `Bool` without an error maps to an unclassified backend error.

Neither the callback `Bool` nor its success mapping means read access was granted. Apple states that read denials are hidden from the app. `authorizationStatus(for:)` is not called: it reports sharing/write status only and cannot reveal read authorization. The package exposes no authorization status, read-result inference, grant query, query, or sample-write operation.

## Capability, entitlement, and Info.plist

The consuming app must enable the **HealthKit** capability in Xcode and ship with the `com.apple.developer.healthkit` Boolean entitlement. HealthKit availability does not prove that this entitlement is present or valid for the app's provisioning profile. Xcode adds `healthkit` to `UIRequiredDeviceCapabilities`; an app that can operate without HealthKit may remove that entry so unsupported devices remain installable.

Set purpose strings for the requested access roles:

- Read types require a meaningful string in `NSHealthShareUsageDescription`.
- Share/write types require a meaningful string in `NSHealthUpdateUsageDescription`.
- This package does not request clinical records, so it does not require `NSHealthClinicalHealthRecordsShareUsageDescription`, `NSHealthRequiredReadAuthorizationTypeIdentifiers`, or the `com.apple.developer.healthkit.access` clinical-record entitlement.
- This package does not use observer queries or background delivery, so it does not require the `com.apple.developer.healthkit.background-delivery` entitlement.

If an app requests both read and share types, provide both read and update purpose strings. Apple warns that the relevant usage-description keys must be present when requesting HealthKit authorization.

## API floor and runtime limits

The selected `HKHealthStore`, `isHealthDataAvailable`, read/share authorization request, base `HKObjectType` factories, and workout type are available from iOS 8.0. The package API floor is iOS 8.0. A specific HealthKit identifier may have a higher introduction version; the caller must select an identifier supported by the deployment target and still handle factory rejection.

HealthKit data is available on supported iOS devices, iPadOS 17 or later, and iOS apps running on Apple Vision Pro. Apple documents that iPads on iPadOS 16 and earlier and macOS 13 or later return false from `isHealthDataAvailable`; enterprise policy can also restrict HealthKit. Availability can change with device, OS, or policy state. The iOS Simulator has HealthKit sample-account support for clinical-record testing, but that does not establish live sensor availability or authorization behavior for every type. This workstream runs target compilation only; it does not launch an app or exercise simulator/device permission UI.

The request may complete without presenting a prompt when the user already made the relevant choices. No callback timing, successful access, sample availability, query result, background execution, or persistence guarantee is made. If the process exits before HealthKit calls the completion block, the Rust callback cannot be delivered.

## Binding surface

The iOS SDK 26.5 `HKHealthStore.h` declares the class at iOS 8.0 and contains `isHealthDataAvailable` and `requestAuthorizationToShareTypes:readTypes:completion:`. `HKObjectType.h` contains the quantity, category, characteristic, correlation, and workout factories used here. The matching generated Rust surface is `objc2-health-kit` 0.3.2 with `HKHealthStore`, `HKObjectType`, `HKTypeIdentifiers`, and `block2` features; `objc2-foundation` enables only `alloc`, `NSError`, `NSObject`, `NSSet`, and `NSString`. The portable crate exposes no Objective-C or third-party binding type. No Swift source, Swift runtime bridge, entitlement mutation, or app metadata is supplied by this package.

Target checks used Rust 1.94.1, Xcode 26.6 build 17F113, and the iOS 26.5 SDK. The iPhoneOS SDK headers are at `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/HealthKit.framework/Headers/`.

## Apple API references

- [Setting up HealthKit](https://developer.apple.com/documentation/healthkit/setting-up-healthkit)
- [Authorizing access to health data](https://developer.apple.com/documentation/healthkit/authorizing-access-to-health-data)
- [HKHealthStore.isHealthDataAvailable](https://developer.apple.com/documentation/healthkit/hkhealthstore/ishealthdataavailable%28%29)
- [HKHealthStore.authorizationStatus(for:)](https://developer.apple.com/documentation/healthkit/hkhealthstore/authorizationstatus%28for%3A%29)
- [HKAuthorizationStatus](https://developer.apple.com/documentation/healthkit/hkauthorizationstatus)
- [HKHealthStore.requestAuthorization(toShare:read:)](https://developer.apple.com/documentation/healthkit/hkhealthstore/requestauthorization%28toshare%3Aread%3Acompletion%3A%29)
- [HealthKit entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.healthkit)
- [Accessing sample data in the Simulator](https://developer.apple.com/documentation/healthkit/accessing-sample-data-in-the-simulator)

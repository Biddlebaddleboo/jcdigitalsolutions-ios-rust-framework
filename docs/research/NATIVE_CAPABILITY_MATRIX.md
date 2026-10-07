# Native Capability Elimination Matrix — Pass 1

Research date: 2026-10-07

This pass starts with common capabilities and tries to eliminate Swift ABI work before studying Swift-only frameworks.

## Executive finding

A large fraction of ordinary iOS application capabilities already have public Objective-C or C surfaces and, in many cases, existing objc2 generated bindings. For these capabilities, Swift ABI interoperability would be unnecessary overhead and complexity.

The highest-value architectural conclusion from this pass is:

> Do not treat "commonly written in Swift" as "Swift-only." Prefer the underlying public native interface and recreate only the ergonomic overlay in Rust.

## Matrix

| Capability | Public native route | objc2 status observed | Class | Research conclusion |
|---|---|---|---|---|
| HTTP/network transfers | `NSURLSession`, delegates, completion handlers | Foundation binding expected; Apple ObjC API confirmed | R2/R3 | Do not implement Swift async solely for normal HTTP. Build cheap Rust callback/Future overlay if useful. |
| Preferences | `NSUserDefaults` | Foundation | R2/R3 | Direct ObjC path. Rust convenience can be extremely thin. |
| Local/push notification client API | `UNUserNotificationCenter` and delegate/protocol types | `objc2-user-notifications` exists | R2/R3 | No Swift ABI required for ordinary notifications. |
| Location | `CLLocationManager`, `CLLocationManagerDelegate` | `objc2-core-location` exists | R2/R3 | No Swift ABI required. |
| Camera/audio capture | `AVCaptureSession` and AVFoundation ObjC classes | objc2 AVFoundation bindings exist in ecosystem | R2/R3 | Native ObjC path. |
| Photos library | `PHPhotoLibrary`, `PHAsset`, etc. | `objc2-photos` exists | R2/R3 | No Swift ABI required. |
| Contacts | `CNContactStore` and related ObjC classes | `objc2-contacts` exists | R2/R3 | No Swift ABI required. |
| Biometrics / device authentication | `LAContext` | `objc2-local-authentication` exists | R2/R3 | No Swift ABI required. |
| Sign in with Apple / passkeys core controller path | `ASAuthorizationController` + delegates | `objc2-authentication-services` exists | R2/R3 | Prefer ObjC controller/delegate API over newer Swift async conveniences. |
| Web views | `WKWebView`, navigation/UI delegates | `objc2-web-kit` exists | R2/R3 | No Swift ABI required. |
| CloudKit database access | `CKContainer`, `CKDatabase`, operations | `objc2-cloud-kit` exists | R2/R3 | Large useful CloudKit surface is Objective-C accessible. |
| Core ML inference | `MLModel`, `MLFeatureProvider` | `objc2-core-ml` exists | R2/R3 | Avoid Xcode-generated Swift wrappers; call generic MLModel API directly from Rust. |
| Vision | `VNRequest`, request handlers, completion handlers | `objc2-vision` exists | R2/R3 | Large Vision surface is ObjC accessible. |
| Metal | ObjC `MTLDevice` protocol plus C factory such as `MTLCreateSystemDefaultDevice` | objc2 Metal bindings exist in ecosystem | R1/R2 | Very strong native-cost fit for Rust. |
| Maps | `MKMapView` and MapKit ObjC classes | objc2 MapKit bindings expected in ecosystem | R2/R3 | SwiftUI Map is not required for native map capability. |
| Core Data persistence | `NSManagedObjectContext` and mature ObjC API | objc2 Core Data bindings expected | R2/R3 or D | Can use native Core Data, but pure Rust persistence may be simpler for many apps. SwiftData not required for persistence capability. |
| Low-level networking | Network framework exposes C reference surface as well as Swift classes | C APIs are public | R1 | Prefer direct C ABI where it gives the required capability. |
| Reachability preflight | Old `SCNetworkReachability` C API exists but is deprecated | N/A | D | Do not build around deprecated reachability. Attempt connections or use current Network APIs. |
| General app state/algorithms/parsing | Rust | N/A | R0 | Never route through Apple object model without need. |
| Reactive streams | Rust channels/callbacks/Futures; Apple capability sources usually have callbacks/delegates | N/A | D | Combine is a Swift abstraction, not a prerequisite for most underlying capabilities. |
| Observation/state notification | Rust state/event mechanisms | N/A | D | Observation macro is Swift compile-time machinery; no reason to adopt it for Rust-owned state. |

## Evidence notes

### NSURLSession

Apple exposes `NSURLSession` as an Objective-C `NSObject` class. Delegates including `NSURLSessionDelegate` and `NSURLSessionTaskDelegate` support authentication, redirects, task completion, and background transfer behavior.

**Implication:** Swift `async` URLSession methods are ergonomic overlays, not required capability access. The framework should begin with native delegate/completion forms and add a Rust `Future` adapter only if it can remain cheap and cancellation-correct.

Apple reference:
https://developer.apple.com/documentation/foundation/urlsession?language=objc

### NSUserDefaults

Apple exposes `NSUserDefaults` directly to Objective-C.

**Implication:** preferences are an R2 path. A Rust helper can mostly be inline convenience and type conversion.

Apple reference:
https://developer.apple.com/documentation/Foundation/UserDefaults?language=objc

### User Notifications

Apple exposes `UNUserNotificationCenter` as an Objective-C `NSObject` class. The objc2 ecosystem contains `objc2-user-notifications`.

**Implication:** authorization, scheduling, categories/actions, settings, and delivery delegate work do not intrinsically require Swift ABI.

Apple reference:
https://developer.apple.com/documentation/usernotifications/unusernotificationcenter?language=objc

objc2 reference:
https://docs.rs/objc2-user-notifications/latest/objc2_user_notifications/

### Core Location

`CLLocationManager` is the long-standing manager/delegate interface. The objc2 ecosystem contains `objc2-core-location`.

**Implication:** use a Rust-defined Objective-C delegate and preserve the native callback model.

Apple reference:
https://developer.apple.com/documentation/corelocation/cllocationmanager

objc2 reference:
https://docs.rs/objc2-core-location/latest/objc2_core_location/

### AVFoundation capture

Apple documents `AVCaptureSession` as an Objective-C `NSObject` class.

**Implication:** camera and audio capture do not require Swift merely because current sample code is often Swift.

Apple reference:
https://developer.apple.com/documentation/avfoundation/avcapturesession?language=objc

### Photos

Apple exposes `PHPhotoLibrary` as `@interface PHPhotoLibrary : NSObject`. `objc2-photos` exposes PhotoKit/Photos bindings.

Apple reference:
https://developer.apple.com/documentation/photos/phphotolibrary?language=objc

objc2 reference:
https://docs.rs/objc2-photos/latest/objc2_photos/

### Contacts

Apple exposes `CNContactStore` as the central contacts database object. `objc2-contacts` includes `CNContactStore` and related request/value classes.

Apple reference:
https://developer.apple.com/documentation/contacts/cncontactstore/

objc2 reference:
https://docs.rs/objc2-contacts/latest/objc2_contacts/

### Local Authentication

Apple exposes `LAContext` as an Objective-C `NSObject` with asynchronous reply blocks. `objc2-local-authentication` contains corresponding bindings.

**Implication:** Face ID / Touch ID policy evaluation is a straightforward Objective-C + Block boundary.

Apple reference:
https://developer.apple.com/documentation/localauthentication/lacontext?language=objc

objc2 reference:
https://docs.rs/objc2-local-authentication/latest/objc2_local_authentication/

### Authentication Services

Apple exposes `ASAuthorizationController` as an Objective-C class with delegate and presentation-context-provider protocols. objc2 already binds the controller, including `performRequests`, delegates, and request types.

Apple reference:
https://developer.apple.com/documentation/authenticationservices/asauthorizationcontroller?language=objc

objc2 reference:
https://docs.rs/objc2-authentication-services/latest/objc2_authentication_services/struct.ASAuthorizationController.html

A newer Swift `AuthorizationController` offers async methods, but it is not necessary to obtain the underlying authorization capability.

### WebKit

`WKWebView` is explicitly exposed to Objective-C and uses delegate objects. `objc2-web-kit` binds it.

Apple reference:
https://developer.apple.com/documentation/webkit/wkwebview?language=objc

objc2 reference:
https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKWebView.html

### CloudKit

Apple exposes `CKContainer` to Objective-C. `objc2-cloud-kit` provides broad generated CloudKit coverage.

Apple reference:
https://developer.apple.com/documentation/cloudkit/ckcontainer?language=objc

objc2 reference:
https://docs.rs/objc2-cloud-kit/latest/objc2_cloud_kit/

### Core ML

Apple exposes `MLModel` as `@interface MLModel : NSObject`, including generic feature-provider prediction APIs. `objc2-core-ml` binds `MLModel`.

**Implication:** the framework should not depend on Xcode-generated language-specific model wrappers. It can build a Rust-native typed layer above `MLModel` only where ergonomically useful.

Apple reference:
https://developer.apple.com/documentation/CoreML/MLModel?language=objc

objc2 reference:
https://docs.rs/objc2-core-ml/latest/objc2_core_ml/

### Vision

Apple exposes `VNRequest` as an Objective-C class with completion-handler APIs. `objc2-vision` has broad request/observation coverage including text recognition, barcodes, Core ML requests, tracking, and request handlers.

Apple reference:
https://developer.apple.com/documentation/vision/vnrequest?language=objc

objc2 reference:
https://docs.rs/objc2-vision/latest/objc2_vision/

### Metal

Apple exposes `MTLDevice` as an Objective-C protocol and also exposes C entry functions such as `MTLCreateSystemDefaultDevice`.

**Implication:** GPU access is naturally compatible with a Rust-first, near-zero-overhead design. Keep high-volume application/GPU data in Rust/native buffers and cross Objective-C only where Metal's object API requires it.

Apple reference:
https://developer.apple.com/documentation/metal/mtldevice?language=objc

### Network framework

Apple's Network documentation exposes a C reference surface in addition to higher-level Swift types.

**Implication:** where `URLSession` is not appropriate, investigate the C Network API before attempting Swift `NWConnection` ABI interop.

Apple reference:
https://developer.apple.com/documentation/network

### Core Data versus SwiftData

Core Data retains a mature Objective-C object API centered on classes such as `NSManagedObjectContext`.

SwiftData exposes newer Swift-native model/container APIs, but SwiftData is not required to obtain persistence as a capability.

**Implication:** SwiftData should be researched later as an interoperability feature, not treated as foundational persistence infrastructure for this framework.

Apple Core Data reference:
https://developer.apple.com/documentation/CoreData/NSManagedObjectContext?language=objc

## Swift abstractions deliberately deprioritized by this pass

### Combine

Apple describes Combine as a declarative **Swift API** based on generic `Publisher<Output, Failure>` and `Subscriber` protocols.

Many underlying event sources—URLSession, NotificationCenter, timers, delegates—are accessible without Combine.

Classification: **D for framework core**. A Rust-native reactive adapter could be added later if applications actually need it.

Apple reference:
https://developer.apple.com/documentation/combine

### Observation

Apple's `Observable` documentation states that merely conforming to the protocol does not implement observation; the `@Observable` macro generates the functionality and conformance at compile time.

Classification: **D for Rust-owned state**. Rust should use Rust state/event primitives rather than reproducing Swift Observation semantics solely for internal state.

Apple reference:
https://developer.apple.com/documentation/observation/observable

## Genuine Swift residual already identified

### StoreKit 2

Modern `Product` and `Transaction` are Swift structures. Transaction streams use `AsyncSequence`. The old Objective-C `SKPaymentQueue` API is now marked deprecated/no longer supported.

This is qualitatively different from URLSession or Photos: the modern supported purchase model creates a real reason to study Swift ABI.

Initial classification: **S2, high priority**.

Apple references:
https://developer.apple.com/documentation/storekit/product
https://developer.apple.com/documentation/storekit/transaction
https://developer.apple.com/documentation/storekit/skpaymentqueue?language=objc

### App Intents

`AppIntent` is a Swift protocol. Parameters are declared with macros such as `@Parameter`, and the system depends on app-intent type declarations/discovery.

Initial classification: **S3, high priority** because compile-time/generated metadata may be as important as runtime ABI.

Apple reference:
https://developer.apple.com/documentation/appintents/appintent

### WidgetKit

`Timeline<EntryType>` is generic, and providers conform to Swift protocols such as `TimelineProvider` / `AppIntentTimelineProvider`.

Initial classification: **S2/S3**, but UI-heavy WidgetKit/SwiftUI rendering is lower priority for this framework than non-UI system capabilities.

Apple reference:
https://developer.apple.com/documentation/widgetkit/timeline

## Next research passes

1. Expand the R0/R1/R2 matrix across filesystem, Security/Keychain, CryptoKit alternatives, Bluetooth, motion/sensors, background tasks, audio/video playback, speech, sharing/pasteboard, document APIs, printing, NFC, HomeKit, HealthKit, CallKit, PushKit, NetworkExtension, DeviceCheck/App Attest, GameKit, EventKit, MessageUI, NearbyInteraction, MultipeerConnectivity, ExternalAccessory, CoreBluetooth, CoreMotion, CoreNFC, PDFKit and related high/medium-frequency capabilities.
2. Deep-dive StoreKit 2 as the first strong Swift-ABI candidate.
3. Deep-dive App Intents as the first Swift compiler/metadata candidate.
4. Catalogue newer Swift-only frameworks and rank them by actual capability gap, not popularity of Swift syntax.

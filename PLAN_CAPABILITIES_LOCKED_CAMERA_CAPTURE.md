# PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md — D95: LockedCameraCapture row 109 feasibility audit

## Scope

Report-only audit of row `109-extension-entitlement-capabilities-lockedcameracapture`. Inspect the installed iPhoneOS SDK, public LockedCameraCapture and related camera declarations, cached Rust bindings, and Apple primary documentation. No matrix, aggregate plan, index, Cargo, CI, or code edits; no builds or tests

## Recommendation

Keep row 109 at `X`. LockedCameraCapture is an iOS 18+ SwiftUI/ExtensionKit capture-extension lifecycle, not a general device capability bit. The public APIs provide extension session state and captured-content handoff; no documented API reports whether a compatible capture extension is installed, configured by the user, enabled, authorized, or ready to launch

The closest bounded app-side value is `LockedCameraCaptureManager.sessionContentURLs`, which lists directories for captured content already handed to the containing app. It reports pending content, not the availability or readiness of LockedCameraCapture, and presumes a configured extension and camera-capture flow. `sessionContentUpdates` is an asynchronous sequence of such content changes, not a support check

The public `NSUserActivityTypeLockedCameraCapture` value is a launch-source marker for `LockedCameraCaptureSession.openApplication(for:)`; it is not a capability query. The installed `.tbd` exports a Swift-mangled `LockedCameraCaptureSession.hasActiveSession` getter, but neither the public Swift interface nor Apple documentation declares it. An exported symbol alone is not a supported API contract, so do not bind or call it

## Installed SDK and binding evidence

Toolchain: Xcode 26.6, build `17F113`; iPhoneOS SDK 26.5

- Framework path: `iPhoneOS.sdk/System/Library/Frameworks/LockedCameraCapture.framework`
- `Headers/LockedCameraCapture.h` imports `LCCDefines.h`; the public Objective-C header declares only version globals and `NSUserActivityTypeLockedCameraCapture`, with no Objective-C class, protocol, or selector for the framework's capture/session APIs
- `LCCDefines.h` declares `NSUserActivityTypeLockedCameraCapture` as an `NSString` global and describes it as the activity type used with `openApplication(for:)`
- The public `LockedCameraCapture.swiftinterface` marks the API iOS 18.0+, unavailable on macOS, Mac Catalyst, tvOS, watchOS, and visionOS. It imports ExtensionKit, SwiftUI, and Swift concurrency
- `LockedCameraCaptureExtension` and `LockedCameraCaptureExtensionScene` are `@MainActor` protocols. `LockedCameraCaptureExtension` extends `AppExtension`; its associated `Body` is a `LockedCameraCaptureExtensionScene`. `LockedCameraCaptureUIScene<Content>` is also `@MainActor` and requires `Content: View`
- `LockedCameraCaptureSession` is a final `Sendable` Swift class, system-initialized for the extension scene. It exposes `sessionContentURL`, `openApplication(for:) async throws`, and `invalidateSessionContent() async throws`
- `LockedCameraCaptureManager` is a final `Sendable` Swift class with `shared`, synchronous `sessionContentURLs: [URL]`, `sessionContentUpdates: some AsyncSequence<SessionContentUpdate, Never>`, and `invalidateSessionContent(at:) async throws`. `beginDelayingAppearance()` and `endDelayingAppearance()` have an iOS 18.1 floor
- The `.tbd` export target is `arm64e-ios` and the framework API symbols are Swift-mangled. It also exports `LockedCameraCaptureSession.hasActiveSession.getter : Swift.Bool`; that symbol is absent from the public Swift interface and Apple API reference. The inspected SDK artifacts do not establish a Simulator import/link path
- No `objc2-locked-camera-capture` crate or LockedCameraCapture Rust binding was found in the local Cargo source cache or repository Rust/C/Objective-C source
- There is no direct C/Objective-C entry to the Swift session, manager, or scene APIs in the public headers. The synchronous manager property still returns a Swift `[URL]`; its update stream and session operations use Swift concurrency. The current Swift ABI plans record no supported public Swift async task-entry/context/resume contract; see `PLAN_SWIFT_ABI_ASYNC.md` and `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`

## Status-like APIs and limits

| Public API | API floor | Meaning | Why it does not establish LockedCameraCapture support |
| --- | --- | --- | --- |
| `NSUserActivityTypeLockedCameraCapture` | iOS 18.0 framework surface | Activity type for a launch from the capture extension | It is a constant used after an extension launch, not an extension availability or registration query |
| `LockedCameraCaptureManager.sessionContentURLs` | iOS 18.0 | URLs for directories with captured session content | Presence reflects content already available to the app; an empty list does not distinguish no capture, no configured extension, no permission, or no pending content |
| `LockedCameraCaptureManager.sessionContentUpdates` | iOS 18.0 | Async updates for initial, added, or removed content URLs | It monitors the containing app's capture-content handoff, not device or extension readiness |
| `AVCaptureDevice.authorizationStatus(for: .video)` | AVFoundation camera API, not LockedCameraCapture | The app's user authorization for camera capture | Camera permission is one prerequisite; it does not report capture-extension registration, user control configuration, or a runnable capture scene |
| `LockedCameraCaptureSession.hasActiveSession` exported symbol | No public floor established | The installed `.tbd` exports a Swift-mangled Bool getter | No declaration exists in the installed public Swift interface or Apple docs; no supported public call path is established |

`sessionContentURLs` is a plausible separate host-app snapshot if a future plan explicitly covers captured-content presence. It cannot move row 109 from `X` without changing the row's meaning from LockedCameraCapture support to one particular output of a previously invoked capture extension

## Extension, permission, and lifecycle limits

- Apple documents a Capture Extension target created from the Xcode extension template. The extension implements the `@MainActor` `LockedCameraCaptureExtension` protocol and provides a SwiftUI `LockedCameraCaptureUIScene`; the system creates its `LockedCameraCaptureSession`
- The host also needs a camera capture control path. Apple directs the app to include a `CameraCaptureIntent` in the app, control widget, and capture-extension targets; a user adds the control to Control Center or the Lock Screen, or configures the Action button. This configuration is user and host state, not a framework support value
- A capture extension requires an active camera view that uses `AVCaptureEventInteraction`; Apple says the system terminates it shortly after launch if there is no such view or camera access has not been requested
- The extension inherits the app's camera permission. If permission is not yet granted, Apple says the system asks the person to authenticate and unlock the device, opens the containing app, and requests camera access there. Camera access needs `NSCameraUsageDescription`; the camera authorization query and request are AVFoundation behavior, not LockedCameraCapture availability. The public pages and headers reviewed do not identify a LockedCameraCapture-specific entitlement or usage-description key
- While the capture extension is active, Apple says it cannot access the network or the App Group shared container. Its extension data container is erased when the system suspends the extension. `sessionContentURL` is temporary; the system copies its contents to the containing app's data container when the extension suspends, then erases the extension container
- `openApplication(for:)` requests a transition to the containing app; if the device is not authenticated, the system asks the person to authenticate. The manager's appearance-delay calls are transition coordination, not app-launch guarantees or an extension support check
- Photo library access is a separate choice. Saving through PhotoKit needs the relevant Photos usage description and user permission; use of the temporary session content URL alone does not imply PhotoKit access

The checked public LockedCameraCapture documentation does not name a dedicated entitlement. That absence is not evidence of distribution approval or app-review behavior. `NSCameraUsageDescription` and camera authorization remain required for camera access; the macOS Camera entitlement documented by AVFoundation does not apply as an iOS requirement

## Disposition and root integration

Row 109 remains `X`; no portable contract or iOS backend is proposed. The framework's public surface is an extension scene, capture session, and containing-app content handoff, all with host lifecycle and user-configuration dependencies. The only functional direct C/Objective-C declaration is a launch activity string; the other header globals are framework version metadata. No public Rust-callable LockedCameraCapture readiness query exists

No package, dependency, workspace, lock, CI, aggregate plan, docs index, matrix, or count change is required. Keep the canonical row's null metadata unverified unless root elects to add a scoped blocker note. If root updates `status_reason`, a precise summary is: `No public Rust-callable LockedCameraCapture readiness query exists; the iOS 18+ API is a SwiftUI/ExtensionKit capture extension that depends on camera authorization, user-added controls, and host/content handoff`

## Audit record

- Read the installed iPhoneOS SDK 26.5 LockedCameraCapture header, module map, Swift interface, `.tbd`, and row `109-extension-entitlement-capabilities-lockedcameracapture`
- Searched the local Cargo source cache and repository Rust, C, and Objective-C sources for LockedCameraCapture bindings or adapters; none were found
- Demangled the `.tbd` `hasActiveSession` symbol for identification only; it has no public Swift-interface or Apple-doc declaration and was not called
- Reviewed Apple LockedCameraCapture, extension, control widget, App Intents, AVFoundation camera authorization, and Info.plist documentation
- No code, manifest, lock, CI, matrix, build, link probe, test, entitlement, app, or runtime query changed or ran

## Apple primary sources

- [LockedCameraCapture overview](https://developer.apple.com/documentation/lockedcameracapture)
- [Creating a camera experience for the Lock Screen](https://developer.apple.com/documentation/lockedcameracapture/creating-a-camera-experience-for-the-lock-screen)
- [`LockedCameraCaptureExtension`](https://developer.apple.com/documentation/lockedcameracapture/lockedcameracaptureextension)
- [`LockedCameraCaptureUIScene`](https://developer.apple.com/documentation/lockedcameracapture/lockedcameracaptureuiscene)
- [`LockedCameraCaptureSession`](https://developer.apple.com/documentation/lockedcameracapture/lockedcameracapturesession)
- [`LockedCameraCaptureSession.sessionContentURL`](https://developer.apple.com/documentation/lockedcameracapture/lockedcameracapturesession/sessioncontenturl)
- [`LockedCameraCaptureSession.openApplication(for:)`](https://developer.apple.com/documentation/lockedcameracapture/lockedcameracapturesession/openapplication%28for%3A%29)
- [`LockedCameraCaptureManager`](https://developer.apple.com/documentation/lockedcameracapture/lockedcameracapturemanager)
- [`NSUserActivityTypeLockedCameraCapture`](https://developer.apple.com/documentation/lockedcameracapture/nsuseractivitytypelockedcameracapture)
- [`CameraCaptureIntent`](https://developer.apple.com/documentation/appintents/cameracaptureintent)
- [Creating controls to perform actions across the system](https://developer.apple.com/documentation/widgetkit/creating-controls-to-perform-actions-across-the-system)
- [Requesting authorization to capture and save media](https://developer.apple.com/documentation/avfoundation/requesting-authorization-to-capture-and-save-media)
- [`NSCameraUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nscamerausagedescription)

## Internal evidence paths

- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/LockedCameraCapture.framework/Headers/LockedCameraCapture.h`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/LockedCameraCapture.framework/Headers/LCCDefines.h`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/LockedCameraCapture.framework/Modules/LockedCameraCapture.swiftmodule/arm64e-apple-ios.swiftinterface`
- `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/LockedCameraCapture.framework/LockedCameraCapture.tbd`
- `PLAN_SWIFT_ABI_ASYNC.md`, `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`
- `docs/capabilities/capability-status.json`, row `109-extension-entitlement-capabilities-lockedcameracapture`

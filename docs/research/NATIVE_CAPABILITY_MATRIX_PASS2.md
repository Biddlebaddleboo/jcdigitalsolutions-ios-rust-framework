# Native Capability Elimination Matrix — Pass 2

Research date: 2026-10-07

This pass extends the native-path census into security, device services, background execution, media, sensors, communications, accessories, and extension-based capabilities.

## Executive finding

The second tier reinforces the first-pass conclusion: many capabilities that modern Apple documentation presents with Swift examples remain public Objective-C or C APIs, and the current objc2 ecosystem already has generated bindings for a substantial portion of them.

This means the framework can support a surprisingly broad set of iOS capabilities without introducing Swift runtime interoperability at all.

## Matrix

| Capability | Public native route | Existing Rust/objc2 evidence | Class | Conclusion |
|---|---|---|---|---|
| Keychain | Security C functions such as `SecItemAdd` / `SecItemCopyMatching` | `objc2-security` exposes C functions | R1/R3 | Excellent Rust fit; direct C ABI, thin typed Rust query builder optional. |
| Ordinary sandbox filesystem | POSIX/Rust std + `NSFileManager` for Apple-specific directory/coordination behavior | Foundation | R0/R1/R2 | Use Rust std/POSIX for ordinary I/O; use Foundation only for Apple-specific semantics. |
| Bluetooth | `CBCentralManager`, delegates | ObjC API confirmed | R2/R3 | No Swift ABI required. |
| Motion sensors | `CMMotionManager` | ObjC API confirmed | R2/R3 | No Swift ABI required. |
| Background task scheduling | `BGTaskScheduler` + Blocks | `objc2-background-tasks` exists | R2/R3 | No Swift ABI required for task registration/submission. |
| Audio graph / realtime audio | `AVAudioEngine` | `objc2-avf-audio` exists | R2/R3 | Native Objective-C path; keep DSP/data work in Rust. |
| Audio/video playback | `AVPlayer` | AVFoundation ObjC API | R2/R3 | No Swift ABI required. |
| Speech recognition | `SFSpeechRecognizer` | `objc2-speech` exists | R2/R3 | Native class/callback path available. |
| Pasteboard | `UIPasteboard` | UIKit | R2/R3 | Native ObjC path. |
| NFC tag access | Core NFC ObjC protocols/classes such as `NFCISO7816Tag` | Public ObjC protocol confirmed | R2/R3 | No Swift ABI required; entitlements/configuration remain separate requirements. |
| Health data | `HKHealthStore` and HealthKit object/query API | Apple class API; ObjC heritage | R2/R3 | Treat as native-object path unless a specific newer Swift-only HealthKit surface proves otherwise. |
| CallKit | `CXProvider`, delegates | `objc2-call-kit` exists | R2/R3 | No Swift ABI required. |
| PushKit | `PKPushRegistry`, delegate | `objc2-push-kit` exists | R2/R3 | No Swift ABI required. |
| DeviceCheck | `DCDevice` | `objc2-device-check` exists | R2/R3 | No Swift ABI required. |
| App Attest | `DCAppAttestService` completion-handler API | `objc2-device-check` exists | R2/R3 | No Swift ABI required; server verification remains application/server concern. |
| Calendar/reminders | `EKEventStore` | `objc2-event-kit` exists | R2/R3 | No Swift ABI required. |
| Game Center | `GKLocalPlayer` and GameKit classes | `objc2-game-kit` exists | R2/R3 | Native ObjC path remains available. |
| Packet tunnel VPN | `NEPacketTunnelProvider` subclass + packet flow | `objc2-network-extension` exists | R2/R3 | Major result: VPN provider can remain Rust/ObjC ABI; entitlement/extension packaging are separate. |
| Nearby Interaction | `NISession`, delegate | `objc2-nearby-interaction` exists | R2/R3 | No Swift ABI required for core session API. |
| HomeKit | `HMHomeManager` and HomeKit object model | Public ObjC API confirmed | R2/R3 | Core HomeKit capability remains native ObjC. |
| External accessories | `EAAccessoryManager`, `EAAccessory`, `EASession` | Public ObjC API confirmed | R2/R3 | No Swift ABI required. |
| Email/SMS composer | `MFMailComposeViewController`, `MFMessageComposeViewController` | Public UIKit-style ObjC classes | R2 | UI capability available without Swift. |
| PDF manipulation/rendering | `PDFDocument`, PDFKit | Public ObjC API confirmed | R2/R3 | No Swift ABI required. |
| File preview | `QLPreviewController` + data source | Public ObjC UIKit controller | R2 | No Swift ABI required. |
| Peer-to-peer MultipeerConnectivity | ObjC API exists | Public, but now deprecated | D | Do not invest; Apple says migrate to Network framework. |

## Keychain / Security

Apple's Keychain Services API is fundamentally C/CoreFoundation style. `SecItemAdd` accepts a `CFDictionary`, optionally returns a `CFTypeRef`, and returns an `OSStatus`.

The current `objc2-security` crate exposes `SecItemAdd` as an extern function.

**Framework decision:**
- implement the raw path as direct public Security ABI;
- consider a zero/near-zero-cost Rust query builder around strongly typed constants;
- do not route keychain operations through Swift/CryptoKit merely for convenience;
- preserve direct access to Security status codes and returned CF objects.

Apple:
https://developer.apple.com/documentation/security/secitemadd(_:_:)

objc2:
https://docs.rs/objc2-security/latest/objc2_security/fn.SecItemAdd.html

## Filesystem

Ordinary file reads/writes inside the app sandbox do not need Objective-C at all. Rust's filesystem APIs ultimately use native OS file operations.

Use Foundation `NSFileManager` selectively for Apple-specific behavior:
- locating semantic app directories;
- coordinated/document-provider access;
- iCloud/file-provider behavior;
- resource metadata where Foundation is the appropriate public API.

Apple explicitly documents an Objective-C `NSFileManager` API and recommends high-level Foundation APIs where APFS-aware behavior matters.

**Framework decision:** filesystem should be hybrid R0/R1/R2 rather than forcing every file operation through Foundation.

Apple:
https://developer.apple.com/documentation/foundation/file-system?language=objc
https://developer.apple.com/documentation/foundation/using-the-file-system-effectively?language=objc

## Core Bluetooth

`CBCentralManager` is exposed as an Objective-C class with delegate/dispatch-queue initialization.

**Framework decision:** define the required delegate in Rust and keep Bluetooth packet/application processing in Rust.

Apple:
https://developer.apple.com/documentation/corebluetooth/cbcentralmanager?language=objc

## Core Motion

`CMMotionManager` is an Objective-C class for accelerometer, gyroscope, magnetometer, and fused device-motion delivery.

**Framework decision:** native callbacks into Rust; high-rate math remains Rust data/math rather than Objective-C objects.

Apple:
https://developer.apple.com/documentation/coremotion/cmmotionmanager?language=objc

## BackgroundTasks

Apple exposes `BGTaskScheduler` as an Objective-C class and a Block-based registration method. `objc2-background-tasks` already includes the scheduler and task/request classes.

**Framework decision:** no reason to involve Swift concurrency merely to schedule background work. Bridge the launch Block cheaply, then execute application work under explicit iOS deadlines/cancellation semantics.

Apple:
https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler?language=objc

objc2:
https://docs.rs/objc2-background-tasks/latest/objc2_background_tasks/

## AVFAudio / AVFoundation playback

`AVAudioEngine` and `AVPlayer` are Objective-C `NSObject` classes. objc2 exposes AVAudioEngine.

For realtime audio, the framework should avoid introducing Rust-side locks/allocations in render-sensitive callbacks and keep DSP buffers/data layouts Rust-native.

Apple:
https://developer.apple.com/documentation/avfaudio/avaudioengine?language=objc
https://developer.apple.com/documentation/avfoundation/avplayer?language=objc

objc2:
https://docs.rs/objc2-avf-audio/latest/objc2_avf_audio/struct.AVAudioEngine.html

## Speech

The objc2 ecosystem exposes `SFSpeechRecognizer`.

**Framework decision:** use native task/request/callback objects; do not adopt Swift async solely for syntax.

objc2:
https://docs.rs/objc2-speech/latest/objc2_speech/struct.SFSpeechRecognizer.html

## Core NFC

Apple exposes tag types such as `NFCISO7816Tag` as Objective-C protocols and delivers them through session delegates. Use still requires the applicable entitlement and Info.plist declarations.

**Framework decision:** ABI access is straightforward R2; entitlement/configuration support belongs in packaging/compliance tooling, not a Swift bridge.

Apple:
https://developer.apple.com/documentation/corenfc/nfciso7816tag?language=objc

## CallKit and PushKit

`CXProvider` is an Objective-C class using `CXProviderDelegate`; `objc2-call-kit` already covers the provider and call actions.

`PKPushRegistry` uses an Objective-C delegate, and `objc2-push-kit` provides generated bindings.

**Framework decision:** a VoIP application's CallKit/PushKit integration can be Rust-first without Swift. Pay close attention to platform timing/lifecycle rules rather than language ABI.

Apple:
https://developer.apple.com/documentation/callkit/cxprovider?language=objc
https://developer.apple.com/documentation/pushkit/pkpushregistry?language=objc

objc2:
https://docs.rs/objc2-call-kit/latest/objc2_call_kit/
https://docs.rs/objc2-push-kit/latest/objc2_push_kit/

## DeviceCheck / App Attest

Apple exposes `DCAppAttestService` as `NSObject` in Objective-C with completion handlers for key generation, attestation, and assertions. `objc2-device-check` already provides the classes.

**Framework decision:** App Attest does not require Swift ABI. Expose the native client path; leave server-side attestation verification to application/server libraries.

Apple:
https://developer.apple.com/documentation/devicecheck/dcappattestservice?language=objc

objc2:
https://docs.rs/objc2-device-check/latest/objc2_device_check/

## EventKit

`EKEventStore` has a public Objective-C API for requesting access and manipulating calendar/reminder data. `objc2-event-kit` binds it.

Apple:
https://developer.apple.com/documentation/eventkit/ekeventstore?language=objc

objc2:
https://docs.rs/objc2-event-kit/latest/objc2_event_kit/struct.EKEventStore.html

## GameKit

`GKLocalPlayer` is exposed to Objective-C and the objc2 crate exposes authentication state and GameKit APIs.

**Framework decision:** do not assume modern Game Center examples imply Swift-only implementation.

Apple:
https://developer.apple.com/documentation/gamekit/gklocalplayer?language=objc

objc2:
https://docs.rs/objc2-game-kit/latest/objc2_game_kit/struct.GKLocalPlayer.html

## NetworkExtension / VPN

Apple's `NEPacketTunnelProvider` is the subclass point for custom packet tunnel extensions. It exposes packet flow and completion-handler methods. The NetworkExtension entitlement and extension packaging are required.

`objc2-network-extension` already exposes `NEPacketTunnelProvider`, including Block-based tunnel lifecycle methods.

This is especially important for the framework architecture: a packet-tunnel extension does **not** require a Swift application layer. The difficult pieces are Objective-C subclass/protocol correctness, extension entry/lifecycle, entitlements, packet ownership, and Xcode packaging.

Apple:
https://developer.apple.com/documentation/networkextension/nepackettunnelprovider

objc2:
https://docs.rs/objc2-network-extension/latest/objc2_network_extension/struct.NEPacketTunnelProvider.html

## Nearby Interaction

Apple exposes `NISession` as `@interface NISession : NSObject`, using delegate callbacks. `objc2-nearby-interaction` already contains NISession/configuration/delegate coverage.

Apple:
https://developer.apple.com/documentation/nearbyinteraction/nisession?language=objc

objc2:
https://docs.rs/objc2-nearby-interaction/latest/objc2_nearby_interaction/

## HomeKit

Apple still exposes `HMHomeManager` as an Objective-C object and uses a delegate for changes.

**Framework decision:** HomeKit's core capability is R2. Research newer Matter/Home APIs separately rather than treating all smart-home APIs as equivalent.

Apple:
https://developer.apple.com/documentation/homekit/hmhomemanager?language=objc

## ExternalAccessory

`EAAccessoryManager` and `EAAccessory` are public Objective-C classes; accessory sessions expose native stream-oriented communication.

Apple:
https://developer.apple.com/documentation/externalaccessory/eaaccessorymanager?language=objc
https://developer.apple.com/documentation/externalaccessory/eaaccessory?language=objc

## PDF / Quick Look / Message UI

PDFKit exposes `PDFDocument` as an Objective-C object.

Quick Look exposes `QLPreviewController` as a UIKit controller with a data source.

Message UI exposes Objective-C mail/message composition controllers.

These are low-risk R2 integrations; UI performance is dominated by the Apple frameworks themselves.

Apple:
https://developer.apple.com/documentation/pdfkit/pdfdocument?language=objc
https://developer.apple.com/documentation/quicklook/qlpreviewcontroller?language=objc
https://developer.apple.com/documentation/messageui/mfmailcomposeviewcontroller?language=objc
https://developer.apple.com/documentation/messageui/mfmessagecomposeviewcontroller?language=objc

## Deprecated path discovered: MultipeerConnectivity

Apple now marks the Multipeer Connectivity framework deprecated and explicitly directs developers to migrate to Network framework.

**Framework decision:** do not invest in a new abstraction around MCSession. If compatibility is ever required, expose existing native bindings narrowly; new peer networking should be designed on supported Network APIs.

Apple:
https://developer.apple.com/documentation/multipeerconnectivity?language=objc

## Research implications

After two passes, the expected high-frequency Swift-ABI surface is much smaller than the total Apple SDK surface.

The framework's native-first implementation can plausibly cover:
- general networking;
- filesystem;
- preferences;
- keychain/security;
- authentication;
- notifications;
- location;
- Bluetooth;
- motion;
- background task scheduling;
- audio/video;
- camera;
- Photos;
- Contacts;
- calendar/reminders;
- web;
- CloudKit;
- Core ML/Vision;
- Metal;
- maps;
- Core Data;
- NFC;
- CallKit/PushKit;
- App Attest/DeviceCheck;
- Game Center;
- VPN/NetworkExtension;
- Nearby Interaction;
- HomeKit;
- external accessories;
- PDF/file previews;
- common system composition UI;

without first solving Swift ABI.

The next priority should therefore shift increasingly toward **proving the residual list**, especially StoreKit 2, App Intents, newer Swift-only frameworks, and Swift-only additions to otherwise Objective-C-capable frameworks.

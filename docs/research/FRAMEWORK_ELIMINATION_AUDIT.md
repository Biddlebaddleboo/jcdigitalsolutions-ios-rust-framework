# Framework Elimination Audit: Rust Reimplementation vs Apple Dependencies

Research date: 2026-10-07

## Goal

Determine, framework by framework, how much Apple API surface should remain in the long-term architecture.

The governing rule is performance-driven:

> Reimplement an Apple/library-layer function in Rust only when an optimized Rust implementation has a credible path to outperform the Apple call or materially reduce allocations, copies, working-set size, cache misses, startup/runtime cost, binary/linkage cost, or energy for equivalent semantics.

Do **not** replace Apple code merely because a Rust rewrite is possible.

When Apple provides hardware acceleration, privileged system integration, background execution, protected storage, account services, entitlement-gated capabilities, or architecture-specific tuning that ordinary app code cannot reproduce, retain the Apple API and place the smallest possible Rust facade around it.

This document classifies the current framework census. It is family/capability level, not a claim that every symbol has been individually benchmarked.

## Classification

- **R — Rust replacement candidate**: most useful semantics can live in Rust; benchmark before replacing Apple path.
- **M — Mostly Rust + minimal OS primitive**: high-level/library logic can move to Rust but a small OS primitive remains.
- **H — Hybrid**: substantial Rust logic is useful, but the Apple framework remains an important capability boundary.
- **B — Bind/system-owned**: Apple framework is fundamentally the supported gateway to protected data, hardware, system UI, daemon, account, background execution, entitlement, or service.
- **A — Apple-performance-preferred**: public Apple implementation is likely difficult to beat because of hardware acceleration, architecture-specific kernels, or private internal tuning; retain unless a concrete benchmark proves otherwise.
- **C — Compiler/build/discovery contract**: runtime reimplementation alone cannot reproduce the integration.
- **X — Avoid/defer**: deprecated, redundant, or low-value to clone.

A framework can have more than one label, such as **H/A**.

---

# Decision rule

For every prospective Rust replacement:

1. Define the exact semantics being replaced.
2. Identify whether Apple performs privileged/system work unavailable to the process.
3. Identify whether Apple has hardware/private tuning advantages.
4. Build the Rust version behind the same framework-owned API.
5. Benchmark equivalent Release behavior on representative physical devices.
6. Keep Rust as default only if it wins the important metric(s) enough to justify maintenance.
7. Otherwise keep the Apple path and preserve the replacement seam.

Cross-platform portability by itself is not enough to select a slower implementation. The portable API may dispatch to an Apple-optimized backend on Apple hardware and a Rust implementation elsewhere.

---

# Foundation and basic runtime utilities

| Framework/family | Class | Reimplement public semantics? | Replace underlying capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| Foundation collections/value conveniences | R | Yes | Yes | Strong for compact Rust-native state and avoiding ObjC allocations | Prefer Rust internal types |
| NSString/String utilities | R/H | Mostly | Mostly | Rust may win for UTF-8-native workloads; Foundation may win for locale/Unicode-specialized paths | Benchmark hot operations; avoid conversions |
| NSData/Data utilities | R/H | Yes | Mostly | Rust slices/Vec can avoid object wrappers/copies | Rust internally, bridge only at boundary |
| Date/Calendar/DateComponents | R/H | Mostly | Locale/calendar DB partly system-owned | Simple arithmetic likely Rust-fast; locale/calendar rules uncertain | Rust simple time; Apple for locale/calendar semantics unless benchmarked replacement |
| URL/URLComponents | R | Yes | Yes | Rust parser can be faster/cache-friendlier | Strong candidate |
| JSON/property-list parsing | R | Yes | Yes | Strong Rust candidate if parser benchmarks win | Pluggable parser |
| NSCache/cache helpers | R | Yes | Yes | Rust compact cache can likely win for app-specific patterns | Strong candidate |
| Formatter families | R/H | Mostly | Locale data partly system-owned | Custom formats can win; localized formatting likely Apple-favored | Split portable formatting from locale backend |
| Bundle/process/environment helpers | H/B | API layer yes | System state no | Calls are not hot enough to justify cloning | Thin native boundary |
| NotificationCenter (in-process) | R | Yes | Yes | Rust event bus can be cheaper if needed | Do not import Foundation notification machinery by default |
| Timer/run-loop convenience | R/H | Yes | Kernel/run-loop scheduling no | Rust timer state possible; system wakeup remains | Runtime-neutral timers over OS primitive |

Apple Foundation is therefore not a single yes/no dependency. The project should progressively eliminate object-heavy utility usage while retaining system-facing pieces.

---

# Filesystem, persistence, and documents

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| FileManager ordinary file I/O | M | Yes | Kernel filesystem no | Direct Rust/POSIX path likely as fast or faster than ObjC wrappers | Prefer Rust filesystem API |
| POSIX/Darwin filesystem | B primitive | No need | No | Already minimal | Use as OS primitive |
| Core Data | R/H | Yes | Yes for ordinary persistence | Rust/SQLite can be leaner and more portable | Default Rust persistence; Core Data adapter optional |
| SwiftData | R | Yes | Yes for persistence semantics | Rust likely lower overhead for custom models | Do not depend on SwiftData |
| FileProvider | B/C | App logic yes | Files app/provider integration no | System integration dominates | Rust engine inside native extension shell |
| FSKit | B/C | Filesystem engine partly | extension/system mount contract no | Native shell required | Rust engine behind FSKit |
| PDFKit | R/H | Yes | System UI convenience only | Rust PDF engine may win for selective parsing; rendering is benchmark-dependent | Optional Rust engine; native UI adapter |
| QuickLook | B/H | Preview generation yes | system preview UI/service no | System UI wins integration | Rust generators + Apple presentation |
| QuickLookThumbnailing | H/A | Could | system thumbnail cache/service no | Apple service likely energy/cache optimized | Retain by default |
| UniformTypeIdentifiers | R/H | Yes | system UTI registry integration partly no | Portable MIME/type tables may be cheaper | Rust registry + Apple mapping |

---

# Networking

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| Network.framework | M/H | Protocol/app logic yes | kernel path/TLS/system policy no | Apple transport likely highly tuned; Rust can win above transport | Keep transport backend; Rust protocol layer |
| URLSession foreground HTTP | H | HTTP semantics yes | system transport not necessary in all cases | Rust HTTP stack may win in some app-specific paths | Benchmark Rust vs URLSession; do not assume |
| URLSession background transfer | B | Request facade yes | No | Apple separate-process transfer is unique | Retain |
| CFNetwork | M/H | Mostly | proxy/system config no | Often replaceable by Network/Rust | Avoid as core dependency |
| SystemConfiguration | B/H | Some | system configuration no | Not a hot path | Bind only when required |
| MultipeerConnectivity | X/H | Protocol yes | discovery/system integration partly no | Better to use Network/custom protocol | Avoid for new core |
| NetworkExtension | B | Tunnel protocol logic yes | system VPN/filter extension lifecycle no | Rust can optimize packet processing, but shell is mandatory | Rust engine inside NE |
| WebKit | B/H/A | App web logic yes | browser engine/system process no | Rebuilding browser stack for speed is unrealistic | Bind WKWebView |
| SafariServices | B | No useful replacement | No | System UI integration is the value | Bind |

Apple documents background URLSession sessions as handing transfers to a separate system process that can continue while the app is suspended or terminated; this is not reproducible as an in-process Rust HTTP client.

---

# Security, cryptography, identity, and privacy

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| CryptoKit software algorithms | R/A | Yes | Yes | Rust crypto can equal/win for some workloads, but benchmark vetted implementations | Pluggable Rust default only when faster |
| Security software crypto | R/H | Yes | Yes for software keys | Similar | Rust where benchmark wins |
| Keychain Services | B | Facade yes | No | System encrypted DB/access control is unique | Thin boundary |
| Secure Enclave / hardware-backed SecKey | B/A | No meaningful replacement | No | Hardware security wins by definition | Retain |
| LocalAuthentication | B | Facade yes | No | biometric/system UI capability | Retain |
| AuthenticationServices passkeys | B | Request model yes | No | iCloud Keychain + system authenticator | Retain |
| Sign in with Apple | B | Request model yes | No | Apple account/system auth | Retain |
| DeviceCheck | B | No | No | Apple-authenticated token/service | Retain |
| App Attest | B/A | Server parsing can be Rust | attestation key/certification no | Secure Enclave + Apple service | Retain client service |
| AppTrackingTransparency | B | No | No | OS permission state/UI | Retain |
| AdServices | B | No | No | Apple attribution service | Retain |
| SensitiveContentAnalysis | H/A | Alternate model possible | Apple model/service no | Apple may have tuned private models | Backend choice; benchmark alternative model |
| CryptoTokenKit | H/B | Token protocol logic yes | system token integration no | Rust protocol code may help | Hybrid |

Apple documents Keychain as an encrypted system database, passkeys as credentials backed by iCloud Keychain, and App Attest as creating hardware-based keys certified by Apple. Those underlying guarantees cannot be cloned by an app.

---

# Notifications, background execution, and lifecycle

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| UserNotifications | B | Scheduling/model logic yes | delivery no | System delivery is unique | Rust facade + Apple scheduler |
| PushKit | B | App protocol logic yes | push wake/delivery no | System-owned | Retain |
| BackgroundTasks | B | Task logic yes | scheduling/runtime grant no | System decides execution window | Retain shell |
| UIKit background task APIs | B | Work logic yes | background runtime no | System-owned | Retain |
| BackgroundAssets | B | Download processing yes | system background delivery no | System-owned | Retain |

Local notifications are delivered by the system even when the app is not running; BackgroundTasks scheduling is also system-controlled. A Rust runtime cannot replace those semantics.

---

# Location, sensors, Bluetooth, NFC, and nearby hardware

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| CoreLocation | B/A | Filtering/geofence logic yes | sensor fusion/location service no | Apple combines GPS/Wi-Fi/Bluetooth/cellular etc. | Retain boundary |
| CoreMotion | B/A | Signal processing yes | raw/processed sensor service no | Apple sensor fusion likely optimized | Rust analytics after data retrieval |
| CoreBluetooth | B | GATT/protocol logic yes | controller/radio access no | System Bluetooth stack is required | Rust protocol/state machine + CoreBluetooth |
| CoreNFC | B | NDEF/protocol parsing yes | NFC session/secure hardware no | Hardware access required | Rust parsers + CoreNFC |
| NearbyInteraction | B/A | App logic yes | UWB/spatial measurement service no | Hardware calibration/system access | Retain |
| SensorKit | B | Analytics yes | protected sensor repository no | Entitlement/system store | Retain |
| ExternalAccessory | B/H | Protocol parser yes | MFi/session/accessory transport no | System/MFi integration | Rust protocol engine |
| AccessorySetupKit | B | Config model yes | system accessory setup UI/permissions no | System integration | Retain |
| ThreadNetwork | B/H | protocol logic possible | radio/network credentials/system integration no | System stack likely preferred | Thin adapter |

Core Location explicitly aggregates multiple hardware sources; Core Bluetooth is the supported system stack for Bluetooth communication.

---

# Camera, audio, media, and codecs

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| AVFoundation capture | B/H/A | Capture graph facade yes | camera/mic device access no | Native device stack required | Retain capture boundary |
| AVFoundation media composition | R/H/A | Some processing yes | system codecs/timing integration partly no | Benchmark-specific | Rust only for proven hot algorithms |
| AVFAudio | H/A | DSP/app logic yes | audio session/hardware graph no | Apple audio path optimized | Rust DSP + native audio I/O |
| AudioToolbox/CoreAudio | H/A | DSP/codecs partly | hardware/audio session no | Very low-level optimized C APIs | Keep unless Rust benchmark wins |
| CoreMedia | H | Time/media structs can be Rust | Apple pipeline interop no | Mostly boundary types | Rust internally, convert at boundary |
| CoreVideo | H/A | Buffer math yes | zero-copy graphics/media buffer integration no | System memory pipeline advantage | Retain pixel-buffer boundary |
| VideoToolbox | A/B | Software codecs theoretically yes | hardware codec engines no | Hardware encode/decode is likely far faster/less energy | Retain by default |
| MediaPlayer | B/H | Queue/state can be Rust | system media/Now Playing integration no | System integration | Hybrid |
| ReplayKit | B | Processing yes | screen/system recording service no | System-owned | Retain |
| MediaAccessibility | B/H | Some semantics yes | system accessibility state no | Not hot | Bind |
| ShazamKit | H/A/B | Acoustic fingerprint algorithm can be implemented | Shazam catalog/service cannot | Apple service/catalog is unique | Rust custom-catalog engine only if benchmark/need |
| SoundAnalysis | H/A | Alternate ML possible | Apple models/framework optimizations not necessarily | Apple ML integration may win | Backend selection |

Apple documents VideoToolbox as direct access to hardware-accelerated encoders/decoders. Replacing it with a software Rust codec for "purity" would normally be a performance regression.

---

# Graphics, image processing, math, and GPU compute

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| CoreGraphics | H/A | Geometry/raster algorithms yes | system drawing/compositor integration no | Apple raster paths may be tuned | Rust geometry; benchmark raster replacements |
| QuartzCore/Core Animation | B/A | Animation state could be Rust | compositor/display server no | System compositor is irreducible | Retain |
| CoreText | H/A | Text layout/shaping theoretically yes | system fonts/locale integration partly no | HarfBuzz/Rust stack could compete, but broad parity expensive | Benchmark only targeted workloads |
| CoreImage | H/A | Filters can be Rust/Metal | no unique protected service | GPU pipeline optimized | Prefer CoreImage/MPS unless fused Rust/Metal path wins |
| ImageIO | R/H/A | Image parsing/encoding can be Rust | system codecs not essential | Rust libraries can win for selective formats; Apple broad support is efficient | Pluggable; benchmark per format |
| Metal | B/A | GPU algorithms written by app | GPU driver/API cannot be replaced | Direct GPU API already minimal | Retain |
| MetalKit | R/H | Convenience layer replaceable | drawable/display integration remains | Rust can remove convenience overhead if meaningful | Optional |
| Metal Performance Shaders | A | Algorithms can be reimplemented in shaders | GPU access remains Metal | Apple kernels tuned per GPU family | Strong presumption to retain |
| MPSGraph | A/H | Graph compiler could be replaced | compute devices remain | Apple optimizes across CPU/GPU/Neural Engine | Retain unless specialized Rust engine proves faster |
| Accelerate/vDSP/BLAS/LAPACK | A | Algorithms implementable | Yes | Apple architecture-tuned kernels likely hard to beat | Retain by default; benchmark specialized kernels |
| Compression | R/A | Algorithms implementable | Yes | Apple LZFSE/LZMESH/LZBITMAP optimized for Apple hardware; cross-platform codecs may favor Rust | Select fastest backend per codec/platform |
| ModelIO | R/H | Parsing/model transforms replaceable | some Metal/SceneKit integration only | Rust parsers may be leaner | Good candidate for targeted formats |
| SpriteKit/SceneKit | R/H/A | Engine theoretically replaceable | rendering still Metal | Custom Rust engine may win specialized workloads but huge scope | Not core target |

Apple states MPS kernels are tuned for the unique characteristics of each GPU family. Accelerate supplies Apple's BLAS/LAPACK implementation. The Compression framework includes algorithms explicitly optimized for Apple CPUs. These should not be replaced without direct benchmark evidence.

---

# Machine learning, vision, language, and speech

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| Core ML | H/A | Model runtime theoretically replaceable | Neural Engine access/scheduling not generally replaceable | Apple chooses CPU/GPU/Neural Engine based on overall cost | Retain for supported models |
| Vision | H/A | CV algorithms/models can be Rust | Apple model/optimized pipeline not necessarily | Could win for specialized classical CV; Apple likely strong for built-ins | Rust specialized algorithms; Vision backend otherwise |
| NaturalLanguage | R/H/A | Tokenization/classification/etc. can be Rust | Apple pretrained models/features no | Rust may win simple algorithms | Replace simple algorithms; retain unique models |
| Speech | B/H/A | Speech model could theoretically be custom | Apple recognizer/service/on-device models are unique | Custom model only if specifically better | Retain default |
| CoreML preprocessing/postprocessing | R | Yes | Yes | Strong Rust candidate due fewer copies/compact data | Keep in Rust |
| Foundation Models | B/H/A | Could use another model | Apple's on-device model access itself no | Apple system model is unique; custom model may be faster for narrow task | Pluggable AI backend |
| Translation | B/H/A | Own translation model possible | Apple's model/service unique | Compare model quality/latency/size | Retain Apple backend unless custom stack clearly wins |

Core ML's tooling shows it can schedule individual operations across CPU, GPU, and Neural Engine based on total transfer/ramp costs. A generic Rust inference engine should not replace it by default on Apple hardware.

---

# Photos, contacts, calendar, health, and personal data stores

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| Photos/PhotoKit | B | Asset-processing logic yes | user Photos/iCloud library no | Store/authorization is system-owned | Retain boundary |
| Contacts | B/H | Contact models/search logic yes | system Contacts database/permission no | System-owned | Retain store adapter |
| EventKit | B/H | Calendar logic yes | system calendar/reminders store no | System-owned | Retain store adapter |
| HealthKit | B | Analytics/modeling yes | health repository/authorization no | System-owned protected store | Rust analytics after query |
| HealthKitUI | B | No useful replacement for system integration | No | System UI | Bind only |
| HomeKit | B | Automation logic yes | home database/accessory authorization no | System-owned ecosystem | Rust logic + HomeKit |
| SafetyKit | B | No equivalent | No | OS safety service | Bind |
| ScreenTime / FamilyControls | B/C | Policy logic yes | entitlement/system enforcement no | System-owned | Retain shell |
| DeviceActivity | B/C | Analysis yes | monitored system activity no | System-owned | Retain |
| ManagedSettings | B/C | Policy model yes | OS enforcement no | System-owned | Retain |
| FinanceKit | B | Analytics yes | protected financial store no | Entitlement/service | Retain |

PhotoKit represents the Photos-managed local+iCloud library and enforces authorization. HealthKit is the central protected repository for health data. A private Rust database is not a replacement for either capability.

---

# Cloud, accounts, sharing, and system services

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| CloudKit | B | Client model/cache/sync planning yes | iCloud databases/sharing/account identity no | Service-owned | Rust cache/sync layer + CloudKit |
| iCloud document/key-value surfaces | B/H | local state yes | iCloud service no | Service-owned | Thin backend |
| MessageUI | B | App message composition model yes | system composer/account integration no | System UI | Bind |
| SharedWithYou | B | App model yes | system sharing metadata no | System-owned | Bind if needed |
| WatchConnectivity | B/H | Protocol serialization/state yes | paired-device transport/service no | OS transport | Rust protocol + native transport |
| GameKit | B/H | game logic yes | Game Center identity/matchmaking/leaderboards no | Service-owned | Rust gameplay + GameKit services |
| ClassKit | B | app progress model yes | Schoolwork/system service no | System-owned | Bind |
| PassKit / Apple Pay | B/A | cart/payment state yes | payment authorization/secure element/system UI no | Security/system path | Retain |
| CarPlay | B/C | app state yes | system vehicle UI/session no | System integration | Retain |
| CallKit | B | call state engine yes | system call UI/telephony coordination no | System integration | Rust call engine + CallKit |
| PushToTalk | B | voice protocol yes | system PTT lifecycle/background no | System integration | Hybrid |

CloudKit databases are Apple/iCloud services with public/private/shared stores tied to iCloud identity. That backend cannot be recreated locally while retaining CloudKit semantics.

---

# Commerce, attribution, and app ecosystem integration

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| StoreKit 2 | B | product/purchase state wrappers yes | App Store purchase/transaction authority no | Network/system service dominates | Implement ABI/binding, not replacement |
| Legacy StoreKit | B | Same | No | Same | Bind only where needed |
| AdAttributionKit | B | Attribution model yes | Apple attribution/postback service no | Service-owned | Bind |
| AppIntents | C/B | Intent logic yes | Siri/Shortcuts discovery/execution ecosystem no | Compiler/system integration dominates | Rust intent logic + metadata integration |
| WidgetKit | C/B | Data logic yes | widget host/timeline/render lifecycle no | System host | Rust provider logic where possible |
| ActivityKit | B/C | activity state logic yes | Lock Screen/Dynamic Island host no | System host | Rust logic + ABI/system shell |
| AlarmKit | B | scheduling model yes | OS alarm presentation/wake behavior no | System-owned | Bind |
| TipKit | R/H | eligibility/rules yes | Apple TipKit UI/state ecosystem optional | Rust implementation may be leaner | Rust tips unless system TipKit specifically desired |
| JournalingSuggestions | B | suggestion processing yes | protected/system suggestion source no | System-owned | Bind if needed |
| WeatherKit | R/B | Client SDK replaceable with REST | Apple weather service remains | Direct REST may reduce Swift dependency | Prefer REST/native HTTP backend |
| MusicKit | H/B | catalog client/state yes | Apple Music account/playback/service no | service/system playback | REST/MediaPlayer where faster/simpler; bind residual |
| MediaPlayer | B/H | queue modeling yes | system Now Playing/playback integration no | system-owned | Hybrid |

---

# AR, spatial, mapping, and environment

| Framework/family | Class | Reimplement semantics? | Replace capability? | Performance outlook | Recommendation |
|---|---|---:|---:|---|---|
| MapKit | B/H/A | Map UI engine theoretically yes | Apple Maps data/search/Look Around services no | Apple's tile/render/service integration strong | Bind for Apple maps; portable map backend abstraction |
| ARKit | B/A | SLAM theoretically possible | privileged camera/IMU calibration/system tracking integration no | Apple hardware calibration/tuning strong | Retain |
| RealityKit | R/H/A | scene/entity engine could be Rust | AR/system rendering integration partly | Custom engine may win specialized cases; large scope | Not core; prefer Metal/ARKit primitives if needed |
| RoomPlan | B/A | Reconstruction theoretically possible | Apple LiDAR/model pipeline unique | Apple likely optimized | Retain if used |
| DockKit | B | control logic yes | accessory tracking/system integration no | system-owned | Bind |
| NearbyInteraction | B/A | already above | No | hardware/UWB | Retain |

---

# Extension and entitlement-driven frameworks

| Framework/family | Class | Reimplement semantics? | Replace capability? | Recommendation |
|---|---|---:|---:|---|
| ExtensionKit / ExtensionFoundation | C/B | extension business logic yes | host/discovery/lifecycle no | Rust logic inside required shell |
| BrowserEngineKit | C/B | engine can be Rust | entitlement/process/system contracts no | Only for eligible browser projects |
| ContactProvider | C/B | provider data engine yes | contact-provider extension contract no | Rust provider behind shell |
| ManagedApp / ManagedAppDistribution | B/C | management logic yes | MDM/system distribution no | Bind |
| MarketplaceKit | B/C | marketplace logic yes | OS marketplace integration no | Bind |
| MatterSupport | B/C | Matter logic can be Rust | system request/extension/account integration no | Hybrid |
| SecureElementCredential | B | No practical replacement | secure element entitlement/hardware | Retain |
| CarKey | B | No practical replacement | automaker/MFi/UWB/NFC secure hardware | Retain |
| ProximityReader | B | payment flow logic yes | Tap to Pay system reader/security no | Retain |
| LockedCameraCapture | C/B | camera logic yes | lock-screen extension lifecycle no | Rust camera engine + Apple shell |

---

# Frameworks likely worth benchmarking for Rust replacement

These are the strongest candidates where an optimized Rust implementation could plausibly beat the Apple/library path for application-specific workloads:

1. URL/URI parsing and request construction.
2. JSON and other serialization/parsing.
3. App-specific caches.
4. Core Data/SwiftData-style persistence for non-CloudKit use.
5. Small-data Foundation value utilities that otherwise allocate Objective-C objects.
6. Natural-language tokenization/parsing without Apple pretrained models.
7. Image format parsing/metadata for a bounded set of formats.
8. PDF parsing for bounded application needs.
9. Crypto software algorithms using vetted optimized Rust implementations.
10. Compression for cross-platform formats when Rust backend benchmarks faster.
11. Model pre/post-processing around Core ML.
12. Classical computer vision or custom image processing when a fused Rust/Metal implementation beats generic Vision/CoreImage.
13. Media container/protocol parsing around AVFoundation/VideoToolbox.
14. Application protocol stacks above Network.framework.
15. Internal event/state/observation/reactive abstractions replacing NotificationCenter/Combine/Observation.
16. Transfer/model convenience layers replacing CoreTransferable-like abstractions.
17. Custom game/scene logic rather than GameplayKit/SceneKit convenience layers.

None of these should be selected on theory alone. Each needs an apples-to-apples benchmark.

---

# Frameworks with a strong presumption against reimplementation for performance

These either expose hardware acceleration or Apple-only optimization/service integration:

- Accelerate / vDSP / BLAS / LAPACK.
- Metal Performance Shaders.
- Metal Performance Shaders Graph.
- VideoToolbox hardware codecs.
- Core ML execution and Neural Engine scheduling.
- Core Animation compositor.
- ARKit tracking.
- Core Location sensor fusion.
- AVFoundation device capture.
- Core Audio/AudioToolbox hardware I/O.
- CoreVideo zero-copy media/GPU buffer integration.
- Apple Compression algorithms such as LZFSE/LZMESH/LZBITMAP when Apple-only payloads are acceptable.

A Rust alternative can still be kept as a cross-platform backend, but it should not become the Apple default without measured superiority.

---

# Frameworks whose underlying capability cannot be reimplemented by an ordinary app

The developer-facing facade can be Rust, but the actual service remains Apple-owned:

- Keychain / Secure Enclave.
- LocalAuthentication / Face ID / Touch ID.
- AuthenticationServices passkeys and Sign in with Apple.
- UserNotifications/APNs.
- BackgroundTasks and background URLSession execution.
- Core Location hardware/system service.
- Core Bluetooth radio stack.
- Core NFC hardware sessions.
- Nearby Interaction/UWB.
- Photos/iCloud Photos library.
- Contacts system store.
- EventKit system calendar/reminders.
- HealthKit store.
- HomeKit home database and accessory authorization.
- CloudKit/iCloud databases and identity.
- DeviceCheck/App Attest.
- StoreKit/App Store transactions.
- Apple Pay/PassKit.
- CarPlay.
- CallKit system call integration.
- NetworkExtension system VPN/filter lifecycle.
- system extension/provider hosts.
- AppIntents/Siri/Shortcuts discovery.
- WidgetKit/ActivityKit system hosts.
- Screen Time/ManagedSettings enforcement.
- protected entitlement frameworks such as CarKey, ProximityReader, SecureElementCredential.

---

# Architectural consequence

The framework should not aim for "zero Apple frameworks."

It should aim for:

> **zero unnecessary Apple framework work on hot application paths.**

The desired architecture is:

```text
developer-friendly portable API
        |
        v
compact/cache-friendly Rust implementation
        |
        +--> pure Rust when benchmark-proven faster
        |
        +--> Apple optimized library when Apple wins
        |
        '--> minimal Apple system-service boundary when capability is OS-owned
```

The backend can differ by platform and even by operation.

For example:

```text
Image resize
  iOS: CoreImage/MPS if benchmark winner
  Android: optimized Rust/GPU backend
  Linux: optimized Rust/SIMD backend
```

while:

```text
JSON parse
  all platforms: Rust parser if benchmark winner
```

and:

```text
secure credential storage
  iOS: Keychain
  Android: Keystore
  Windows: system credential/key APIs
  Linux: selected secure backend
```

The portable public API remains unchanged.

---

# Proposed implementation-time benchmark policy

Before replacing an Apple implementation, create a micro/meso benchmark that records:

- median and tail latency;
- CPU time;
- allocations;
- bytes allocated;
- copies/transcodes;
- peak/steady RSS where meaningful;
- hot working-set size;
- cache-sensitive throughput where measurable;
- energy/power on physical device for sustained work;
- binary/linkage delta;
- cold-start/setup cost;
- semantic differences.

Test at realistic data sizes. A tiny-input win can become a large-input loss and vice versa.

For Apple hardware-accelerated frameworks, benchmark energy and end-to-end latency, not only CPU time.

---

# Current conclusion

A substantial amount of Apple's **library-like developer surface** can eventually disappear from ordinary application code and move into optimized portable Rust.

However, attempting to reproduce the entire Apple OS API in Rust would be counterproductive. Most important Apple frameworks are clients of capabilities an app cannot own: protected databases, hardware controllers, daemon-managed scheduling, GPU/media engines, system UI, account identity, entitlements, and cloud services.

Therefore the highest-performance strategy is selective:

- **replace library abstractions only when Rust proves faster;**
- **retain Apple hardware/system paths when they win or are unique;**
- **keep every dependency behind a replaceable boundary;**
- **make the developer-facing API independent of that choice.**

## Primary Apple references checked

- Foundation: https://developer.apple.com/documentation/foundation
- URLSession background configuration: https://developer.apple.com/documentation/foundation/urlsessionconfiguration/background(withidentifier:)
- Network: https://developer.apple.com/documentation/network
- Core Location: https://developer.apple.com/documentation/corelocation
- Core Bluetooth: https://developer.apple.com/documentation/corebluetooth
- UserNotifications local delivery: https://developer.apple.com/documentation/usernotifications/scheduling-a-notification-locally-from-your-app
- Keychain Services: https://developer.apple.com/documentation/security/keychain-services
- AuthenticationServices passkeys: https://developer.apple.com/documentation/authenticationservices/supporting-passkeys
- App Attest: https://developer.apple.com/documentation/devicecheck/establishing-your-app-s-integrity
- Photos: https://developer.apple.com/documentation/photos/phphotolibrary
- HealthKit: https://developer.apple.com/documentation/healthkit
- BackgroundTasks: https://developer.apple.com/documentation/backgroundtasks
- AVFoundation capture: https://developer.apple.com/documentation/avfoundation/avcapturedevice
- VideoToolbox: https://developer.apple.com/documentation/videotoolbox
- Metal: https://developer.apple.com/documentation/metal
- Metal Performance Shaders: https://developer.apple.com/documentation/metalperformanceshaders
- Core ML performance: https://developer.apple.com/documentation/coreml/analyzing-a-core-ml-model-s-performance-in-xcode
- Accelerate BLAS: https://developer.apple.com/documentation/accelerate/blas-library
- Compression: https://developer.apple.com/documentation/compression
- MapKit: https://developer.apple.com/documentation/mapkit
- CloudKit: https://developer.apple.com/documentation/cloudkit

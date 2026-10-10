# PLAN_CAPABILITIES_ACTIVITYKIT.md — D98: Row 112 Feasibility Gate

## Installed tooling scope for future work

R1/R2 shared tooling is completed. Read `docs/SHARED_TOOLING.md` rather than implementing or inspecting shared tooling source. The configured `ios-activitykit-status` pilot is available through `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-activitykit-status`; inspect `--explain ios-activitykit-status` for exactly what it covers. The registered validator checks the scoped status pilot only; it does not prove Live Activity lifecycle, general Swift async or device-runtime behavior. Retain every historic D/B/F acceptance limit and compiler-oracle requirement; for new coverage add declarative schema-v1 gates and optional bounded adapters without weakening negative tests or treating unexecuted runtime checks as passed.

## Disposition

Row `112-compiler-build-host-capabilities-activitykit-support-if-layer-2-abi-work-is-proven` has one bounded partial candidate that does not require Layer-2 `ActivityAttributes` conformance or generic `Activity<Attributes>` support: `ActivityAuthorizationInfo().areActivitiesEnabled`, available from iOS 16.1. Apple defines the result as whether the current app can start a Live Activity; a person can disable Live Activities for an app in Settings.

The ActivityKit declarations are Swift-only, so this is not a generated Objective-C/Rust binding. B209 implements the bounded query in `ios-activitykit-status` using a compiler-derived `swiftcall` C thunk and the audited `swift-abi-core/apple-runtime` ownership path; no Swift source or reusable general method bridge is added. The selected operation needs only the public class initializer, a synchronous final `Bool` getter, and `ActivityKit.framework`.

The root aggregate marks row 112 partial (`B`) for this exact point-in-time start-eligibility snapshot after integrating the focused package and gate evidence. The current status reason about unproven Layer-2 ABI still applies to full typed ActivityKit operations, not to this bounded thunk. Do not claim full ActivityKit, Live Activity creation/update/end, or WidgetKit UI support from a Boolean snapshot.

## Audit scope

Audit row 112 only. Keep App Intents row 110 and WidgetKit row 111 separate. C7's `Metadata.appintents` no-go does not determine whether the synchronous ActivityKit authorization snapshot can use a dedicated, compiler-derived Swift call thunk.

## Installed public API and binding evidence

- The inspected toolchain is Xcode 26.6, build `17F113`, with iPhoneOS SDK 26.5. The installed ActivityKit Swift interface was built with Swift 6.3.2.
- `ActivityKit.swiftinterface` declares `ActivityAuthorizationInfo` as a public final class, available from iOS 16.1. Its public `init()` creates the authorization-info object. Its public final property is `areActivitiesEnabled: Swift.Bool { get }`; the property inherits the class's iOS 16.1 availability.
- Apple documents `areActivitiesEnabled` as a Boolean that indicates whether the app can start a Live Activity. It is a point-in-time value; `activityEnablementUpdates` is a separate `AsyncSequence` and is outside this slice.
- `ActivityKit.framework/Headers/ActivityKit.h` is an empty include guard and declares no Objective-C API. The public operation is present in the Swift module interface only.
- Local `objc2` 0.6.5 generated-framework coverage marks `ActivityKit` as `Swift-only`. No local generated ActivityKit Rust binding or repository Rust/C/Objective-C adapter was found.
- The installed `ActivityKit.tbd` exports symbols for the public class metadata, initializer, and getter: `_$s11ActivityKit0A17AuthorizationInfoCMa`, `_$s11ActivityKit0A17AuthorizationInfoCACycfC`, and `_$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg`. This supports symbol availability only; the exact Swift-call lowering and ownership sequence still require compiler-oracle proof. Do not infer signatures from mangled names.

## Why the status candidate avoids Layer 2

This one-shot query does not instantiate `Activity<Attributes>`, define an application-specific `ActivityAttributes` conformance, or pass a custom `ContentState`. The public `ActivityAttributes` protocol has an associated `ContentState` constrained to `Decodable`, `Encodable`, and `Hashable`; full typed ActivityKit operations therefore need protocol conformance, generic metadata, and value-handling support that are not proven here.

The D98 compiler-only follow-up derived and compared the exact lowering on arm64 iOS device and arm64 Simulator. Swift 6.3.3 emitted these calls for the temporary expression `ActivityAuthorizationInfo().areActivitiesEnabled`:

```text
call swiftcc %swift.metadata_response @"$s11ActivityKit0A17AuthorizationInfoCMa"(i64 0)
call swiftcc ptr @"$s11ActivityKit0A17AuthorizationInfoCACycfC"(ptr swiftself %metadata)
call swiftcc i1 @"$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg"(ptr swiftself %info)
call void @swift_release(ptr %info)
```

Clang 21.0.0 emitted matching `swiftcall` declarations from a temporary C thunk using `__attribute__((swift_context))` for both instance-context parameters:

```text
declare swiftcc { ptr, i64 } @"\01_$s11ActivityKit0A17AuthorizationInfoCMa"(i64 noundef)
declare swiftcc ptr @"\01_$s11ActivityKit0A17AuthorizationInfoCACycfC"(ptr noundef swiftself)
declare swiftcc i1 @"\01_$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg"(ptr noundef swiftself)
```

The metadata response is 16 bytes on these 64-bit targets (`ptr` plus `i64`, state offset 8). The public constructor result is owned: the Swift oracle releases it with `swift_release` after the getter. A C-facing wrapper can normalize the result to `uint8_t` (`zeroext i8`) after the getter and release. The matching shapes remove the earlier uncertainty about the symbols' call lowering and the one-shot object lifetime on these compiler/SDK targets; they do not create a general Swift method bridge.

Do not infer ABI from the mangled names or `.tbd` alone. The compiler-generated Swift and Clang IR above is the evidence for these exact call shapes. The `.tbd` confirms only exported symbol names. A maintained gate must preserve this compiler oracle and reject changed lowering.

## D98 device and Simulator ABI follow-up

- Swift and Clang compiler IR both used target triple `arm64-apple-ios16.1.0` for device and `arm64-apple-ios16.1.0-simulator` for Simulator. The metadata accessor, owned class initializer, getter, and release sequence matched for both targets.
- The iPhoneOS and iPhoneSimulator ActivityKit `.tbd` files both export the metadata accessor `_$s11ActivityKit0A17AuthorizationInfoCMa`, initializer `_$s11ActivityKit0A17AuthorizationInfoCACycfC`, and getter `_$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg`.
- The iPhoneOS `.tbd` declares target `arm64e-ios`, while the Simulator `.tbd` declares `x86_64-ios-simulator` and `arm64-ios-simulator`. The D98 symbol inspection alone was not an arm64 device link check. B209 adds an arm64 device and arm64 Simulator package link/import gate; this is link evidence only, not runtime evidence.
- This thunk's release path imports Swift runtime `swift_release`, unlike B61's static RoomPlan property query. B209 uses the audited `swift-abi-core/apple-runtime` path and verifies `_swift_release` from `libswiftCore` in linked device/Simulator examples; this establishes no runtime behavior.
- The synchronous lowering gives no guarantee about call latency, internal system-service work, or thread safety. Keep the operation synchronous on the caller's thread, expose no object across the boundary, and do not mark the Rust facade `Send`/`Sync` or claim real-time behavior without further evidence.
- The API and compiler oracle used deployment target iOS 16.1. Any implementation must either require that minimum or guard the call on older deployment targets; this audit gives no compatibility claim below iOS 16.1.

## User setting and host configuration boundary

- `areActivitiesEnabled` reflects whether the current app can start a Live Activity. Apple says people can deactivate Live Activities for an app in Settings. A query reads that status and does not itself invoke a request or present UI.
- A positive snapshot does not guarantee that a later `Activity.request` succeeds. Apple documents start errors, including when the person disabled Live Activities or the device reached its active/scheduled activity limit.
- An app that offers Live Activities must set the Boolean `NSSupportsLiveActivities` key in its host app `Info.plist`. That configuration is not written or validated by a status-only package. `NSSupportsLiveActivitiesFrequentUpdates` and frequent push authorization are separate and out of scope.
- No dedicated entitlement or privacy usage-description key for this one-shot query is established by the reviewed ActivityKit docs. Do not infer a permission grant or entitlement state from its Boolean.
- Live Activity display is implemented in a WidgetKit extension with SwiftUI `ActivityConfiguration`; that UI work remains separate from this row's scalar status candidate and from row 111's WidgetKit support.

## Full ActivityKit boundary

The candidate does not implement any of the following:

- user-defined `ActivityAttributes` / associated `ContentState` types and their serialization or generic metadata;
- `Activity.request`, activity enumeration/identity, state, update, end, dismissal, or expiration behavior;
- `ActivityAuthorizationInfo.activityEnablementUpdates`, which is async, or any ActivityKit `AsyncSequence`;
- frequent push authorization, push tokens, APNs server delivery, or remote start/update/end behavior;
- WidgetKit/SwiftUI presentation, App Intents, host lifecycle, Live Activity UI, or a production app extension.

Apple documents that `Activity.request` starts a Live Activity in the foreground unless an App Intent path is used. ActivityKit start requests can throw even after the status query returns true. These operation and presentation semantics are not represented by the Boolean snapshot.

## Feasibility result and next evidence

A status-only slice is technically feasible without Layer-2 type/conformance machinery. D98 proved the compiler-oracle ABI and one-object release sequence for the installed Swift 6.3.3 compiler and iOS 26.5 SDK at iOS 16.1 for arm64 device and Simulator compile targets. B209 adds and passes the maintained package gate and arm64 device/Simulator link/import inspection, including `_swift_release` from `libswiftCore`; runtime behavior remains unverified. No reusable direct framework initializer/property API exists in the common Swift ABI crates.

The B209 package exposes `activities_enabled_for_current_app() -> Result<bool, ActivityAuthorizationError>` with an iOS 16.1 API floor. It copies the Boolean into Rust-owned scalar state and releases the temporary ActivityKit object through `SwiftRetained<SwiftObject>`. It exposes no ActivityKit object, `Activity` identifier, content, token, callback, or async stream. The result reports current-app start eligibility only, not device-wide support, Activity creation success, extension presentation, or future state.

## Compiler-ABI follow-up

D98's initial source audit did not prove callable lowering. A focused follow-up compiled temporary Swift and Clang LLVM-IR oracles for arm64 iOS 16.1 and arm64 Simulator iOS 16.1. The metadata accessor, owned initializer, `swiftself` getter returning `i1`, and `swift_release` signatures matched, and both SDK `.tbd` files export the three API symbols

At D98 time this still did not establish a usable production path: no final link/import gate had run, the iPhoneOS `.tbd` listed `arm64e-ios` rather than proving arm64 device compatibility, and the owned initializer required `_swift_release`/`libswiftCore` linkage. B209 adds those static link/ownership checks; it does not validate runtime behavior

## Root integration needs

- D98 adds only this feasibility report. It does not change `docs/capabilities/capability-status.json`, root plans, workspace membership, Cargo lock, CI, indexes, or code.
- B209 adds the exact status query and final arm64 device/Simulator link/import checks at iOS 16.1. The compiler-oracle signature checks remain in the package gate. The link gate checks the `swift_release` runtime import and device-link compatibility rather than relying on the arm64e-only iPhoneOS `.tbd` target label.
- Root can revise the row 112 status reason to distinguish this status-only slice from broader ActivityKit protocol/generic/async work; preserve separate rows and reasons for App Intents and WidgetKit.
- The repository's planned Xcode baseline is 27.x. This audit uses Xcode 26.6 and makes no Xcode 27 compatibility claim.

## Evidence and checks

- Toolchain: `/Applications/Xcode.app/Contents/Developer`, Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5, Swift 6.3.3, and Clang 21.0.0. The imported ActivityKit Swift interface reports compiler version 6.3.2.
- Inspected `ActivityKit.framework/Modules/ActivityKit.swiftmodule/arm64e-apple-ios.swiftinterface`, `Headers/ActivityKit.h`, and `ActivityKit.tbd` in the iPhoneOS 26.5 SDK.
- Inspected `objc2-0.6.5/src/topics/about_generated/list_unsupported.md`, the current row 112 JSON entry, `PLAN_SWIFT_ABI.md`, `docs/swift-abi/SYNCHRONOUS_BOUNDARY.md`, and the B61 `PLAN_IOS_ROOMPLAN.md`, C shim, and compiler-oracle gate.
- Temporary oracle files were created outside the repository. The compile-only commands were `xcrun --sdk iphoneos swiftc -target arm64-apple-ios16.1 -sdk <iPhoneOS26.5.sdk> -module-name ActivityKitAuthorizationOracle -parse-as-library -emit-ir ...`, `xcrun --sdk iphoneos clang -target arm64-apple-ios16.1 -isysroot <iPhoneOS26.5.sdk> -std=c11 -S -emit-llvm ...`, and the corresponding Simulator commands with `--sdk iphonesimulator`, `arm64-apple-ios16.1-simulator`, and `<iPhoneSimulator26.5.sdk>`.
- Both ABI-oracle/thunk comparisons passed, and both `.tbd` symbol checks passed. No executable or linked binary was produced; no tests, ActivityKit calls, or runtime probes ran. No file beyond this report was changed.

## Apple references

- [ActivityKit framework](https://developer.apple.com/documentation/activitykit)
- [`ActivityAuthorizationInfo`](https://developer.apple.com/documentation/activitykit/activityauthorizationinfo)
- [`ActivityAuthorizationInfo.areActivitiesEnabled`](https://developer.apple.com/documentation/activitykit/activityauthorizationinfo/areactivitiesenabled)
- [`ActivityAuthorizationInfo.init()`](https://developer.apple.com/documentation/activitykit/activityauthorizationinfo/init%28%29)
- [Displaying live data with Live Activities](https://developer.apple.com/documentation/activitykit/displaying-live-data-with-live-activities)
- [`NSSupportsLiveActivities`](https://developer.apple.com/documentation/bundleresources/information-property-list/nssupportsliveactivities)
- [`ActivityAttributes`](https://developer.apple.com/documentation/activitykit/activityattributes)
- [`Activity.request(attributes:content:pushType:)`](https://developer.apple.com/documentation/activitykit/activity/request%28attributes%3Acontent%3Apushtype%3A%29)

## B209 — current-app Live Activity start eligibility

`ios_activitykit_status::activities_enabled_for_current_app()` returns
`Result<bool, ActivityAuthorizationError>`. On iOS it constructs
`ActivityAuthorizationInfo`, calls only `areActivitiesEnabled`, copies that scalar to Rust, and
releases the owned Swift object through `swift-abi-core/apple-runtime`. The value keeps Apple's
meaning: whether ActivityKit currently reports this app can start a Live Activity. It is not a
device-support result or a guarantee that a subsequent start succeeds. Non-iOS targets return
`NativeApiUnavailable`; unavailable weak-linked API symbols also return that error.

The SDK exposes the class and property as Swift-only declarations. The public iOS 26.5 SDK exports
the class metadata accessor `_$s11ActivityKit0A17AuthorizationInfoCMa`, initializer
`_$s11ActivityKit0A17AuthorizationInfoCACycfC`, and getter
`_$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg`. Apple documents the
property as the current app's ability to start a Live Activity and directs apps to query it before
showing the start UI. The class/API floor is iOS 16.1. The C thunk calls only these public symbols,
uses the compiler-derived metadata response and `swiftself` signatures, weak-imports those API
symbols, and does not author or ship Swift source. The Rust function is synchronous; it makes no
main-thread or arbitrary-thread-safety claim and does not hop queues.

`NSSupportsLiveActivities` is host configuration for apps that offer Live Activities; this crate
does not validate or write that key, provide a WidgetKit extension or SwiftUI presentation, or
perform any ActivityKit lifecycle operation. No dedicated entitlement or usage-description key for
this snapshot is established by the reviewed docs. `true` does not establish that a later
`Activity.request` succeeds, that a Live Activity is active, or that rendering is configured.

The focused gate `sh platform/ios/ios-activitykit-status/check.sh` passed locally with Rust 1.94.1:
package formatting; host, iOS device, and arm64 Simulator checks; strict Clippy for all three;
warnings-denied host/device rustdoc; compiler-oracle comparison for metadata, init, getter, and
release on arm64 device and Simulator; and link/import inspection for ActivityKit and
`libswiftCore` on iOS device and Simulator. `cargo +1.94.1 xtask docs-check` and
`cargo +1.94.1 xtask zero-swift-source` also passed. The link examples were built and inspected,
not executed. No tests, ActivityKit calls, app launches, runtime probes, or device/Simulator queries
ran for B209.

This used Xcode 26.6 build `17F113`, iPhoneOS/iPhoneSimulator SDK 26.5, Swift 6.3.2/6.3.3, Clang
21.0.0, and Rust 1.94.1. The installed Xcode remains below the repository Xcode 27.x baseline, so
this does not establish baseline-toolchain behavior. The shared aggregate now marks row 112 `B` only for this start-eligibility snapshot; full ActivityKit and WidgetKit remain unsupported. Its package gate is wired in macOS CI, with no passing workflow run recorded.

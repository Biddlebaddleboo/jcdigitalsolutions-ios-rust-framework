# PLAN_VALIDATION_IOS_CALL_OBSERVER.md — G67 CallKit active-call snapshot

## Scope

G67 validates D76/B72's iOS-only `CXCallObserver.calls` snapshot. The backend copies a call count
and aggregate outgoing, connected, on-hold, and ended flags. It exposes no call object, UUID, caller
data, callback, call control, provider, PushKit, or audio API. The initial synchronous native read
may block.

## Gate

`sh platform/ios/callkit/ios-call-observer/check.sh` passed after integration into the root Cargo
workspace. The gate includes formatting, host/device/Simulator checks, strict Clippy, iOS rustdoc,
feature isolation, and device/Simulator Release link/import inspection.

The inspected probes import exactly CallKit, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`.
Their minimum OS versions are iOS 10.0 on device and iOS 14.0 on Simulator. These probe minimums
do not change the iOS 10.0 API floor. Probe binaries were not executed; no live-call, timing, or
callback behavior is established. Local evidence used Rust 1.94.1, Xcode 26.6 build 17F113, and
iOS SDK 26.5, below the plan's Xcode 27.x baseline.

## Limits

Compile, Clippy, rustdoc, and link/import results validate the bounded source and link surface only.
They do not establish call observation on a device, UI-thread safety for the synchronous initial
read, call lifecycle delivery, or CallKit service behavior.

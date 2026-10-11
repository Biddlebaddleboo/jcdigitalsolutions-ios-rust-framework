# PLAN_IOS_CALL_PROVIDER_OUTGOING.md — Workstream B540: CallKit outgoing transactions

## Objective

Extend `platform/ios/callkit/ios-call-provider` with the smallest reusable outgoing-call transaction flow missing from the existing provider: submit one public `CXStartCallAction` through `CXCallController`, deliver CallKit's start action through the existing action-handler API, and report the app's connecting and connected milestones in valid order.

This workstream is separate from B538 selectable `UITextView` and B539 UDP/TCP networking. It does not implement a complete VoIP service.

## Contract and boundaries

- `OutgoingCall` owns a framework `CallId`, one configured public CallKit handle kind/value, and an optional video request.
- `CallProvider::request_outgoing` submits one action transaction. Its asynchronous completion reports only CallKit transaction acceptance/rejection; acceptance does not mean signaling was sent or a remote peer connected.
- `CallProviderEvent::StartRequested` and `CallActionKind::Start` use the existing synchronous or deferred action-resolution contract. The application performs its own signaling and decides whether to fulfill the start action.
- Per-call state permits the connecting report only after successful start-action fulfillment, and the connected report only after connecting. Call end is valid after start-action fulfillment.
- Provider reset, invalidation, duplicate IDs, rejected transactions/actions, stale completions, and invalid transition order remain explicit failure cases.
- The application owns destination identity/ACK policy, signaling, PushKit/APNs, audio-session configuration, media transport, background execution, call-directory extensions, default-calling-app role, and runtime recovery policy.
- Only public CallKit APIs are used. No Swift source, private runtime symbol, C ABI expansion, or portable-core API is introduced.

## Acceptance

1. Host tests cover outgoing input constraints, transaction/action state transitions, ordered progress reports, rejection, duplicate IDs, reset/stale completion, and invalid ordering.
2. The iOS device and Simulator targets compile with the package's existing feature configuration and strict Clippy settings.
3. Release link probes preserve the package's framework/deployment contract while requiring the public controller, start action, transaction selector, start delegate method, and outgoing progress-report selectors. Existing negative checks continue to reject Swift runtime, PushKit, call-directory, audio, UIKit, and Python imports/strings.
4. Rust documentation describes the separation between transaction acceptance, app signaling, and CallKit progress reports.
5. No Simulator or physical-device runtime behavior is claimed by compile or link evidence.

## Implemented package surface

- `platform/ios/callkit/ios-call-provider/src/types.rs`: `OutgoingCall`, start event/action types, outgoing-input validation, and explicit invalid-state error.
- `platform/ios/callkit/ios-call-provider/src/registry.rs`: outgoing start and progress state transitions with deterministic negative regressions.
- `platform/ios/callkit/ios-call-provider/src/platform.rs`: `CXCallController` transaction submission, start-action routing/resolution, and ordered public CallKit progress reports.
- `platform/ios/callkit/ios-call-provider/check-link-imports.sh` and `examples/ios_call_provider_link_probe.rs`: device/Simulator release link and import/string/deployment checks for the added API.
- `platform/ios/callkit/ios-call-provider/README.md`: package-level API and non-claim documentation.

## Validation performed on base `0a54b83c6651876ea8e939e32981a2b438363067`

- `cargo +1.94.1 test --locked --offline -p ios-call-provider` — passed, 31 tests.
- `cargo +1.94.1 clippy --locked --offline -p ios-call-provider --all-targets -- -D warnings` — passed.
- `cargo +1.94.1 check --locked --offline -p ios-call-provider --target aarch64-apple-ios` — passed.
- `cargo +1.94.1 check --locked --offline -p ios-call-provider --target aarch64-apple-ios-sim` — passed.
- Strict target Clippy for `aarch64-apple-ios` and `aarch64-apple-ios-sim` — passed.
- `cargo +1.94.1 fmt -p ios-call-provider -- --check` — passed.
- `cargo +1.94.1 doc --locked --offline --no-deps -p ios-call-provider` — passed.
- `sh -n platform/ios/callkit/ios-call-provider/check-link-imports.sh && sh platform/ios/callkit/ios-call-provider/check-link-imports.sh` — passed for device iOS 10.0 and Simulator iOS 14.0 after the gate was narrowed to strings actually emitted by the compiler.

The root capability profile and capability registry are integration-owned and are updated separately to require the positive CallKit controller/start-action surface while retaining narrow prohibited-API guards. These package checks establish compilation and final-binary linkage only, not CallKit runtime behavior.

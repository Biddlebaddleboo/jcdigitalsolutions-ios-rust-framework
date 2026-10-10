# PLAN_IOS_ACCESSIBILITY.md — active B8 iOS accessibility backend

## Current evidence / baseline
The prior 179 KB rolling record of approximately 99 bounded UIKit accessibility metadata slices is retained verbatim in Git at `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247:PLAN_IOS_ACCESSIBILITY.md`. Implementations live in `platform/ios/ios-accessibility/src/{lib,platform,container_type}.rs`; the developer contract is `docs/ios/accessibility.md`. The manifest row B8 is **partial**, not a complete accessibility framework. On Xcode 26.6/SDK 26.5 numerous static checks passed; this does not validate real VoiceOver, Switch Control or physical-device interactions.

## Installed tooling integration

The R1/R2 build and validation engines are completed external PATH tools, not workstreams to implement. Consult `docs/SHARED_TOOLING.md`, `tools/validation/specs/validation-v1.json`, and `tools/validation/specs/schema-v1.json` for the supported contract. Accessibility is not automatically covered merely because the validator runs four current pilots. Retain the existing `ios-accessibility` checks and their unique UIKit/availability/import/negative assertions until a scoped validation profile and regression parity are established. Add only required declarative checks or bounded Python adapters; report unexecuted assistive-tech/device gates separately. Do not retrieve historical tooling engine source.


## Implementation scope and shared invariants
Start with `AccessibilityMetadata`, `AccessibilityTrait`, focus/status snapshots, `ios-accessibility::platform`, the relevant `objc2-ui-kit` typed API and `docs/SHARED_TOOLING.md`. Keep metadata on caller-owned live UIKit objects; retain main-thread proofs, nil/object lifetime handling, dynamic selector/version guards, explicit unknown-native-value handling and no unexpected UI mutation. Values for navigation style, accessibility language, preferred content-size category and text metadata are properties/snapshots; they do **not** guarantee assistive-tech output.

## Residual and non-goals
Do not infer full accessibility parity from scalar getter count. Verify integration with actual UI widgets, semantics for required activation/scroll/actions, focus events/notifications, dynamic changes, VoiceOver/assistive output and device behavior as *separate explicitly scoped* future work. Preserve every native availability floor, trait-specific action requirement and no-swift/public-API restriction. Do not fabricate input behavior merely because a trait was set.

## Tests / handoff
Use existing `ios-accessibility` checks, `cargo +1.94.1 fmt --all -- --check`, locked device/simulator Cargo check and strict Clippy, SDK/import audits, `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`. Add deterministic trait/enum conversion tests and real assistive-tech/device tests before claiming runtime parity. Read the historical plan only for a disputed slice-specific invariant. Report changed symbols, toolchain, checks, incomplete physical-device evidence and SHA.

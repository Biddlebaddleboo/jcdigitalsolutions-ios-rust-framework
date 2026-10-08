# PLAN_IOS_ACCESSIBILITY.md — Workstream B8: UIKit Accessibility Metadata

## Objective

Add a capability-scoped iOS extension for accessibility metadata on caller-owned UIKit `UIView` instances, using public `objc2` bindings and preserving native UIKit behavior

This is a bounded metadata adapter, not a reusable view system or a claim of full accessibility coverage

## Dependencies

- Foundation A and the iOS `ios-runtime` main-thread contract are integrated
- Use the active Xcode SDK and existing workspace `objc2` dependencies; do not add a Swift source layer
- This workstream does not depend on Swift ABI or other capability backends

## Read first

- `PLAN.md`
- `PLAN_IOS_NATIVE.md`
- `PLAN_CAPABILITIES.md`
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/UNSAFE.md`
- `docs/ios/runtime.md`
- Current `objc2-ui-kit` generated bindings for the selected public accessibility properties

## Write scope

- `platform/ios/ios-accessibility/**`
- `docs/ios/accessibility.md`
- `PLAN_IOS_NATIVE.md` only to link this B8 decomposition

Do not edit examples, other platform crates, shared capability manifests/indexes, workspace dependency versions, or CI. The orchestrator owns shared manifest and CI updates

## Required behavior

- Provide a small iOS-only crate for setting caller-selected accessibility element state, label, hint, value, and a bounded set of standard traits on a borrowed `UIView`
- Use the public `objc2-ui-kit` accessibility bindings; keep generated dependency types and conversions behind the crate except for the unavoidable borrowed UIKit view boundary
- Require the existing typed main-thread proof, remain `!Send`, and perform synchronous property updates on the caller's main thread
- Define whether each setter replaces or clears the prior value. Nil/absent text must clear the matching native property; setting traits must have explicit replacement semantics
- Use framework-owned trait names/bit flags where practical; map only traits whose constants and availability are verified from the active public SDK. Do not expose arbitrary raw pointers or guessed integer masks
- Record the exact SDK declarations, framework imports, deployment floor, and whether device/simulator results are compile/link-only
- Use no private selectors, private APIs, hand-written Objective-C runtime calls, Swift source, UIAccessibility callback blocks, or accessibility runtime clone
- Mark the capability partial. Do not claim VoiceOver UX, announcements, focus movement, custom actions, custom accessibility elements/containers, SwiftUI, or complete accessibility coverage

## Non-goals

- Creating, laying out, retaining, or rendering UIKit views
- Native view/controller lifecycle or general presentation
- Dynamic accessibility blocks, announcements, focus APIs, custom actions, or custom `UIAccessibilityElement` implementations
- Portable accessibility contracts or changes to `framework-ui`
- Live VoiceOver/screen-reader behavior claims

## Status

B8's implementation and guide meet the bounded metadata contract. The public API borrows only a
`UIView` and the typed `MainThread` proof, stores the main-thread marker, and is `!Send`/`!Sync`.
Each setter is synchronous; text `None` clears through `nil`, `Some("")` remains an empty string,
and `set_traits` replaces the full trait mask, using UIKit's `UIAccessibilityTraitNone` for an empty
set. The seven framework-owned traits map to public constants from the active SDK. Focused tests
cover optional text, trait combination, empty-set behavior, order independence, and deriving masks
only from the passed traits. No source or guide correction was needed.

On Xcode 26.6 build 17F113 with iPhoneOS and iPhoneSimulator SDK 26.5, these checks pass:

- `cargo test --locked -p ios-accessibility` (3 focused unit tests pass; 0 doc tests)
- `cargo check --locked -p ios-accessibility`
- `cargo check --locked -p ios-accessibility --target aarch64-apple-ios`
- `cargo check --locked -p ios-accessibility --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-accessibility --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo xtask docs-check`
- `cargo xtask zero-swift-source`
- `git diff --check`

Inspection of `UIAccessibility.h` confirms the selected nullable copied text properties and
synchronous accessibility setters; `UIAccessibilityConstants.h` declares `UIAccessibilityTraits`
as `uint64_t` and all eight referenced public constants. The header's only explicit iOS minimum in
the selected trait set is `UIAccessibilityTraitHeader` at iOS 6.0. The SDK's
`SupportedTargets.iphoneos.MinimumDeploymentTarget` and
`SupportedTargets.iphonesimulator.MinimumDeploymentTarget` values are 12.0.
`objc2-ui-kit` 0.3.2 uses the direct features `UIAccessibility`, `UIAccessibilityConstants`,
`UIResponder`, and `UIView`; Foundation uses `NSString`.

A temporary external Rust `cdylib` consumer linked for device and simulator with
`IPHONEOS_DEPLOYMENT_TARGET=12.0`. `vtool -show-build` reports device minimum iOS 12.0 / SDK 26.5
and simulator minimum iOS 14.0 / SDK 26.5. `otool -L` reports UIKit, Foundation,
`/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `nm -u` reports the seven selected
trait constants, `UIAccessibilityTraitNone`, and `objc_msgSend`. This is compile/link evidence only.
The host remains below the Xcode 27.x planning baseline; no simulator launch, device run, live
VoiceOver session, announcement, focus movement, accessibility audit, or UX behavior is claimed.
No CI/G8 files were changed.

## Validation and handoff

- Add focused conversion/trait-mapping checks where behavior is framework-owned; do not simulate VoiceOver
- Run `cargo fmt --all -- --check`, the crate checks, device/simulator `cargo check`, Clippy with `-D warnings`, SDK/link inspection, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`
- Report changed files, commit SHA, exact SDK/API availability, checks and evidence limits, deviations, and unresolved assumptions. Do not push

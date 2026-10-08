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

## Validation and handoff

- Add focused conversion/trait-mapping checks where behavior is framework-owned; do not simulate VoiceOver
- Run `cargo fmt --all -- --check`, the crate checks, device/simulator `cargo check`, Clippy with `-D warnings`, SDK/link inspection, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`
- Report changed files, commit SHA, exact SDK/API availability, checks and evidence limits, deviations, and unresolved assumptions. Do not push

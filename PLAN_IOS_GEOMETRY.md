# PLAN_IOS_GEOMETRY.md — Workstream B21: CoreGraphics Frame Intersection

## Status

B21 adds `ios_ui::geometry::intersection` over D16 finite `Frame` values. It uses three public
CoreGraphics C functions through private Rust declarations and checks the native result against the
portable contract. `check-link-imports.sh` passes for device and Simulator: C/Rust LP64 layouts
agree, dead-strip leaves only `CoreGraphics` and `libSystem.B.dylib`, and the linked probe has the
three expected CoreGraphics symbols. Locked device/Simulator checks and strict Clippy pass. The
probe does not execute the API; runtime parity remains unverified

## Objective

Provide one bounded CoreGraphics intersection adapter using the existing `ios-ui` crate. Keep
`framework-ui` portable and leave D14 UIKit control APIs unchanged

## API and behavior

- Expose `ios_ui::geometry::intersection(first: Frame, second: Frame) -> Result<Option<Frame>>`
- Require both values to share a coordinate space and logical units
- Use D16's `Frame::intersection` first to validate input edges and define the expected result
- Call `CGRectIntersection`, then `CGRectIsNull` and `CGRectIsEmpty`; map native empty/null to
  `None` only when D16 also returns `None`
- Return a positive-area native `Frame` only when its fields are finite and exactly equal the D16
  result. Preserve D16 `InvalidInput`; map invalid or nonmatching native output to
  `framework_core::ErrorKind::Platform`
- Use `objc2_core_foundation::{CGRect, CGPoint, CGSize}` from the existing `CFCGTypes` feature
- Keep the three `unsafe extern "C"` declarations private to `ios-ui`; link the public
  `CoreGraphics` framework. No public C ABI/header, new crate, or Swift source is added

The installed iOS SDK marks all three functions available since iOS 2.0. The repository does not
declare an effective package deployment minimum. The link probe's iOS 12.0 device and iOS 14.0
Simulator settings are validation targets, not product minimums. Current geometry layout probes
target LP64/64-bit iOS, matching the existing `Frame`-to-`CGFloat` conversion

## Dependencies and write scope

- Existing dependencies only: `framework-core`, `framework-ui`, and workspace-pinned
  `objc2-core-foundation` with `CFCGTypes`
- Write scope: `platform/ios/ios-ui/**`, `docs/ios/ui.md`, and this plan
- Do not edit Cargo manifests/lockfile, shared capability metadata/counts, indexes, CI, shared
  validation docs, or D14 control semantics; root/G15 owns integration and CI

## ABI and validation evidence

- Header signatures: `CGRectIntersection(CGRect, CGRect) -> CGRect`, `CGRectIsNull(CGRect) -> bool`,
  and `CGRectIsEmpty(CGRect) -> bool`; the SDK declares C linkage and includes `<stdbool.h>`
- `objc2-core-foundation` supplies `#[repr(C)]` geometry structs and target-sized `CGFloat`, not
  bindings for these functions. The local declarations use these public types by value
- `platform/ios/ios-ui/probes/geometry-layout.c` checks C sizes, alignments, and field offsets;
  `examples/ios_ui_geometry_link_import_probe.rs` checks Rust layouts and references all three
  symbols; `check-link-imports.sh` builds both targets and inspects imports
- `platform/ios/ios-ui/check-link-imports.sh`: pass on `aarch64-apple-ios` and
  `aarch64-apple-ios-sim`; after `-Wl,-dead_strip_dylibs`, each has exact direct imports
  `CoreGraphics` and `libSystem.B.dylib`; each references `_CGRectIntersection`, `_CGRectIsNull`,
  and `_CGRectIsEmpty`
- `cargo +1.94.1 check --locked --offline -p ios-ui --all-targets` and strict Clippy pass on host
- `cargo +1.94.1 clippy --locked --offline -p ios-ui --all-targets --target aarch64-apple-ios -- -D warnings`: pass
- `cargo +1.94.1 clippy --locked --offline -p ios-ui --all-targets --target aarch64-apple-ios-sim -- -D warnings`: pass
- Locked `ios-ui` checks pass for device and Simulator
- `cargo +1.94.1 xtask docs-check`, scoped formatting, shell syntax, `git diff --check`, and scoped
  trailing-whitespace scan: pass
- Required evidence is C/Rust LP64 layout agreement, no Swift/Python import, and portable D16 tests
- Build/link evidence does not execute native geometry and does not establish runtime parity;
  report this limit explicitly

## Exclusions

No CGRect public handle, raw C API, general CoreGraphics/QuartzCore facade, null/infinite inputs,
union, containment, points, paths, transforms, UIKit views, main-thread state, Swift ABI, runtime
parity claim, or performance claim

# PLAN_VALIDATION_IOS_GEOMETRY.md — Workstream G15: iOS Finite Geometry Gates

## Status

G15 local validation passed on 2026-10-08 with Rust 1.94.1, Xcode 26.6 (build 17F113), and
iPhoneOS/iPhoneSimulator SDK 26.5. Locked device/Simulator checks, strict all-target Clippy, and
the linked import/layout script passed. Artifacts were built but not executed; no CI workflow run,
runtime parity, or visual behavior is claimed

## Objective

Add persistent device/Simulator compile, strict Clippy, and focused CoreGraphics link/layout
checks for D16 `framework-ui::Frame::intersection` and B21 `ios-ui` finite-frame intersection

## Dependencies

- D16 portable finite-frame intersection in `framework-ui`
- B21 `ios-ui::geometry::intersection` backend, reusing the existing `Frame` contract
- G1 validation tooling and CI; existing D14 `ios-ui` package and target support

## Write scope

- `PLAN_VALIDATION_IOS_GEOMETRY.md`
- `.github/workflows/ci.yml`
- concise G15 evidence in `PLAN_VALIDATION.md` and `docs/VALIDATION.md`

Do not edit the D16/B21 APIs, backend, probe, shared Cargo manifests/lockfile, capability counts,
or other validation gates. Coordinate probe paths and accepted symbols/layout with the B21 owner

## Required gates

- Reuse CI's existing installation of `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run locked device and Simulator checks and strict all-target Clippy for `ios-ui` on both targets
- Review the private Rust→CoreGraphics `extern "C"` declarations against the Apple signatures:
  `CGRectIntersection(CGRect, CGRect) -> CGRect`, `CGRectIsNull(CGRect) -> bool`, and
  `CGRectIsEmpty(CGRect) -> bool`; there is no public C header/API
- Run `sh platform/ios/ios-ui/check-link-imports.sh`. It must build but not execute the device and
  Simulator probes, compile `platform/ios/ios-ui/probes/geometry-layout.c` for both SDK targets,
  require CoreGraphics.framework and libSystem.B.dylib, and verify `_CGRectIntersection`,
  `_CGRectIsNull`, and `_CGRectIsEmpty`
- Verify the C and Rust probe layout assertions for `CGFloat`, `CGPoint`, `CGSize`, and `CGRect`:
  arm64 CGFloat is 8 bytes; CGPoint/CGSize are 16 bytes with 8-byte alignment; CGRect is 32 bytes
  with 8-byte alignment and origin/size offsets 0/16. The C fixture also checks the C `bool`
  size. Inspect exact direct imports and allow additional frameworks only when the linked probe
  has a concrete reference to them; reject unrelated imports, Swift, and Python
- Reject Swift source and unrelated Objective-C/UIKit imports; retain host docs, zero-Swift,
  workflow-YAML, and diff checks
- Do not claim native visual behavior, runtime parity, device execution, or general geometry
  support

## G15 local results (2026-10-08)

The following commands passed with Rust 1.94.1:

```sh
cargo +1.94.1 check --locked -p ios-ui --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-ui --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked -p ios-ui --all-targets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked -p ios-ui --all-targets --target aarch64-apple-ios-sim -- -D warnings
sh platform/ios/ios-ui/check-link-imports.sh
```

Xcode 26.6 (build 17F113) and SDK 26.5 built arm64 Mach-O probes and C layout objects for both
targets. The exact probe outputs are `target/ios-ui-geometry-link-aarch64-apple-ios/aarch64-apple-ios/release/examples/ios_ui_geometry_link_import_probe` and `target/ios-ui-geometry-link-aarch64-apple-ios-sim/aarch64-apple-ios-sim/release/examples/ios_ui_geometry_link_import_probe`. `vtool -show-build` reports device iOS minimum 12.0 and Simulator minimum 14.0, both SDK 26.5. Each probe's exact direct import set is `CoreGraphics.framework` and `/usr/lib/libSystem.B.dylib`. `dyld_info -imports` maps `_CGRectIntersection`, `_CGRectIsNull`, and `_CGRectIsEmpty` to CoreGraphics; the remaining C/runtime imports, including allocation, pthread, dispatch, and unwind symbols, map to libSystem. The link uses `-Wl,-dead_strip_dylibs` to remove otherwise autolinked UIKit, Foundation, CoreFoundation, and libobjc load commands that have no symbol references from this geometry-only probe. No Swift/Python runtime imports or unresolved Rust symbols remain

The first standalone dead-strip diagnostic was attempted before central Cargo.lock reconciliation and failed before compilation because `--locked` detected that the lockfile needed an update. After root completed lock reconciliation, the device and Simulator dead-strip builds and the checked-in script passed with locked dependency resolution; the Cargo.lock was not changed by this workstream

The C fixture compiled for each target and its `_Static_assert`s verified `bool` size 1, `CGFloat` size/alignment 8/8, `CGPoint` and `CGSize` size/alignment 16/8 with field offsets 0/8, and `CGRect` size/alignment 32/8 with origin/size offsets 0/16. The Rust probe has matching compile-time layout assertions and references both overlapping and disjoint API paths. The private Rust declarations match Xcode's `CGGeometry.h`: `CGRectIntersection(CGRect, CGRect) -> CGRect`, `CGRectIsNull(CGRect) -> bool`, and `CGRectIsEmpty(CGRect) -> bool`; the declarations are marked iOS-available since 2.0 by the SDK

These are compile, link, import, and layout checks only. Neither artifact was executed, so they do not establish CoreGraphics runtime parity, device/simulator behavior, visual behavior, or general geometry support. No passing CI workflow run is recorded

## Validation and handoff

- Keep the target check, strict Clippy, and link/import/layout gates in the macOS workflow
- Record exact commands, Xcode/SDK versions, direct imports, symbol/layout evidence, and any first
  attempt failures and their resolution
- State that compile/link/layout evidence does not establish runtime geometry parity or visual
  behavior

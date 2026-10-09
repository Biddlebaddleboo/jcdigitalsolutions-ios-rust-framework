# PLAN_VALIDATION_IOS_SPRITEKIT.md — Workstream G64: SpriteKit Node Position Gate

## Gates

- `framework-spritekit`: format check, host no-default-features check, strict Clippy, rustdoc; no tests
- `ios-spritekit`: format check, host check, device and Simulator checks, strict Clippy for both Apple targets, and rustdoc
- Link-only consumer on device and Simulator: check exact required framework imports and target minimum OS; never run the probe
- `sh -n platform/ios/ios-spritekit/check.sh platform/ios/ios-spritekit/check-link-imports.sh`
- `git diff --check`

These are compile, lint, documentation, and link-metadata checks only. They do not prove a live SpriteKit runtime, scene attachment, drawing, animation, or performance.

## Validation record

On Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, the package gate and link/import check pass in `/private/tmp/jc-ios-spritekit-worktree` and the integrated checkout. Device and Simulator probes report `minos` 12.0 and 14.0 respectively; 12.0 is the active SDK's minimum deployment, and 14.0 is the repository's Simulator validation baseline. SpriteKit's API floor for the unannotated `SKNode` members is iOS 7.0. The probes were built and inspected, not executed; no tests were run.

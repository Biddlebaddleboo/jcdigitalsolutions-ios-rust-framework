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

## Hosted Xcode 27 evidence

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check iOS SpriteKit node-position package` ran `sh platform/ios/ios-spritekit/check.sh`, and `Audit iOS SpriteKit node-position link imports` ran `sh platform/ios/ios-spritekit/check-link-imports.sh`; both passed
- Both gates also passed in `Rust checks (macos-15)`, which reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped these Apple-only gates
- No diff from the run SHA to current main `f7a6d0cd98d9a23ecd718fbc98751f46c048a204` in `platform/ios/ios-spritekit`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers package compile, lint, documentation, and link-metadata gates only; probes were not executed, and no scene, drawing, animation, live runtime, or performance result is claimed

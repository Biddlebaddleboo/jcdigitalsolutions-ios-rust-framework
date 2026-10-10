# PLAN_VALIDATION_IOS_COREML.md — G50: Core ML Compute-Device Status

## Objective

Gate D51/B56's iOS-only query of whether Core ML reports a nonempty compute-device list.

## Required checks

- Run `sh platform/ios/ios-core-ml-status/scripts/check.sh` on macOS.
- Run host, device, and Simulator package checks, strict Clippy, host rustdoc, and device rustdoc.
- Build, but do not execute, device and Simulator Release probes; require exact CoreML/Foundation/
  libSystem/libobjc imports, the Objective-C query symbols, and no Swift runtime.
- Run formatting, zero-Swift-source, and whitespace checks. Do not add or run tests, load a model,
  execute a probe, or run inference.

## Evidence boundary

These gates establish compile/link shape only. They do not show a live hardware list, model
compatibility, inference success, or runtime performance. The prior local inspection used Xcode
26.6 / SDK 26.5; see hosted evidence below for Xcode 27.x requalification.

## Hosted Xcode 27 qualification

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check Core ML compute-device status` ran `sh platform/ios/ios-core-ml-status/scripts/check.sh` and passed
- `Rust checks (macos-15)` also passed this package gate, but that runner reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped the Apple-only gate
- `Optional iOS Core ML status C ABI check` and `Optional iOS Core ML status C ABI link/import check` passed in the Xcode 27 job; logs identify static/build and link-import checks only, with consumers/probes not executed
- No diff from the run SHA to current main `cb807e43b2693f950293599f729d4445e8f827d6` in `platform/ios/ios-core-ml-status`, `bindings/c`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers static, compile, and link gates only; no model load, live compute-device list, inference, compatibility, or performance result

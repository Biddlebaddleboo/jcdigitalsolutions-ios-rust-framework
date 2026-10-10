# PLAN_VALIDATION_IOS_VISION.md — G51: Vision Text-Revision Support

## Objective

Gate D52/B57's portable revision-membership value and bounded `VNRecognizeTextRequest` class query.

## Required checks

- Run `sh platform/ios/ios-vision/check.sh` on macOS.
- Check `framework-vision` without default features, strict Clippy, and rustdoc; check `ios-vision`
  on host, device, and Simulator with strict Clippy and rustdoc.
- Verify only `VNRequest`, `VNRecognizeTextRequest`, and `NSIndexSet` features are enabled.
- Build, but do not execute, device and Simulator Release probes; inspect exact Vision/Foundation/
  libobjc/libSystem imports, request class/selector, no forbidden image/request-handler/UI/Swift
  imports, and device/Simulator minimum OS values.
- Run formatting, docs freshness, zero-Swift-source, and diff checks. Do not add or run tests or
  execute an image-recognition request.

## Evidence boundary

Compile/link checks do not establish a live supported-revision set, image recognition, accuracy,
model readiness, or runtime behavior. The query creates no request and reads no image. The prior
local inspection used Xcode 26.6 / SDK 26.5; see hosted evidence below for Xcode 27.x requalification.

## Hosted Xcode 27 qualification

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check Vision text-recognition revision status` ran `sh platform/ios/ios-vision/check.sh` and passed
- `Rust checks (macos-15)` also passed this package gate, but that runner reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped the Apple-only gate
- `Optional iOS Vision revision C ABI check` passed in the Xcode 27 job; its log reports device and Simulator imports/minimum OS checks and states the probe was not executed
- No diff from the run SHA to current main `cb807e43b2693f950293599f729d4445e8f827d6` in `platform/ios/ios-vision`, `bindings/c`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers static, compile, and link checks only; no image request, recognition, accuracy, model readiness, or runtime behavior

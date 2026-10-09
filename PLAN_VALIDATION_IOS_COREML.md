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
compatibility, inference success, or runtime performance. The inspected Xcode 26.6 / SDK 26.5 host
is below the planned Xcode 27.x baseline.

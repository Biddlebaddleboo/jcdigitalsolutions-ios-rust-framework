# PLAN_VALIDATION_IOS_ARKIT.md — G48: ARKit World-Tracking Support

## Objective

Gate D49/B54's portable support value and `ARWorldTrackingConfiguration.isSupported` class query.

## Required checks

- Run `sh platform/ios/ios-maps/check.sh` on macOS.
- Check `framework-maps` no-default-features, strict Clippy, and rustdoc; check `ios-maps` host,
  device, and Simulator targets with strict Clippy and rustdoc.
- Verify only the `ARConfiguration` and `objc2` generated binding features are enabled.
- Build, but do not execute, the device and Simulator link probes; inspect exact ARKit/Foundation/libSystem/
  libobjc imports, required selector, absence of session/camera/UI/Swift symbols, and deployment
  minimums.
- Run formatter, documentation, zero-Swift-source, and diff checks. Do not add or run tests.

## Evidence boundary

Compile and link checks do not establish a live hardware support value, camera authorization, a
working AR session, or tracking quality. The support query does not create a session, access the
camera, or request permission. The inspected Xcode 26.6 / SDK 26.5 host is below the planned Xcode
27.x baseline.

## Hosted Xcode 27 static-gate evidence (2026-10-10)

The Xcode 27 lane in [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed step `Check iOS ARKit support and MapKit geometry`, which ran `sh platform/ios/ios-maps/check.sh` at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The runner recorded Xcode 27.0 build 27A266a, iPhoneOS and iPhoneSimulator SDK 27.0, and Rust 1.94.1. The script's feature-selection checks, format, portable and iOS host/device/Simulator checks and strict Clippy, rustdoc, and device/Simulator link/import checks passed. The lane's `Format` and `Documentation and zero-Swift-source checks` steps also passed. The `crates/framework-maps/`, `platform/ios/ios-maps/`, workspace manifest/lockfile, `xtask/`, and `.github/workflows/ci.yml` gate inputs are unchanged from the run SHA through current `origin/main` `5b59d9d`.

This is compile, lint, documentation, feature-selection, and link/import evidence only. The device and Simulator probes were built and inspected, not executed. It does not establish a live hardware support value, camera authorization, a working AR session, or tracking quality.

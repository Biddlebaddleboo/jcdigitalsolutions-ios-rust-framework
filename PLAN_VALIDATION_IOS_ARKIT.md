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

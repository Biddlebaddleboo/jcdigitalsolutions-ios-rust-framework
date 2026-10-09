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
model readiness, or runtime behavior. The query creates no request and reads no image. The inspected
Xcode 26.6 / SDK 26.5 host is below the planned Xcode 27.x baseline.

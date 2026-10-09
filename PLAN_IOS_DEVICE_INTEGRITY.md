# PLAN_IOS_DEVICE_INTEGRITY.md — Workstream B42: DeviceCheck and App Attest Support Query

## Objective

Implement D37's support snapshot through the public DeviceCheck `isSupported` properties only.

## Bounded API

- `DCDevice.currentDevice().isSupported` is queried only on iOS 11.0+
- `DCAppAttestService.sharedService().isSupported` is queried only on iOS 14.0+
- Return `false` before each API's OS floor; on iOS 11–13, report DeviceCheck support and App Attest unsupported
- Use `objc2-device-check` 0.3.2 with defaults disabled and only `DCDevice` / `DCAppAttestService` features
- Keep the direct `DeviceCheck.framework` API floor at iOS 11.0
- Do not generate tokens/keys, attest/assert, prompt, make network calls, or claim entitlement, enrollment, identity, integrity, or future-operation success
- Record Apple's app-extension caveat for App Attest support

## Validation

Run `platform/ios/ios-device-integrity/check.sh` for portable, device, Simulator, lint, rustdoc, feature, link/import, docs, and zero-Swift-source evidence. Compile and link evidence does not establish a live support result.

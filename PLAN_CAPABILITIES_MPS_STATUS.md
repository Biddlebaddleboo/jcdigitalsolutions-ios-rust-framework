# PLAN_CAPABILITIES_MPS_STATUS.md — Workstream D62: MPS Preferred-Device Status

## Status

D62 adds no portable contract. It covers only whether Apple's default-options MPS preferred-device
query returns a device on iOS.

## Contract boundary

- Expose `ios_mps_status::preferred_mps_device_available() -> bool` on iOS.
- Call `MPSGetPreferredDevice(MPSDeviceOptions::Default)` and map non-null to `true`.
- Drop the retained device before return; expose no native device, identifier, or handle.
- Do not submit GPU work or claim support for any MPS operation, model, or workload.
- Do not make performance, initialization-cost, or thread-affinity claims.
- Add no permission, Info.plist, entitlement, or portable facade requirement.

## Availability and evidence

- `MPSGetPreferredDevice` and the default option are available from iOS 12.2 in the inspected
  iPhoneOS 26.5 SDK.
- The wrapper uses `objc2-metal-performance-shaders` 0.3.2 with default features disabled and only
  `MPSCore` enabled.
- See [the iOS package plan](PLAN_IOS_MPS_STATUS.md), [validation plan](PLAN_VALIDATION_IOS_MPS_STATUS.md),
  and [developer guide](docs/ios/mps-status.md).

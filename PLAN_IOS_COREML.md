# PLAN_IOS_COREML.md — B56: Core ML Compute Device Status

## Objective

Add a narrow iOS Rust query for Core ML compute-device list availability under row 063

## Scope

- `platform/ios/ios-core-ml-status/**`
- `PLAN_CAPABILITIES_COREML.md`
- No portable contract, shared capability manifest, shared docs, CI, or root aggregate index changes

## API requirements

- Export only `has_available_compute_device() -> bool`
- Guard the generated `MLModel::availableComputeDevices` call with `objc2::available!(ios = 17.0, ..)` and return `false` below iOS 17.0
- Enable only `MLModel`, `MLModel_MLComputeDevice`, and `MLComputeDeviceProtocol` in `objc2-core-ml`
- Do not load a model, run inference, read user input, or expose compute-device objects
- Do not add a main-thread marker, prompt, entitlement, or privacy usage key
- Describe a nonempty array only as a Core ML availability snapshot, not a promise that a model can predict

## Validation and limits

- Run `sh platform/ios/ios-core-ml-status/scripts/check.sh`
- The link script builds device and simulator probes and inspects imports/symbols; it never executes a probe
- Do not add or run tests or invoke Core ML prediction
- Report API floor, exact imports, checks, and runtime limits

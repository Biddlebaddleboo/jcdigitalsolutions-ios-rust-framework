# PLAN_CAPABILITIES_COREML.md — D51: Core ML Device Status

## Objective

Expose one point-in-time query for whether Core ML lists any compute device available for prediction on this iOS device. No portable contract is useful for this hardware- and OS-specific result

## Public boundary

- `ios_core_ml_status::has_available_compute_device() -> bool` calls only `MLModel::availableComputeDevices`
- Return `false` below the iOS 17.0 API floor
- Return `true` only when the Core ML device array is nonempty at the time of the call
- Do not load a model, create a model configuration, run inference, accept user input, or return hardware objects
- Do not claim that any specific model or operation can run

## API and evidence

The local iPhoneOS26.5 SDK declares `MLModel.availableComputeDevices` on the `MLModel (MLComputeDevice)` category with an iOS 17.0 floor. Apple defines it as the list of compute devices that Core ML predictions may use. The generated `objc2-core-ml` 0.3.2 method is `unsafe fn availableComputeDevices() -> Retained<NSArray<ProtocolObject<dyn MLComputeDeviceProtocol>>>`, gated by `MLModel`, `MLModel_MLComputeDevice`, and `MLComputeDeviceProtocol`

The wrapper reads only `NSArray::count` and drops the retained array. It does not inspect model files, feature values, inference output, or personal data

## Validation and limits

- Run host, device, and simulator package check, strict Clippy, and rustdoc gates
- Build but do not execute the device and simulator link probes; verify the exact CoreML import allowlist, Objective-C symbols, and absence of Swift runtime symbols
- Do not add or run tests, execute probes, load a model, or run inference
- These checks show compile and link shape only; they do not prove hardware availability, model compatibility, prediction success, or runtime performance

## D51 audit — 2026-10-10

- Base: `c14817086faaeae93767b797f600bc4f8fa09213`
- The existing `ios_core_ml_status::has_available_compute_device()` meets the public boundary; no product-source change was needed. It returns `false` below iOS 17.0, calls only `MLModel::availableComputeDevices`, and returns whether its array count is nonzero
- The pinned `ios-rust-build` and `ios-rust-validate` tools installed as version `0.1.0` for `x86_64-apple-darwin`, source SHA `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`
- `ios-rust-validate --explain ios-core-ml-status` returned `unknown capability`; this capability has no registered validator pilot
- `sh platform/ios/ios-core-ml-status/scripts/check.sh` passed with exit code 0. It covered host formatting, check, strict Clippy and rustdoc; iOS device and simulator checks/Clippy; iOS rustdoc; release device and simulator probe builds with import/symbol/string inspection; the zero-Swift-source check; and `git diff --check`
- No tests were run and neither probe binary was executed. The gate does not establish a live device list, model compatibility, prediction success, or performance

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

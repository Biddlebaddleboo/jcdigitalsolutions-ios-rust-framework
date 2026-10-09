# Core ML compute-device status

`ios_core_ml_status::has_available_compute_device()` returns whether `MLModel.availableComputeDevices` contains at least one device at the time of the call. It returns `false` below iOS 17.0

This is a coarse availability snapshot only. It does not load a model, read input values, run inference, name a compute device, or guarantee that a particular model or operation can run. The getter returns native compute-device values internally; this crate exposes only the Boolean result

The inspected iPhoneOS26.5 SDK marks `MLModel (MLComputeDevice)` and `availableComputeDevices` as available from iOS 17.0. The generated binding is `objc2-core-ml` 0.3.2 with features `MLModel`, `MLModel_MLComputeDevice`, and `MLComputeDeviceProtocol`

The package gate checks device and simulator compile/lint and inspects linked imports. It does not run the probe binary, load a model, perform inference, verify a live device list, or measure performance

See [Apple's `availableComputeDevices` reference](https://developer.apple.com/documentation/coreml/mlmodel/availablecomputedevices-42uzt) and [`MLModel`](https://developer.apple.com/documentation/coreml/mlmodel)

# iOS Core ML compute-device C API

The opt-in `ios-core-ml-status` feature exposes B56's one-byte snapshot of whether `MLModel.availableComputeDevices` is nonempty. The API floor is iOS 17.0. On iOS below that floor, B56 returns false and the C function returns `FRAMEWORK_STATUS_OK` with output zero

## Call contract

`framework_ios_core_ml_status_has_available_compute_device` takes one required writable output byte. The API checks only nullness; any non-null pointer must actually address valid, properly aligned writable memory for the synchronous call, and the caller must prevent unsynchronized concurrent access to that byte. The wrapper initializes it to zero before platform handling. On iOS, `FRAMEWORK_STATUS_OK` writes exactly `0` or `1` from the Rust backend. A valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` with output zero. A null pointer returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A caught Rust panic returns `FRAMEWORK_STATUS_PANIC` with zero output. No pointer or Core ML object is retained or exposed

A true value means only that Core ML reported at least one compute device for prediction at the instant of the query. It does not establish that a specific model or operation can run. This API does not load a model, create a configuration, run inference, accept user data, or expose device objects. Calls are synchronous on the caller's thread; no queue guarantee is added

The `MLModel.availableComputeDevices` API floor is iOS 17.0. The F27 link gate uses device minos 11.0, the `MLModel`/CoreML framework floor, and Simulator minos 14.0. B56's runtime guard returns false before the iOS 17.0 category call. These link settings are not the getter's API floor

## Validation limits

The static/build gate checks the feature graph, Rust format/check/Clippy/rustdoc, and C11/C++17 header syntax. The separate link-import gate is available for root validation but was not run in this F27 workstream. No tests, consumers, or probes were run here. Compile/import evidence cannot prove that a model or operation can predict, device availability at runtime, output parity, or performance

See [F27 binding plan](../../PLAN_BINDINGS_F27.md), [B56 backend plan](../../PLAN_IOS_COREML.md), and [D51 contract](../../PLAN_CAPABILITIES_COREML.md)

#ifndef FRAMEWORK_IOS_CORE_ML_STATUS_H
#define FRAMEWORK_IOS_CORE_ML_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Opt in through framework-c-api's Cargo feature `ios-core-ml-status`. This exposes only B56's
 * `MLModel.availableComputeDevices` nonempty-list snapshot. Its API floor is iOS 17.0; the F27
 * link probes use minos 11.0 for device and 14.0 for Simulator because CoreML's `MLModel` class
 * begins at iOS 11.0 and the runtime guard protects the iOS 17.0 category getter. These are link
 * settings, not the getter's API floor.
 *
 * `out_available` is required and writable for one byte. The API checks only nullness: any
 * non-null pointer must actually address valid, properly aligned writable memory for the
 * synchronous call. The caller must prevent unsynchronized concurrent access to that byte. The
 * output is initialized to zero before platform handling and the pointer is not retained. On iOS
 * the function returns FRAMEWORK_STATUS_OK and writes exactly
 * zero or one according to B56's Boolean result. B56 returns false below iOS 17.0, so this API
 * returns OK with zero there. A valid non-iOS call returns FRAMEWORK_STATUS_UNSUPPORTED with zero
 * output. A null pointer returns FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. A caught Rust
 * panic returns FRAMEWORK_STATUS_PANIC with zero output.
 *
 * A true value means only that Core ML reported at least one compute device for prediction at this
 * instant. It does not prove that a model or operation can run. This API does not load a model,
 * create a model configuration, run inference, accept input, or expose device objects. Calls are
 * synchronous on the caller's thread; no queue guarantee is added.
 */
FrameworkStatus framework_ios_core_ml_status_has_available_compute_device(uint8_t *out_available);

#ifdef __cplusplus
}
#endif

#endif

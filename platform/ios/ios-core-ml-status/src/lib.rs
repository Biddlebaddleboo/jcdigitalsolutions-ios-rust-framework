#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS Core ML compute-device availability query"]

use objc2_core_ml::MLModel;

/// Return whether Core ML reports at least one compute device available for prediction
///
/// This reads only the device list from `MLModel.availableComputeDevices`. It does not load a
/// model, run inference, read user input, or promise that a specific model or operation can run
///
/// This returns `false` below iOS 17.0. A `true` result means only that Core ML's device list is
/// nonempty at the time of this call
pub fn has_available_compute_device() -> bool {
    if !objc2::available!(ios = 17.0, ..) {
        return false;
    }

    // SAFETY: this branch enforces the iOS 17.0 category API floor; the class getter has no inputs
    let devices = unsafe { MLModel::availableComputeDevices() };
    devices.count() != 0
}

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Non-prompting iOS RoomPlan device-support query."]

use framework_roomplan::RoomPlanDeviceSupport;

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_roomplan_is_supported() -> u8;
}

/// Queries whether the current iOS device supports RoomPlan room scanning.
///
/// On iOS, this calls only `RoomCaptureSession.isSupported`, available from iOS 16.0. The backend
/// does not create or run a capture session, access camera frames, request permission, present UI,
/// or establish that a scan can succeed. Non-iOS targets return `None`.
pub fn device_support() -> Option<RoomPlanDeviceSupport> {
    #[cfg(target_os = "ios")]
    {
        // The C shim is compiled only for supported 64-bit Apple iOS targets and uses the exact
        // Swift ABI lowered by the package's compiler-oracle gate.
        let is_supported = unsafe { framework_roomplan_is_supported() != 0 };
        Some(RoomPlanDeviceSupport::from_platform_query(is_supported))
    }

    #[cfg(not(target_os = "ios"))]
    {
        None
    }
}

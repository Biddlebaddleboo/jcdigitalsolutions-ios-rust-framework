#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable status value for RoomPlan device support."]

/// The result of a platform query for RoomPlan device support.
///
/// On iOS, this reflects `RoomCaptureSession.isSupported`. A positive result means the device
/// supports RoomPlan's room-scanning framework; it does not mean that a scan has started or will
/// succeed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RoomPlanDeviceSupport {
    is_supported: bool,
}

impl RoomPlanDeviceSupport {
    /// Creates a support result from a platform's status query.
    pub const fn from_platform_query(is_supported: bool) -> Self {
        Self { is_supported }
    }

    /// Returns whether the platform reports RoomPlan device support.
    pub const fn is_supported(self) -> bool {
        self.is_supported
    }
}

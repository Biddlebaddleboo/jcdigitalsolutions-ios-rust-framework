use framework_metal::{MetalDevicePresence, MetalDevicePresenceBackend};
use objc2_metal::MTLCreateSystemDefaultDevice;

/// Stateless iOS backend for the system-default Metal device-presence query.
///
/// Construction is inert. A snapshot calls `MTLCreateSystemDefaultDevice`, checks whether the
/// returned retained device object is non-null, and then releases that object without exposing it.
#[derive(Clone, Copy, Debug, Default)]
pub struct CoreMetalDeviceBackend;

impl MetalDevicePresenceBackend for CoreMetalDeviceBackend {
    fn snapshot(&self) -> MetalDevicePresence {
        let device = MTLCreateSystemDefaultDevice();
        let present = device.is_some();
        drop(device);
        if present {
            MetalDevicePresence::Present
        } else {
            MetalDevicePresence::Absent
        }
    }
}

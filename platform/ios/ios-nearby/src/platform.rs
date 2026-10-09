use framework_nearby::{NearbyInteractionCapabilityBackend, NearbyInteractionCapabilitySnapshot};
use objc2_nearby_interaction::{NIDeviceCapability, NISession};

/// A stateless iOS backend for a non-prompting precise-distance capability query.
///
/// The host app must set its iOS deployment target to 16.0 or later before it calls this backend.
/// A `true` result is not permission, peer, session, or ranging success.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IosNearbyInteractionBackend;

impl IosNearbyInteractionBackend {
    /// Creates the stateless backend without making a native call.
    pub const fn new() -> Self {
        Self
    }
}

impl NearbyInteractionCapabilityBackend for IosNearbyInteractionBackend {
    fn snapshot(&self) -> NearbyInteractionCapabilitySnapshot {
        NearbyInteractionCapabilitySnapshot::new(read_precise_distance_capability())
    }
}

fn read_precise_distance_capability() -> bool {
    // SAFETY: The host contract requires iOS 16 or later, where Apple's SDK declares
    // `NISession.deviceCapabilities` and `NIDeviceCapability`. The generated binding has the exact
    // public class-property selector and returns an owned `Retained<ProtocolObject<_>>`; it uses
    // no caller pointer, callback, or borrowed storage. The retained protocol object keeps its
    // Objective-C lifetime valid for the following scalar getter call.
    let capabilities = unsafe { NISession::deviceCapabilities() };

    // SAFETY: The public generated getter is a no-argument BOOL property accessor invoked on the
    // retained protocol object returned above. It has no pointer, ownership-transfer, or callback
    // precondition, and the deployment-target contract ensures the getter exists on this process.
    unsafe { capabilities.supportsPreciseDistanceMeasurement() }
}

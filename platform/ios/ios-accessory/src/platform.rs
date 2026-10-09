use framework_accessory::{AccessoryPresenceSnapshot, ExternalAccessoryBackend};
use objc2_external_accessory::EAAccessoryManager;

/// Reads whether the current iOS accessory-manager list has any available entries.
///
/// The value is a transient snapshot, not a connection guarantee.
pub struct IosExternalAccessoryBackend;

impl ExternalAccessoryBackend for IosExternalAccessoryBackend {
    fn connected_accessory_presence() -> AccessoryPresenceSnapshot {
        // SAFETY: Apple's documented shared-manager method returns the framework singleton; this
        // call passes no pointers or caller-controlled values.
        let manager = unsafe { EAAccessoryManager::sharedAccessoryManager() };
        // SAFETY: the receiver is the retained manager singleton and the documented getter has no
        // arguments; the generated binding returns a retained array.
        let accessories = unsafe { manager.connectedAccessories() };
        if accessories.count() == 0 {
            AccessoryPresenceSnapshot::NoneAvailable
        } else {
            AccessoryPresenceSnapshot::OneOrMoreAvailable
        }
    }
}

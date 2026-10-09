#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable scalar contract for a current external-accessory presence snapshot."]

/// A scalar snapshot of entries in the platform's connected-and-available accessory list.
///
/// `NoneAvailable` means the list was empty when queried. It does not prove that no accessory is
/// physically attached, that the platform lacks accessory support, or that a particular app can
/// communicate with an accessory. Query again to observe a later list state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AccessoryPresenceSnapshot {
    /// The platform's current connected-and-available accessory list is empty.
    NoneAvailable,
    /// The platform's current connected-and-available accessory list has at least one entry.
    OneOrMoreAvailable,
}

/// A compile-time-selected backend for an external-accessory presence snapshot.
///
/// This contract exposes no accessory identity, protocol, session, or stream and starts no
/// monitoring or communication.
pub trait ExternalAccessoryBackend {
    /// Returns whether the current platform list has at least one available accessory.
    fn connected_accessory_presence() -> AccessoryPresenceSnapshot;
}

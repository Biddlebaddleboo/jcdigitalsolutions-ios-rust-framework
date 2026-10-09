#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable no_std snapshots for the presence of a system-default Metal device."]

/// One point-in-time report about whether a system-default Metal device object exists.
///
/// `Present` means the selected backend reported that its system-default Metal device factory
/// returned a device object. It does not establish that a GPU workload can run, that a feature is
/// supported, or that any performance target can be met. This contract exposes no GPU device,
/// command queue, buffer, shader, or MetalKit view.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum MetalDevicePresence {
    /// The backend cannot classify whether a system-default Metal device exists.
    Unknown = 0,
    /// The backend reports that a system-default Metal device object exists.
    Present = 1,
    /// The backend reports that no system-default Metal device object is available.
    Absent = 2,
}

/// A statically selected backend that returns a system-default Metal device presence snapshot.
pub trait MetalDevicePresenceBackend {
    /// Returns the backend's current point-in-time presence report.
    fn snapshot(&self) -> MetalDevicePresence;
}

/// A thin facade over caller-owned, statically selected Metal-device query state.
pub struct MetalDeviceQuery<B> {
    backend: B,
}

impl<B: MetalDevicePresenceBackend> MetalDeviceQuery<B> {
    /// Creates the facade around an explicitly supplied backend without native work.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Returns one point-in-time system-default Metal device presence snapshot.
    pub fn snapshot(&self) -> MetalDevicePresence {
        self.backend.snapshot()
    }

    /// Borrows the selected backend for backend-specific controls.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Returns the selected backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeBackend(MetalDevicePresence);

    impl MetalDevicePresenceBackend for FakeBackend {
        fn snapshot(&self) -> MetalDevicePresence {
            self.0
        }
    }

    #[test]
    fn facade_preserves_snapshot_and_fixed_width_semantics() {
        assert_eq!(core::mem::size_of::<MetalDevicePresence>(), 1);
        for expected in [
            MetalDevicePresence::Unknown,
            MetalDevicePresence::Present,
            MetalDevicePresence::Absent,
        ] {
            let query = MetalDeviceQuery::new(FakeBackend(expected));
            assert_eq!(query.snapshot(), expected);
            assert_eq!(query.backend().snapshot(), expected);
            assert_eq!(query.into_backend().snapshot(), expected);
        }
    }
}

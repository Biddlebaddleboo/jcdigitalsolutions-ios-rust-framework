#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable status values and a backend contract for a bounded Nearby Interaction capability snapshot."]

/// A snapshot of one documented Nearby Interaction device capability.
///
/// This value records only whether the selected backend reports support for precise distance
/// measurement. It does not report permission, peer compatibility, session readiness, or whether
/// a Nearby Interaction operation will succeed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NearbyInteractionCapabilitySnapshot {
    supports_precise_distance_measurement: bool,
}

impl NearbyInteractionCapabilitySnapshot {
    /// Creates a snapshot from the backend-reported precise-distance capability.
    pub const fn new(supports_precise_distance_measurement: bool) -> Self {
        Self {
            supports_precise_distance_measurement,
        }
    }

    /// Returns the reported precise-distance capability.
    pub const fn supports_precise_distance_measurement(self) -> bool {
        self.supports_precise_distance_measurement
    }
}

/// A statically selected backend for a non-prompting Nearby Interaction capability snapshot.
///
/// The backend reports only the precise-distance capability. It must not create or run a session,
/// request permission, exchange discovery tokens, or start ranging as part of this query.
pub trait NearbyInteractionCapabilityBackend {
    /// Reads the precise-distance capability at call time.
    fn snapshot(&self) -> NearbyInteractionCapabilitySnapshot;
}

#[cfg(test)]
mod tests {
    use super::{NearbyInteractionCapabilityBackend, NearbyInteractionCapabilitySnapshot};

    struct FixedBackend(bool);

    impl NearbyInteractionCapabilityBackend for FixedBackend {
        fn snapshot(&self) -> NearbyInteractionCapabilitySnapshot {
            NearbyInteractionCapabilitySnapshot::new(self.0)
        }
    }

    #[test]
    fn snapshot_preserves_true_and_false_values() {
        assert!(
            NearbyInteractionCapabilitySnapshot::new(true).supports_precise_distance_measurement()
        );
        assert!(
            !NearbyInteractionCapabilitySnapshot::new(false)
                .supports_precise_distance_measurement()
        );
    }

    #[test]
    fn backend_returns_its_snapshot_value() {
        assert!(
            FixedBackend(true)
                .snapshot()
                .supports_precise_distance_measurement()
        );
        assert!(
            !FixedBackend(false)
                .snapshot()
                .supports_precise_distance_measurement()
        );
    }
}

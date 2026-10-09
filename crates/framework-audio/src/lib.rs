#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable values for bounded playback capability snapshots."]

/// Whether the system currently reports that this device can present HDR content to an
/// appropriate HDR display.
///
/// This value does not describe any particular asset, current item, active playback, or whether
/// the display is presently showing HDR. It is a snapshot and may change when display or system
/// resources change.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HdrPlaybackEligibility(bool);

impl HdrPlaybackEligibility {
    /// Creates an HDR eligibility snapshot from the system-reported value.
    pub const fn new(eligible: bool) -> Self {
        Self(eligible)
    }

    /// Returns whether the system reports HDR playback eligibility.
    pub const fn is_eligible(self) -> bool {
        self.0
    }
}

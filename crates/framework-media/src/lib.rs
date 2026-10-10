#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable finite media-time values and bounded audio and video capability snapshots."]

use core::cmp::Ordering;
use core::num::NonZeroI32;

/// One finite rational media time with an epoch of zero.
///
/// The value is `value / timescale` seconds. The pair stays as supplied; comparison uses exact
/// integer arithmetic, not a float. This type has no invalid, infinite, indefinite, or
/// rounded state
#[derive(Clone, Copy, Debug)]
pub struct MediaTime {
    value: i64,
    timescale: NonZeroI32,
}

impl MediaTime {
    /// Creates a finite media time with a strictly positive timescale.
    pub const fn new(value: i64, timescale: i32) -> Result<Self, MediaTimeError> {
        match NonZeroI32::new(timescale) {
            Some(scale) if timescale > 0 => Ok(Self {
                value,
                timescale: scale,
            }),
            _ => Err(MediaTimeError::InvalidTimescale),
        }
    }

    /// Returns the signed rational numerator.
    pub const fn value(self) -> i64 {
        self.value
    }

    /// Returns the strictly positive rational denominator.
    pub const fn timescale(self) -> i32 {
        self.timescale.get()
    }

    /// Compares the represented rational values without float conversion or approximation.
    pub const fn compare(self, other: Self) -> Ordering {
        let left = (self.value as i128) * (other.timescale.get() as i128);
        let right = (other.value as i128) * (self.timescale.get() as i128);
        if left < right {
            Ordering::Less
        } else if left > right {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for MediaTime {
    fn eq(&self, other: &Self) -> bool {
        self.compare(*other) == Ordering::Equal
    }
}

impl Eq for MediaTime {}

impl PartialOrd for MediaTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MediaTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(*other)
    }
}

/// A point-in-time report of whether the legacy ReplayKit recorder is available.
///
/// This value mirrors `RPScreenRecorder.isAvailable`; it is not permission, consent, a recording
/// session, or a guarantee that a future recording request will succeed. Availability can change
/// when hardware, AirPlay or TV Out, or another app's recorder use changes. This type covers only
/// the legacy ReplayKit recorder and does not report ScreenCaptureKit availability.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReplayKitAvailability {
    available_for_recording: bool,
}

impl ReplayKitAvailability {
    /// Creates a snapshot from the platform's point-in-time ReplayKit availability value.
    pub const fn new(available_for_recording: bool) -> Self {
        Self {
            available_for_recording,
        }
    }

    /// Returns the platform's point-in-time report that ReplayKit is available for recording.
    pub const fn is_available_for_recording(self) -> bool {
        self.available_for_recording
    }
}

/// A point-in-time report that the built-in SoundAnalysis classifier request is recognized.
///
/// A `true` value means the built-in version 1 classifier request was constructed successfully
/// and exposed a nonempty known-classification list. This value does not imply microphone access,
/// model execution, analysis success, custom-model support, or ShazamKit catalog access.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SoundAnalysisSupportSnapshot {
    built_in_classifier_available: bool,
}

impl SoundAnalysisSupportSnapshot {
    /// Creates a snapshot from the platform's built-in classifier support probe.
    pub const fn new(built_in_classifier_available: bool) -> Self {
        Self {
            built_in_classifier_available,
        }
    }

    /// Returns whether the platform recognized the built-in sound classifier request.
    pub const fn is_supported(self) -> bool {
        self.built_in_classifier_available
    }
}

/// A point-in-time report of whether any other app is playing audio.
///
/// This snapshot does not identify the source, report this app's playback, expose Now Playing
/// metadata, or control media. It can include audio from another app using an ambient category.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OtherAudioPlaybackSnapshot {
    other_audio_playing: bool,
}

impl OtherAudioPlaybackSnapshot {
    /// Creates a snapshot from the platform's point-in-time other-audio report.
    pub const fn new(other_audio_playing: bool) -> Self {
        Self {
            other_audio_playing,
        }
    }

    /// Returns whether another app was playing any audio at query time.
    pub const fn is_other_audio_playing(self) -> bool {
        self.other_audio_playing
    }
}

/// An error from `MediaTime` construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MediaTimeError {
    /// The supplied timescale is zero or negative.
    InvalidTimescale,
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timescale_must_be_positive() {
        assert_eq!(MediaTime::new(1, 1).unwrap().timescale(), 1);
        assert_eq!(MediaTime::new(1, 0), Err(MediaTimeError::InvalidTimescale));
        assert_eq!(MediaTime::new(1, -1), Err(MediaTimeError::InvalidTimescale));
    }

    #[test]
    fn equivalent_rational_pairs_compare_equal() {
        let half = MediaTime::new(1, 2).unwrap();
        let three_sixths = MediaTime::new(3, 6).unwrap();

        assert_eq!(half, three_sixths);
        assert_eq!(half.cmp(&three_sixths), Ordering::Equal);
    }

    #[test]
    fn compare_orders_signed_rational_values() {
        let negative_half = MediaTime::new(-1, 2).unwrap();
        let negative_third = MediaTime::new(-1, 3).unwrap();
        let zero = MediaTime::new(0, 7).unwrap();
        let positive_half = MediaTime::new(1, 2).unwrap();

        assert!(negative_half < negative_third);
        assert!(negative_third < zero);
        assert!(zero < positive_half);
    }

    #[test]
    fn compare_uses_exact_wide_products_at_integer_limits() {
        let max_scale = MediaTime::new(i64::MAX, i32::MAX).unwrap();
        let min_value = MediaTime::new(i64::MIN, 1).unwrap();
        let max_over_two = MediaTime::new(i64::MAX, 2).unwrap();

        assert!(min_value < max_scale);
        assert!(max_scale < max_over_two);
    }
}

/// A video codec identified by four bytes in FourCC display order.
///
/// For example, H.264 uses `*b"avc1"` and HEVC uses `*b"hvc1"`. Construction does not assert
/// that any platform supports the codec.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VideoCodecType(u32);

impl VideoCodecType {
    /// Creates a codec identifier from four bytes in display order.
    pub const fn from_fourcc(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }

    /// Returns the FourCC bytes in display order.
    pub const fn to_fourcc(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// Returns the FourCC as a big-endian 32-bit value.
    pub const fn to_raw(self) -> u32 {
        self.0
    }
}

/// A system snapshot of hardware decoding support for one video codec.
///
/// A positive result does not reserve a decoder or guarantee future hardware resources.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HardwareDecodeSupport(bool);

impl HardwareDecodeSupport {
    /// Creates a support value from the platform-reported predicate.
    pub const fn from_system(supported: bool) -> Self {
        Self(supported)
    }

    /// Returns whether the platform reports hardware decode support for the queried codec.
    pub const fn is_supported(self) -> bool {
        self.0
    }
}

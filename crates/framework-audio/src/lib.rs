#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable audio formats and live PCM input contracts."]

/// The sample representation of portable PCM input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PcmSampleFormat {
    /// A 32-bit IEEE 754 floating-point sample.
    Float32,
}

/// The sample order of portable PCM input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PcmSampleLayout {
    /// Channels for each frame are adjacent in channel order.
    Interleaved,
}

/// Invalid values for a portable Float32 interleaved PCM input format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmInputFormatError {
    /// The sample rate is not finite and greater than zero.
    InvalidSampleRate,
    /// The channel count is zero.
    ZeroChannels,
}

/// Invalid output shape for a frame-based PCM read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PcmInputBufferError {
    /// The output sample count does not contain a whole number of frames.
    IncompleteFrame,
    /// The output sample count exceeds the frame-count range.
    FrameCountOverflow,
    /// The channel count cannot be represented by this target's address width.
    ChannelCountExceedsAddressSpace,
}

/// The consumer-facing format of live PCM input.
///
/// A format describes samples returned to the caller, not a platform's native buffer layout.
/// Implementations may convert a native planar buffer to interleaved output, but must not imply
/// resampling or channel conversion unless they perform it explicitly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PcmInputFormat {
    sample_rate_hz: f64,
    channels: u32,
}

impl PcmInputFormat {
    /// Creates a Float32 interleaved format with a finite positive sample rate and nonzero channels.
    pub fn float32_interleaved(
        sample_rate_hz: f64,
        channels: u32,
    ) -> Result<Self, PcmInputFormatError> {
        if !sample_rate_hz.is_finite() || sample_rate_hz <= 0.0 {
            return Err(PcmInputFormatError::InvalidSampleRate);
        }
        if channels == 0 {
            return Err(PcmInputFormatError::ZeroChannels);
        }
        Ok(Self {
            sample_rate_hz,
            channels,
        })
    }

    /// Returns the sample representation used by this format.
    pub const fn sample_format(self) -> PcmSampleFormat {
        PcmSampleFormat::Float32
    }

    /// Returns the channel order used by this format.
    pub const fn sample_layout(self) -> PcmSampleLayout {
        PcmSampleLayout::Interleaved
    }

    /// Returns the sample rate in hertz.
    pub const fn sample_rate_hz(self) -> f64 {
        self.sample_rate_hz
    }

    /// Returns the number of channels in each frame.
    pub const fn channels(self) -> u32 {
        self.channels
    }

    /// Validates an interleaved sample count and returns its whole-frame capacity.
    pub fn frames_for_interleaved_samples(
        self,
        sample_count: usize,
    ) -> Result<u32, PcmInputBufferError> {
        let channels = usize::try_from(self.channels)
            .map_err(|_| PcmInputBufferError::ChannelCountExceedsAddressSpace)?;
        if sample_count % channels != 0 {
            return Err(PcmInputBufferError::IncompleteFrame);
        }
        u32::try_from(sample_count / channels).map_err(|_| PcmInputBufferError::FrameCountOverflow)
    }
}

/// A statically dispatched source of live, interleaved Float32 PCM frames.
///
/// This data contract does not define permission, activation, capture startup, or shutdown policy;
/// each platform backend exposes those operations according to its public native APIs.
pub trait PcmInputStream {
    /// The backend-specific error returned by a read.
    type Error;

    /// Returns the configured format, or `None` before the backend has established one.
    fn format(&self) -> Option<PcmInputFormat>;

    /// Copies up to the output slice's frame capacity without waiting for more input.
    ///
    /// Returns frames, not samples. Only the prefix containing returned frames is written; the
    /// rest of the output slice is unchanged. An error leaves the output and read position
    /// unchanged. This operation does not allocate.
    fn try_read_interleaved(&mut self, output: &mut [f32]) -> Result<u32, Self::Error>;

    /// Returns the cumulative number of frames discarded by this stream, saturating at `u64::MAX`.
    fn dropped_frames(&self) -> u64;
}

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

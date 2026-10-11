#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Authorized iOS microphone recording to uncompressed PCM files"]

#[cfg(any(target_os = "ios", test))]
use framework_media_authorization::MediaAuthorizationStatus;

#[cfg(target_os = "ios")]
use framework_media_authorization::{CaptureMedia, MediaAuthorization};
#[cfg(target_os = "ios")]
use ios_media_authorization::IosMediaAuthorization;
#[cfg(target_os = "ios")]
use objc2::AnyThread;
#[cfg(target_os = "ios")]
use objc2::rc::Retained;
#[cfg(target_os = "ios")]
use objc2_avf_audio::{AVAudioCommonFormat, AVAudioFormat, AVAudioRecorder};
#[cfg(target_os = "ios")]
use objc2_foundation::NSURL;

/// PCM sample representation written to the recording file.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PcmSampleFormat {
    /// Signed 16-bit integer samples.
    Int16,
    /// 32-bit floating-point samples.
    Float32,
}

/// A bounded PCM recording format accepted by AVFAudio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PcmRecordingFormat {
    sample_rate_hz: f64,
    channels: u32,
    sample_format: PcmSampleFormat,
}

impl PcmRecordingFormat {
    /// Creates a PCM format with a finite positive sample rate and one or two channels.
    pub fn new(
        sample_rate_hz: f64,
        channels: u32,
        sample_format: PcmSampleFormat,
    ) -> Result<Self, PcmRecordingFormatError> {
        if !sample_rate_hz.is_finite() || sample_rate_hz <= 0.0 {
            return Err(PcmRecordingFormatError::InvalidSampleRate);
        }
        if !(1..=2).contains(&channels) {
            return Err(PcmRecordingFormatError::UnsupportedChannelCount);
        }
        Ok(Self {
            sample_rate_hz,
            channels,
            sample_format,
        })
    }

    /// Returns the sample rate in hertz.
    pub const fn sample_rate_hz(self) -> f64 {
        self.sample_rate_hz
    }

    /// Returns the channel count.
    pub const fn channels(self) -> u32 {
        self.channels
    }

    /// Returns the PCM sample representation.
    pub const fn sample_format(self) -> PcmSampleFormat {
        self.sample_format
    }
}

/// A rejected portable PCM format value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PcmRecordingFormatError {
    /// Sample rate must be finite and greater than zero.
    InvalidSampleRate,
    /// AVAudioFormat construction supports one or two channels in this API.
    UnsupportedChannelCount,
}

/// Failure to create or start an authorized microphone recorder.
#[cfg(target_os = "ios")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicrophoneRecorderError {
    /// Microphone authorization is not currently granted.
    AuthorizationNotGranted,
    /// The operating system predates the selected AVAudioRecorder format initializer.
    UnsupportedOperatingSystem,
    /// AVFAudio could not create the requested PCM format.
    UnsupportedFormat,
    /// AVAudioRecorder could not open the requested URL and format.
    RecorderInitializationFailed,
    /// AVAudioRecorder rejected the request to start recording.
    RecorderStartFailed,
}

#[cfg(any(target_os = "ios", test))]
const fn may_record(status: MediaAuthorizationStatus) -> bool {
    matches!(status, MediaAuthorizationStatus::Authorized)
}

/// An iOS microphone recorder that writes uncompressed PCM to a caller-owned file URL.
///
/// The host must provide `NSMicrophoneUsageDescription`, request authorization separately, and
/// configure and activate the process-wide audio session for recording. This type does not mutate
/// that shared audio session or request permission. The URL must be a writable file URL with an
/// extension matching the desired audio container, such as `.caf`; AVAudioRecorder may overwrite
/// a file that already exists at that URL. Stop closes the output file. Dropping this value stops
/// recording and closes the file, but does not deactivate the host-owned audio session.
///
/// Compile and link checks do not prove microphone capture or output-file contents on a device.
#[cfg(target_os = "ios")]
pub struct IosMicrophoneRecorder {
    recorder: Retained<AVAudioRecorder>,
}

#[cfg(target_os = "ios")]
impl IosMicrophoneRecorder {
    /// Creates a recorder for an already-authorized microphone and explicit PCM format.
    pub fn new(
        output_url: &NSURL,
        format: PcmRecordingFormat,
    ) -> Result<Self, MicrophoneRecorderError> {
        if !may_record(IosMediaAuthorization::authorization_status(
            CaptureMedia::Microphone,
        )) {
            return Err(MicrophoneRecorderError::AuthorizationNotGranted);
        }
        if !objc2::available!(ios = 10.0, ..) {
            return Err(MicrophoneRecorderError::UnsupportedOperatingSystem);
        }

        let common_format = match format.sample_format {
            PcmSampleFormat::Int16 => AVAudioCommonFormat::PCMFormatInt16,
            PcmSampleFormat::Float32 => AVAudioCommonFormat::PCMFormatFloat32,
        };
        // SAFETY: PcmRecordingFormat enforces a finite positive rate, one or two channels, and
        // only the two supported AVAudioCommonFormat values. AVAudioFormat is immutable.
        let native_format = unsafe {
            AVAudioFormat::initWithCommonFormat_sampleRate_channels_interleaved(
                AVAudioFormat::alloc(),
                common_format,
                format.sample_rate_hz,
                format.channels,
                true,
            )
        }
        .ok_or(MicrophoneRecorderError::UnsupportedFormat)?;

        // SAFETY: The caller supplies a valid file URL, and native_format is a valid PCM format.
        // AVAudioRecorder reports filesystem and format failures through its NSError result.
        let recorder = unsafe {
            AVAudioRecorder::initWithURL_format_error(
                AVAudioRecorder::alloc(),
                output_url,
                &native_format,
            )
        }
        .map_err(|_| MicrophoneRecorderError::RecorderInitializationFailed)?;

        Ok(Self { recorder })
    }

    /// Starts recording if microphone authorization remains granted.
    pub fn start(&mut self) -> Result<(), MicrophoneRecorderError> {
        if !may_record(IosMediaAuthorization::authorization_status(
            CaptureMedia::Microphone,
        )) {
            return Err(MicrophoneRecorderError::AuthorizationNotGranted);
        }
        // SAFETY: recorder is a live AVAudioRecorder and this call is serialized by &mut self.
        if unsafe { self.recorder.record() } {
            Ok(())
        } else {
            Err(MicrophoneRecorderError::RecorderStartFailed)
        }
    }

    /// Stops recording and closes the output file.
    pub fn stop(&mut self) {
        // SAFETY: recorder is a live AVAudioRecorder owned by self.
        unsafe { self.recorder.stop() };
    }

    /// Returns AVAudioRecorder's current recording flag.
    pub fn is_recording(&self) -> bool {
        // SAFETY: recorder is a live AVAudioRecorder owned by self.
        unsafe { self.recorder.isRecording() }
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosMicrophoneRecorder {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::{PcmRecordingFormat, PcmRecordingFormatError, PcmSampleFormat, may_record};
    use framework_media_authorization::MediaAuthorizationStatus;

    #[test]
    fn accepts_finite_positive_rates_one_or_two_channels_and_pcm_formats() {
        for channels in [1, 2] {
            for sample_format in [PcmSampleFormat::Int16, PcmSampleFormat::Float32] {
                let format = PcmRecordingFormat::new(48_000.0, channels, sample_format).unwrap();
                assert_eq!(format.sample_rate_hz(), 48_000.0);
                assert_eq!(format.channels(), channels);
                assert_eq!(format.sample_format(), sample_format);
            }
        }
    }

    #[test]
    fn rejects_non_finite_or_non_positive_sample_rates() {
        for rate in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                PcmRecordingFormat::new(rate, 1, PcmSampleFormat::Float32),
                Err(PcmRecordingFormatError::InvalidSampleRate)
            );
        }
    }

    #[test]
    fn rejects_channel_counts_outside_one_or_two() {
        for channels in [0, 3, u32::MAX] {
            assert_eq!(
                PcmRecordingFormat::new(48_000.0, channels, PcmSampleFormat::Int16),
                Err(PcmRecordingFormatError::UnsupportedChannelCount)
            );
        }
    }

    #[test]
    fn recording_requires_authorization_for_every_known_status() {
        assert!(!may_record(MediaAuthorizationStatus::NotDetermined));
        assert!(!may_record(MediaAuthorizationStatus::Restricted));
        assert!(!may_record(MediaAuthorizationStatus::Denied));
        assert!(may_record(MediaAuthorizationStatus::Authorized));
        assert!(!may_record(MediaAuthorizationStatus::Unknown));
    }
}

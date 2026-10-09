#![deny(missing_docs)]
#![doc = "iOS adapters for CoreMedia time, bounded AVAudioSession status, and VideoToolbox hardware-decode support."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::IosMediaTime;

#[cfg(target_os = "ios")]
mod audio_playback;

#[cfg(target_os = "ios")]
pub use audio_playback::other_audio_playback_snapshot;

#[cfg(target_os = "ios")]
mod video_toolbox;

#[cfg(target_os = "ios")]
pub use video_toolbox::hardware_decode_support;

/// Reports that VideoToolbox hardware-decode status is unavailable on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn hardware_decode_support(
    _codec: framework_media::VideoCodecType,
) -> Option<framework_media::HardwareDecodeSupport> {
    None
}

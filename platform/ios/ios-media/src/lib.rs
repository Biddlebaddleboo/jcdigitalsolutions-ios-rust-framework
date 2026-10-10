#![deny(missing_docs)]
#![doc = "iOS adapters for CoreMedia time, bounded AVAudioSession status, and VideoToolbox hardware-decode support."]

#[cfg(all(target_os = "ios", feature = "core-media-time"))]
mod platform;

#[cfg(all(target_os = "ios", feature = "core-media-time"))]
pub use platform::IosMediaTime;

#[cfg(target_os = "ios")]
mod audio_playback;

#[cfg(target_os = "ios")]
pub use audio_playback::other_audio_playback_snapshot;

#[cfg(all(target_os = "ios", feature = "videotoolbox"))]
mod video_toolbox;

#[cfg(all(target_os = "ios", feature = "videotoolbox"))]
pub use video_toolbox::hardware_decode_support;

/// Reports that VideoToolbox hardware-decode status is unavailable on non-iOS targets.
#[cfg(all(not(target_os = "ios"), feature = "videotoolbox"))]
pub fn hardware_decode_support(
    _codec: framework_media::VideoCodecType,
) -> Option<framework_media::HardwareDecodeSupport> {
    None
}

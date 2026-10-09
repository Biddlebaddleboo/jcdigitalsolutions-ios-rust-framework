use framework_media::{HardwareDecodeSupport, VideoCodecType};

/// Queries the current system's hardware-decode support for one codec.
///
/// Returns `None` below iOS 11.0. A `Some(false)` result is the system's report for this codec;
/// it does not describe software decode support.
pub fn hardware_decode_support(codec: VideoCodecType) -> Option<HardwareDecodeSupport> {
    if !objc2::available!(ios = 11.0, ..) {
        return None;
    }

    // SAFETY: The runtime guard checks the API floor, and the portable value provides the exact
    // FourCharCode scalar expected by VTIsHardwareDecodeSupported without pointer preconditions.
    let supported = unsafe { objc2_video_toolbox::VTIsHardwareDecodeSupported(codec.to_raw()) };
    Some(HardwareDecodeSupport::from_system(supported))
}

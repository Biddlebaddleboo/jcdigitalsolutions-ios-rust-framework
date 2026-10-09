#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A main-thread AVFoundation snapshot of system HDR playback eligibility."]

#[cfg(target_os = "ios")]
use framework_audio::HdrPlaybackEligibility;

/// Queries system HDR playback eligibility on the main thread.
///
/// Returns `None` off the main thread and on non-iOS targets. `Some(false)` is a system result,
/// not a claim that HDR playback is impossible for every asset or configuration. Returns `None`
/// below iOS 13.4 because the AVPlayer property is not available there.
#[cfg(target_os = "ios")]
pub fn hdr_playback_eligibility() -> Option<HdrPlaybackEligibility> {
    if !objc2::available!(ios = 13.4, ..) {
        return None;
    }
    let main_thread = objc2::MainThreadMarker::new()?;
    // SAFETY: The runtime guard proves this property is available, the marker proves this call is
    // on the main thread, and the class property returns a scalar without retaining an object.
    let eligible = unsafe { objc2_av_foundation::AVPlayer::eligibleForHDRPlayback(main_thread) };
    Some(HdrPlaybackEligibility::new(eligible))
}

/// Reports that this iOS-only query is unavailable on non-iOS targets.
#[cfg(not(target_os = "ios"))]
pub fn hdr_playback_eligibility() -> Option<framework_audio::HdrPlaybackEligibility> {
    None
}

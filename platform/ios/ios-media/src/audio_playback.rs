use framework_media::OtherAudioPlaybackSnapshot;
use objc2_avf_audio::AVAudioSession;

/// Reads whether another app is playing audio, without changing this app's audio session.
///
/// The result is a transient point-in-time snapshot. It can include ambient audio and does not
/// identify the source, report this app's playback, or control media. The API is available from
/// iOS 6.0.
pub fn other_audio_playback_snapshot() -> OtherAudioPlaybackSnapshot {
    // SAFETY: sharedInstance returns the documented process-wide AVAudioSession singleton.
    let session = unsafe { AVAudioSession::sharedInstance() };
    // SAFETY: this is the read-only scalar getter for the point-in-time other-audio state.
    OtherAudioPlaybackSnapshot::new(unsafe { session.isOtherAudioPlaying() })
}

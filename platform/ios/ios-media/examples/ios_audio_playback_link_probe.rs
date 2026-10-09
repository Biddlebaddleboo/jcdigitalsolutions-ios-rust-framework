#![deny(warnings)]

#[cfg(target_os = "ios")]
use ios_media::other_audio_playback_snapshot;

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(other_audio_playback_snapshot());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

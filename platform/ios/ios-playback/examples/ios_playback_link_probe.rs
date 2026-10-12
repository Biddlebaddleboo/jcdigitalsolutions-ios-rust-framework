#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    if let Some(main_thread) = objc2::MainThreadMarker::new() {
        let player = ios_playback::IosFilePlayer::from_local_file(
            core::hint::black_box("/tmp/ios-playback-link-probe.mov"),
            main_thread,
        );
        if let Ok(player) = player {
            player.play(main_thread);
            let _ = player.volume(main_thread);
            let _ = player.set_volume(0.5, main_thread);
            let _ = player.is_muted(main_thread);
            let _ = player.set_muted(false, main_thread);
            if let Some(observation) = player.observe_current_item_end(main_thread) {
                core::hint::black_box(observation.has_reached_end());
            }
            player.pause(main_thread);
            let seek = player.seek_to(framework_media::MediaTime::new(3, 2).unwrap(), main_thread);
            core::hint::black_box(&seek);
            core::hint::black_box(player.as_av_player(main_thread));
        }
    }
    let _ = core::hint::black_box(ios_playback::hdr_playback_eligibility());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

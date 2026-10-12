#[cfg(target_os = "ios")]
fn main() {
    if std::env::var_os("IOS_CAMERA_CAPTURE_LINK_PROBE").is_none() {
        return;
    }

    let Some(main_thread) = objc2::MainThreadMarker::new() else {
        return;
    };
    let Ok(session) = ios_camera_capture_session::IosCameraCaptureSession::new(main_thread) else {
        return;
    };
    for preset in [
        ios_camera_capture_session::IosCameraSessionPreset::Photo,
        ios_camera_capture_session::IosCameraSessionPreset::High,
        ios_camera_capture_session::IosCameraSessionPreset::Medium,
        ios_camera_capture_session::IosCameraSessionPreset::Low,
        ios_camera_capture_session::IosCameraSessionPreset::Cif352x288,
        ios_camera_capture_session::IosCameraSessionPreset::Vga640x480,
        ios_camera_capture_session::IosCameraSessionPreset::Hd1280x720,
        ios_camera_capture_session::IosCameraSessionPreset::FullHd1920x1080,
        ios_camera_capture_session::IosCameraSessionPreset::Uhd3840x2160,
    ] {
        let _ = std::hint::black_box(session.set_session_preset(preset));
    }
    let _ = std::hint::black_box(session.set_max_photo_quality_prioritization(
        ios_camera_capture_session::IosCameraPhotoQuality::Quality,
    ));
    let Some(main_thread) = objc2::MainThreadMarker::new() else {
        return;
    };
    let _ = std::hint::black_box(session.set_preview_orientation(
        main_thread,
        ios_camera_capture_session::IosCameraVideoOrientation::LandscapeRight,
    ));
    let Some(main_thread) = objc2::MainThreadMarker::new() else {
        return;
    };
    std::hint::black_box(session.preview_layer(main_thread));
    session.start();
    let _ = std::hint::black_box(session.set_torch_enabled(true));
    let _ = std::hint::black_box(session.set_torch_enabled(false));
    let _ = std::hint::black_box(session.set_zoom_factor(1.0));
    let _ = std::hint::black_box(session.set_focus(
        ios_camera_capture_session::IosCameraFocusMode::ContinuousAuto,
        Some(ios_camera_capture_session::NormalizedFocusPoint::new(
            0.5, 0.5,
        )),
    ));
    let _ = std::hint::black_box(
        session.set_focus(ios_camera_capture_session::IosCameraFocusMode::Locked, None),
    );
    for mode in [
        ios_camera_capture_session::IosCameraWhiteBalanceMode::Locked,
        ios_camera_capture_session::IosCameraWhiteBalanceMode::AutoWhiteBalance,
        ios_camera_capture_session::IosCameraWhiteBalanceMode::ContinuousAutoWhiteBalance,
    ] {
        let _ = std::hint::black_box(session.is_white_balance_mode_supported(mode));
        let _ = std::hint::black_box(session.set_white_balance_mode(mode));
    }
    let _ = std::hint::black_box(session.set_exposure(
        ios_camera_capture_session::IosCameraExposureMode::AutoExpose,
        Some(ios_camera_capture_session::NormalizedFocusPoint::new(
            0.5, 0.5,
        )),
    ));
    let _ = std::hint::black_box(session.set_exposure(
        ios_camera_capture_session::IosCameraExposureMode::ContinuousAutoExposure,
        None,
    ));
    let _ = std::hint::black_box(session.exposure_target_bias_range());
    let _ = std::hint::black_box(session.set_exposure_target_bias(0.0));
    let _ = std::hint::black_box(session.custom_exposure_range());
    if let Ok(duration) = framework_media::MediaTime::new(1, 60) {
        let _ = std::hint::black_box(session.set_custom_exposure(duration, 100.0));
    }
    let _ = std::hint::black_box(session.set_movie_orientation(
        ios_camera_capture_session::IosCameraVideoOrientation::LandscapeRight,
    ));
    let _ = session.capture_photo(|result| {
        let _ = std::hint::black_box(result);
    });
    for mode in [
        ios_camera_capture_session::IosCameraFlashMode::Off,
        ios_camera_capture_session::IosCameraFlashMode::On,
        ios_camera_capture_session::IosCameraFlashMode::Auto,
    ] {
        let _ = session.capture_photo_with_flash(mode, |result| {
            let _ = std::hint::black_box(result);
        });
    }
    for quality in [
        ios_camera_capture_session::IosCameraPhotoQuality::Speed,
        ios_camera_capture_session::IosCameraPhotoQuality::Balanced,
        ios_camera_capture_session::IosCameraPhotoQuality::Quality,
    ] {
        let _ = session.capture_photo_with_quality(quality, |result| {
            let _ = std::hint::black_box(result);
        });
    }
    let movie_path = std::path::PathBuf::from("/tmp/ios-camera-link-probe.mov");
    let _ = std::hint::black_box(session.start_video_recording(&movie_path, |result| {
        let _ = std::hint::black_box(result);
    }));
    let _ = std::hint::black_box(
        session.start_video_recording_with_audio(&movie_path, |result| {
            let _ = std::hint::black_box(result);
        }),
    );
    session.stop_video_recording();
    let _ = session.start_video_frame_stream(|sample| {
        let _ = std::hint::black_box(sample);
    });
    session.stop_video_frame_stream();
    session.stop();
    let Some(main_thread) = objc2::MainThreadMarker::new() else {
        return;
    };
    let _ = std::hint::black_box(
        ios_camera_capture_session::IosCameraCaptureSession::new_with_camera_position(
            main_thread,
            ios_camera_capture_session::IosCameraPosition::Front,
        ),
    );
    let _ = std::hint::black_box(
        session.set_camera_position(ios_camera_capture_session::IosCameraPosition::Front),
    );
    let _ = std::hint::black_box(
        session.set_camera_position(ios_camera_capture_session::IosCameraPosition::Back),
    );
}

#[cfg(not(target_os = "ios"))]
fn main() {}

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
    let _ = std::hint::black_box(session.set_movie_orientation(
        ios_camera_capture_session::IosCameraVideoOrientation::LandscapeRight,
    ));
    let _ = session.capture_photo(|result| {
        let _ = std::hint::black_box(result);
    });
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

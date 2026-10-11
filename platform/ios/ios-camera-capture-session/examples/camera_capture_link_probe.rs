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
    let _ = session.capture_photo(|result| {
        let _ = std::hint::black_box(result);
    });
    let movie_path = std::path::PathBuf::from("/tmp/ios-camera-link-probe.mov");
    let _ = std::hint::black_box(session.start_video_recording(&movie_path, |result| {
        let _ = std::hint::black_box(result);
    }));
    session.stop_video_recording();
    let _ = session.start_video_frame_stream(|sample| {
        let _ = std::hint::black_box(sample);
    });
    session.stop_video_frame_stream();
    session.stop();
}

#[cfg(not(target_os = "ios"))]
fn main() {}

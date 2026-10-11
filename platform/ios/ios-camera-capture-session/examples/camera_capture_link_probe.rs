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
    session.stop();
}

#[cfg(not(target_os = "ios"))]
fn main() {}

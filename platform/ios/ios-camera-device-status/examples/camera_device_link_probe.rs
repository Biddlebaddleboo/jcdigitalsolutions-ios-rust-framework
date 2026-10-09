#![allow(dead_code)]

#[cfg(target_os = "ios")]
fn main() {
    std::hint::black_box(ios_camera_device_status::has_default_video_capture_device());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

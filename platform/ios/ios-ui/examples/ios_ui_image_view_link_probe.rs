#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosImageView, IosUiBackend};
#[cfg(target_os = "ios")]
use objc2_ui_kit::UIImage;

#[cfg(target_os = "ios")]
#[inline(never)]
fn replace_image(view: &mut IosImageView, image: &UIImage) {
    let _ = view.set_image(Some(image));
}

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let frame = Frame::new(0.0, 0.0, 32.0, 32.0).expect("valid frame");
    let Ok(mut parent) = backend.create_view(frame) else {
        return;
    };
    let Ok(mut image_view) = backend.add_image_view(&mut parent, frame, None) else {
        return;
    };
    let _ = image_view.native_image_view();
    let _ = image_view.clear_image();
    core::hint::black_box(replace_image as fn(&mut IosImageView, &UIImage));
}

#[cfg(not(target_os = "ios"))]
fn main() {}

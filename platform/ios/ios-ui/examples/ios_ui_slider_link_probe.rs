#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosSlider, IosSliderConfig, IosSliderRange, IosSliderReporting, IosUiBackend};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let frame = Frame::new(0.0, 0.0, 180.0, 32.0).expect("valid slider frame");
    let parent_frame = Frame::new(0.0, 0.0, 240.0, 80.0).expect("valid parent frame");
    let Ok(mut parent) = backend.create_view(parent_frame) else {
        return;
    };
    let range = IosSliderRange::new(-10.0, 10.0).expect("valid slider range");
    let config = IosSliderConfig::new(range, 2.0, IosSliderReporting::Continuous)
        .expect("valid slider config");
    let mut slider: IosSlider = match backend.add_slider(&mut parent, frame, config, |_| {}) {
        Ok(slider) => slider,
        Err(_) => return,
    };
    let _ = slider.value();
    let _ = slider.set_value(4.0, false);
    let _ = slider.reporting();
    let _ = slider.set_reporting(IosSliderReporting::OnRelease);
    core::hint::black_box(slider.native_slider());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

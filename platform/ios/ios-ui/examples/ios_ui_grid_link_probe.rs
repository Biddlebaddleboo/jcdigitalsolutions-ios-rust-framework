#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::IosUiBackend;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let parent_frame = Frame::new(0.0, 0.0, 240.0, 120.0).expect("valid parent frame");
    let grid_frame = Frame::new(0.0, 0.0, 240.0, 120.0).expect("valid grid frame");
    let label_frame = Frame::new(0.0, 0.0, 0.0, 0.0).expect("valid label frame");
    let Ok(mut parent) = backend.create_view(parent_frame) else {
        return;
    };
    let Ok(mut grid) = backend.add_grid_view(&mut parent, grid_frame, 2, 10.0, 8.0) else {
        return;
    };
    for title in ["first", "second", "third"] {
        let Ok(label) = backend.add_label(&mut parent, label_frame, title) else {
            return;
        };
        let _ = grid.add_arranged_subview(label.native_label());
    }
    core::hint::black_box(grid.native_grid_view());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

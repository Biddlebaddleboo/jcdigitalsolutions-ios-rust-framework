#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosStackAxis, IosUiBackend};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let parent_frame = Frame::new(0.0, 0.0, 240.0, 80.0).expect("valid parent frame");
    let stack_frame = Frame::new(0.0, 0.0, 240.0, 80.0).expect("valid stack frame");
    let child_frame = Frame::new(0.0, 0.0, 0.0, 0.0).expect("valid child frame");
    let Ok(mut parent) = backend.create_view(parent_frame) else {
        return;
    };
    let Ok(mut stack) =
        backend.add_stack_view(&mut parent, stack_frame, IosStackAxis::Vertical, 8.0)
    else {
        return;
    };
    let Ok(label) = backend.add_label(&mut parent, child_frame, "Stack link probe") else {
        return;
    };
    let _ = stack.add_arranged_subview(label.native_label());
    let _ = stack.set_spacing(4.0);
    core::hint::black_box(stack.native_stack_view());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

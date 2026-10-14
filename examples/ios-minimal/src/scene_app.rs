use ios_app_scenes::{
    IosApplicationHandler, IosApplicationHost, IosSceneHost, SceneLifecycleEventKind,
};
use ios_runtime::main_thread::MainThread;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::NSString;
use objc2_ui_kit::UILabel;

struct SampleApplication;

impl IosApplicationHandler for SampleApplication {
    fn scene_event(&mut self, scene: &IosSceneHost, event: SceneLifecycleEventKind) {
        if event != SceneLifecycleEventKind::WillConnect {
            return;
        }
        let Some(main_thread) = MainThread::current() else {
            return;
        };
        let label = UILabel::new(main_thread.into_objc2());
        label.setText(Some(&NSString::from_str("UIKit scene hosted by Rust")));
        label.setFrame(CGRect::new(
            CGPoint::new(24.0, 96.0),
            CGSize::new(320.0, 44.0),
        ));
        let _ = scene.attach_view(&label);
    }
}

pub fn run() {
    let main_thread =
        MainThread::current().expect("UIApplicationMain must start on the main thread");
    IosApplicationHost::run(SampleApplication, main_thread);
}

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use ios_app_scenes::{
    IosApplicationHandler, IosApplicationHost, IosSceneHost, SceneLifecycleEventKind,
    SceneSessionIdentifier,
};
use ios_runtime::main_thread::MainThread;
use ios_sharing::{
    DocumentExportResult, IosDocumentExportFile, IosDocumentExportSession, IosShareFile,
    IosShareSession,
};
use ios_ui::{
    AccessibilityActionId, AccessibilityNodeId, IosAlertActionStyle, IosAlertKeyboardType,
    IosAlertSession, IosAlertStyle, IosAlertTextFieldConfig, IosCustomRenderSession,
    IosDisplayFrame, IosTextInputTraits, Rgba8Frame, SemanticAccessibilityAction,
    SemanticAccessibilityNode, SemanticFrame,
};
use objc2::rc::Retained;
use objc2::runtime::{NSObject, Sel};
use objc2::{DefinedClass, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIButton, UIColor, UIControlEvents, UIControlState, UILabel, UIView, UIViewController,
};

type SharedScene = Rc<RefCell<SceneState>>;

struct SampleApplication {
    scenes: Vec<SampleScene>,
}

struct SampleScene {
    identifier: SceneSessionIdentifier,
    state: SharedScene,
    _button_target: Retained<SampleButtonTarget>,
}

struct SceneState {
    controller: Retained<UIViewController>,
    status: Retained<UILabel>,
    render: Option<IosCustomRenderSession>,
    alert: Option<IosAlertSession>,
    alert_finished: bool,
    document_export: IosDocumentExportSession,
    share: IosShareSession,
    display_frames: u32,
    touch_events: u32,
}

struct ButtonTargetState {
    scene: Weak<RefCell<SceneState>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMinimalSceneActions"]
    #[ivars = ButtonTargetState]
    struct SampleButtonTarget;

    impl SampleButtonTarget {
        #[unsafe(method(showPrompt:))]
        fn show_prompt(&self, _sender: &UIButton) {
            let _ = ios_runtime::ffi::catch_unwind(|| {
                self.with_scene(|state, weak| state.show_prompt(weak));
            });
        }

        #[unsafe(method(shareSampleFile:))]
        fn share_sample_file(&self, _sender: &UIButton) {
            let _ = ios_runtime::ffi::catch_unwind(|| {
                self.with_scene(|state, weak| state.share_sample_file(weak));
            });
        }

        #[unsafe(method(exportSampleFile:))]
        fn export_sample_file(&self, _sender: &UIButton) {
            let _ = ios_runtime::ffi::catch_unwind(|| {
                self.with_scene(|state, weak| state.export_sample_file(weak));
            });
        }

        #[unsafe(method(focusSampleText:))]
        fn focus_sample_text(&self, _sender: &UIButton) {
            let _ = ios_runtime::ffi::catch_unwind(|| {
                self.with_scene(|state, _weak| state.focus_sample_text());
            });
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for SampleButtonTarget {}
);

impl SampleButtonTarget {
    fn new(marker: objc2::MainThreadMarker, scene: Weak<RefCell<SceneState>>) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(ButtonTargetState { scene });
        // SAFETY: The ivars are initialized before NSObject's initializer is called.
        unsafe { msg_send![super(this), init] }
    }

    fn with_scene(&self, action: impl FnOnce(&mut SceneState, Weak<RefCell<SceneState>>)) {
        let Some(scene) = self.ivars().scene.upgrade() else {
            return;
        };
        let weak = Rc::downgrade(&scene);
        if let Ok(mut state) = scene.try_borrow_mut() {
            action(&mut state, weak);
        }
    }
}

impl IosApplicationHandler for SampleApplication {
    fn scene_event(&mut self, scene: &IosSceneHost, event: SceneLifecycleEventKind) {
        match event {
            SceneLifecycleEventKind::WillConnect => {
                if self
                    .scenes
                    .iter()
                    .any(|sample| sample.identifier == *scene.session_identifier())
                {
                    return;
                }
                if let Some(sample) = SampleScene::new(scene) {
                    self.scenes.push(sample);
                }
            }
            SceneLifecycleEventKind::DidDisconnect => {
                if let Some(index) = self
                    .scenes
                    .iter()
                    .position(|sample| sample.identifier == *scene.session_identifier())
                {
                    if let Ok(mut state) = self.scenes[index].state.try_borrow_mut() {
                        state.disconnect();
                    }
                    self.scenes.remove(index);
                }
            }
            SceneLifecycleEventKind::DidActivate => {
                self.set_scene_active(scene.session_identifier(), true);
            }
            SceneLifecycleEventKind::WillDeactivate
            | SceneLifecycleEventKind::WillEnterForeground
            | SceneLifecycleEventKind::DidEnterBackground => {
                self.set_scene_active(scene.session_identifier(), false);
            }
        }
    }
}

impl SampleApplication {
    fn set_scene_active(&mut self, identifier: &SceneSessionIdentifier, active: bool) {
        if let Some(sample) = self
            .scenes
            .iter()
            .find(|sample| sample.identifier == *identifier)
            && let Ok(mut state) = sample.state.try_borrow_mut()
            && let Some(render) = &mut state.render
        {
            let _ = render.set_active(active);
        }
    }
}

impl SampleScene {
    fn new(scene: &IosSceneHost) -> Option<Self> {
        let main_thread = MainThread::current()?;
        let marker = main_thread.into_objc2();
        let root_view = scene.root_view().ok()?;
        let controller = scene.root_view_controller().retain();
        let bounds = root_view.bounds();
        let safe_area = root_view.safeAreaInsets();
        let content_width = (bounds.size.width - 48.0).max(1.0);
        let render_frame = CGRect::new(
            CGPoint::new(24.0, safe_area.top + 164.0),
            CGSize::new(content_width, 140.0),
        );
        let text_frame = CGRect::new(
            CGPoint::new(24.0, safe_area.top + 318.0),
            CGSize::new(content_width, 68.0),
        );

        let status = UILabel::new(marker);
        status.setFrame(CGRect::new(
            CGPoint::new(24.0, safe_area.top + 12.0),
            CGSize::new(content_width, 36.0),
        ));
        // SAFETY: UILabel accepts a live system color returned by UIKit.
        unsafe { status.setTextColor(Some(&UIColor::labelColor())) };
        set_label(&status, "Rust scene host · native UIKit APIs");

        let share_main_thread = MainThread::current()?;
        let share_anchor = CGRect::new(
            CGPoint::new(bounds.size.width / 2.0, bounds.size.height / 2.0),
            CGSize::new(1.0, 1.0),
        );
        let share = IosShareSession::new(
            share_main_thread,
            controller.clone(),
            root_view.clone(),
            share_anchor,
        );
        let export_main_thread = MainThread::current()?;
        let document_export = IosDocumentExportSession::new(export_main_thread, &controller);
        let state = Rc::new(RefCell::new(SceneState {
            controller,
            status: status.clone(),
            render: None,
            alert: None,
            alert_finished: false,
            document_export,
            share,
            display_frames: 0,
            touch_events: 0,
        }));

        let mut render = IosCustomRenderSession::new(root_view.clone(), marker);
        render.set_frame(render_frame).ok()?;
        let pixels = sample_pixels(0);
        let frame = Rgba8Frame::new(64, 32, 64 * 4, &pixels, None).ok()?;
        render.present(&frame).ok()?;
        render.enable_touch_input().ok()?;
        render.enable_accessibility().ok()?;
        let accessibility_node = SemanticAccessibilityNode::new(
            AccessibilityNodeId(1),
            SemanticFrame {
                x: 0.0,
                y: 0.0,
                width: render_frame.size.width,
                height: render_frame.size.height,
            },
        )
        .with_label("Rust-rendered color field")
        .with_hint("Touch the image to update its color")
        .with_activation(SemanticAccessibilityAction::new(
            AccessibilityActionId(1),
            "Activate sample rendering",
        ));
        render.set_accessibility_nodes(&[accessibility_node]).ok()?;

        let text_state = Rc::downgrade(&state);
        render
            .enable_text_input(
                text_frame,
                "Rust text input",
                IosTextInputTraits::default(),
                move |result| {
                    if let Ok(snapshot) = result
                        && snapshot.marked_text.is_none()
                        && let Some(state) = text_state.upgrade()
                        && let Ok(mut state) = state.try_borrow_mut()
                    {
                        state.set_status("Native text input changed");
                    }
                },
            )
            .ok()?;

        let frame_state = Rc::downgrade(&state);
        render
            .enable_display_link(move |frame| {
                if let Some(state) = frame_state.upgrade()
                    && let Ok(mut state) = state.try_borrow_mut()
                {
                    state.on_display_frame(frame);
                }
            })
            .ok()?;
        state.borrow_mut().render = Some(render);

        let target = SampleButtonTarget::new(marker, Rc::downgrade(&state));
        let button_width = (content_width - 8.0).max(2.0) / 2.0;
        let button_titles = ["Prompt", "Share", "Export", "Text"];
        let button_actions = [
            sel!(showPrompt:),
            sel!(shareSampleFile:),
            sel!(exportSampleFile:),
            sel!(focusSampleText:),
        ];
        for index in 0..button_titles.len() {
            let column = index % 2;
            let row = index / 2;
            let x = 24.0 + column as f64 * (button_width + 8.0);
            let y = safe_area.top + 52.0 + row as f64 * 48.0;
            add_button(
                marker,
                &root_view,
                &target,
                button_titles[index],
                button_actions[index],
                CGRect::new(CGPoint::new(x, y), CGSize::new(button_width, 44.0)),
            );
        }
        root_view.addSubview(&status);

        Some(Self {
            identifier: scene.session_identifier().clone(),
            state,
            _button_target: target,
        })
    }
}

impl SceneState {
    fn show_prompt(&mut self, weak: Weak<RefCell<SceneState>>) {
        if self.alert.is_some() && !self.alert_finished {
            self.set_status("A prompt is already open");
            return;
        }
        self.alert.take();
        let Some(main_thread) = MainThread::current() else {
            self.set_status("Main-thread proof unavailable");
            return;
        };
        let Ok(mut alert) = IosAlertSession::new(
            main_thread,
            &self.controller,
            IosAlertStyle::Alert,
            Some("Rust-owned alert"),
            Some("UIKit owns this native presentation"),
        ) else {
            self.set_status("Could not create alert");
            return;
        };
        let field = IosAlertTextFieldConfig {
            placeholder: Some("Short note".to_owned()),
            keyboard_type: IosAlertKeyboardType::Default,
            maximum_utf16_units: 128,
            ..IosAlertTextFieldConfig::default()
        };
        if alert.add_text_field(field).is_err()
            || alert
                .add_action("Done", IosAlertActionStyle::Default)
                .is_err()
            || alert
                .add_action("Cancel", IosAlertActionStyle::Cancel)
                .is_err()
        {
            self.set_status("Could not configure alert");
            return;
        }
        match alert.present(move |result| {
            let text = match result {
                ios_ui::IosAlertResult::ActionSelected { .. } => "Alert action selected",
                ios_ui::IosAlertResult::Cancelled => "Alert cancelled",
                ios_ui::IosAlertResult::Failed(_) => "Alert returned invalid native text",
            };
            if let Some(state) = weak.upgrade()
                && let Ok(mut state) = state.try_borrow_mut()
            {
                state.alert_finished = true;
                state.set_status(text);
            }
        }) {
            Ok(()) => {
                self.alert = Some(alert);
                self.alert_finished = false;
                self.set_status("Alert presented by Rust session");
            }
            Err(_) => self.set_status("UIKit rejected alert presentation"),
        }
    }

    fn share_sample_file(&mut self, weak: Weak<RefCell<SceneState>>) {
        let file = match IosShareFile::from_utf8(
            "ios-rust-sample.txt",
            "A bounded UTF-8 file staged by the Rust iOS framework.\n",
        ) {
            Ok(file) => file,
            Err(_) => {
                self.set_status("Could not stage sample file");
                return;
            }
        };
        match self.share.start_staged_file(file, move |result| {
            let text = if result.is_ok() {
                "Share sheet completed or dismissed"
            } else {
                "Share sheet returned an error"
            };
            if let Some(state) = weak.upgrade()
                && let Ok(mut state) = state.try_borrow_mut()
            {
                state.set_status(text);
            }
        }) {
            Ok(()) => self.set_status("Native share sheet requested"),
            Err(_) => self.set_status("UIKit rejected share request"),
        }
    }

    fn export_sample_file(&mut self, weak: Weak<RefCell<SceneState>>) {
        let file = match IosDocumentExportFile::from_bytes(
            "ios-rust-sample.txt",
            b"A Rust-owned document export sample.\n",
        ) {
            Ok(file) => file,
            Err(_) => {
                self.set_status("Could not stage export file");
                return;
            }
        };
        match self.document_export.start_staged(file, move |result| {
            let text = match result {
                DocumentExportResult::Exported => "Document export completed",
                DocumentExportResult::Cancelled => "Document export cancelled",
            };
            if let Some(state) = weak.upgrade()
                && let Ok(mut state) = state.try_borrow_mut()
            {
                state.set_status(text);
            }
        }) {
            Ok(()) => self.set_status("Document picker requested"),
            Err(_) => self.set_status("UIKit rejected export request"),
        }
    }

    fn focus_sample_text(&mut self) {
        let focused = self
            .render
            .as_mut()
            .and_then(IosCustomRenderSession::text_input_mut)
            .is_some_and(|input| input.request_focus().unwrap_or(false));
        self.set_status(if focused {
            "Native text input focused"
        } else {
            "Native text input focus unavailable"
        });
    }

    fn on_display_frame(&mut self, frame: IosDisplayFrame) {
        self.display_frames = self.display_frames.saturating_add(1);
        let mut latest_touch = None;
        if let Some(render) = &self.render {
            for _ in 0..32 {
                let Some(event) = render.next_touch_event() else {
                    break;
                };
                self.touch_events = self.touch_events.saturating_add(1);
                latest_touch = Some(event);
            }
        }
        if let Some(event) = latest_touch {
            self.set_status(&format!(
                "Touch {:?} at {:.0}, {:.0} pt",
                event.phase, event.x_points, event.y_points
            ));
            self.present_sample_frame(self.touch_events);
        } else if let Some(action) = self
            .render
            .as_mut()
            .and_then(IosCustomRenderSession::take_accessibility_action)
        {
            self.set_status(&format!("Accessibility action: {}", action.action_id.0));
        } else if self.display_frames % 120 == 0 {
            self.set_status(&format!(
                "Display link active · {:.1} ms frame estimate",
                frame.duration_seconds() * 1000.0
            ));
        }
    }

    fn present_sample_frame(&self, seed: u32) {
        let pixels = sample_pixels(seed);
        let Ok(frame) = Rgba8Frame::new(64, 32, 64 * 4, &pixels, None) else {
            return;
        };
        if let Some(render) = &self.render {
            let _ = render.present(&frame);
        }
    }

    fn disconnect(&mut self) {
        if let Some(render) = &mut self.render {
            let _ = render.set_active(false);
            render.detach();
            while render.next_touch_event().is_some() {}
        }
        let _ = self.share.cancel();
        let _ = self.document_export.cancel();
    }

    fn set_status(&mut self, text: &str) {
        set_label(&self.status, text);
    }
}

fn add_button(
    marker: objc2::MainThreadMarker,
    root: &UIView,
    target: &SampleButtonTarget,
    title: &str,
    action: Sel,
    frame: CGRect,
) {
    let button = UIButton::new(marker);
    button.setFrame(frame);
    let title = NSString::from_str(title);
    button.setTitle_forState(Some(&title), UIControlState::Normal);
    button.setTitleColor_forState(Some(&UIColor::systemBlueColor()), UIControlState::Normal);
    // SAFETY: `target` is retained by its per-scene Rust owner while this button is in the scene.
    unsafe {
        button.addTarget_action_forControlEvents(
            Some(target),
            action,
            UIControlEvents::TouchUpInside,
        );
    }
    root.addSubview(&button);
}

fn set_label(label: &UILabel, text: &str) {
    let text = NSString::from_str(text);
    label.setText(Some(&text));
}

fn sample_pixels(seed: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(64 * 32 * 4);
    for y in 0_u32..32 {
        for x in 0_u32..64 {
            let shift = seed.wrapping_add(x / 8).wrapping_add(y / 4) as u8;
            pixels.extend_from_slice(&[
                32_u8.wrapping_add(shift.wrapping_mul(3)),
                96_u8.wrapping_add(shift.wrapping_mul(2)),
                176_u8.wrapping_add(shift),
                255,
            ]);
        }
    }
    pixels
}

pub fn run() {
    let main_thread =
        MainThread::current().expect("UIApplicationMain must start on the main thread");
    IosApplicationHost::run(SampleApplication { scenes: Vec::new() }, main_thread);
}

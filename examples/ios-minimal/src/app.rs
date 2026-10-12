use std::cell::{Cell, OnceCell, RefCell};
#[cfg(feature = "custom-render-metal-demo")]
use std::time::{Duration, Instant};

use ios_runtime::{ffi::catch_unwind, main_thread::MainThread};
#[cfg(feature = "custom-render-metal-demo")]
use ios_ui::{Bgra8Frame, IosCustomMetalHost};
use ios_ui::{
    IosCustomRenderHost, IosCustomTextInput, IosTextInputTraits, Rgba8Frame,
};
use objc2::rc::{Allocated, Retained};
use objc2::{ClassType, DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UIButton, UIColor, UIControlEvents, UIControlState,
    UILabel, UIScreen, UIView, UIViewController, UIWindow,
};

#[derive(Default)]
struct AppState {
    window: OnceCell<Retained<UIWindow>>,
    target: OnceCell<Retained<ButtonTarget>>,
    render_host: OnceCell<IosCustomRenderHost>,
    #[cfg(feature = "custom-render-metal-demo")]
    metal_host: RefCell<Option<IosCustomMetalHost>>,
    #[cfg(feature = "custom-render-metal-demo")]
    metal_benchmark_cpu_host: RefCell<Option<IosCustomRenderHost>>,
    #[cfg(feature = "custom-render-metal-demo")]
    metal_benchmark_ran: Cell<bool>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - AppDelegate does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMinimalAppDelegate"]
    #[ivars = AppState]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(AppState::default());
            // SAFETY: `this` is a fresh allocation of this NSObject subclass and NSObject's `init` is valid.
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(application:didFinishLaunchingWithOptions:))]
        unsafe fn did_finish_launching(
            &self,
            _application: &UIApplication,
            _options: Option<&objc2_foundation::NSDictionary>,
        ) -> bool {
            catch_unwind(|| self.install_ui()).unwrap_or(false)
        }

        #[cfg(feature = "custom-render-metal-demo")]
        #[unsafe(method(applicationDidBecomeActive:))]
        fn did_become_active(&self, _application: &UIApplication) {
            let _ = catch_unwind(|| {
                if std::env::args_os().any(|arg| arg == "--custom-metal-render-benchmark") {
                    self.run_custom_metal_benchmark();
                } else {
                    self.present_custom_metal_demo();
                }
            });
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and the class uses NSObject's implementation.
    unsafe impl NSObjectProtocol for AppDelegate {}
    // SAFETY: the class implements the documented launch selector with its generated ABI signature.
    unsafe impl UIApplicationDelegate for AppDelegate {}
);

struct TargetState {
    label: Retained<UILabel>,
    render_host: RefCell<Option<IosCustomRenderHost>>,
    text_input: RefCell<Option<IosCustomTextInput>>,
    taps: Cell<u32>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ButtonTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMinimalButtonTarget"]
    #[ivars = TargetState]
    struct ButtonTarget;

    impl ButtonTarget {
        #[unsafe(method(buttonTapped:))]
        fn button_tapped(&self, _sender: &UIButton) {
            let _ = catch_unwind(|| {
                let taps = self.ivars().taps.get().wrapping_add(1);
                self.ivars().taps.set(taps);
                let pixels = demo_pixels(taps);
                let frame = Rgba8Frame::new(64, 32, 64 * 4, &pixels, None)
                    .expect("fixed demo frame is valid");
                if self
                    .ivars()
                    .render_host
                    .borrow()
                    .as_ref()
                    .is_some_and(|host| host.present(&frame).is_ok())
                {
                    let text = NSString::from_str(&format!("Rust callback: {taps} tap(s)"));
                    self.ivars().label.setText(Some(&text));
                } else {
                    let text = NSString::from_str("Rust framebuffer presentation failed");
                    self.ivars().label.setText(Some(&text));
                }
            });
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and the class uses NSObject's implementation.
    unsafe impl NSObjectProtocol for ButtonTarget {}
);

impl ButtonTarget {
    fn with_state(
        mtm: objc2::MainThreadMarker,
        label: Retained<UILabel>,
        render_host: IosCustomRenderHost,
        text_input: Option<IosCustomTextInput>,
    ) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TargetState {
            label,
            render_host: RefCell::new(Some(render_host)),
            text_input: RefCell::new(text_input),
            taps: Cell::new(0),
        });
        // SAFETY: `this` is a fresh allocation of this NSObject subclass and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

impl AppDelegate {
    #[allow(deprecated)]
    fn install_ui(&self) -> bool {
        let mtm = MainThread::current()
            .expect("UIKit launch callback must be on the main thread")
            .into_objc2();
        let bounds = UIScreen::mainScreen(mtm).bounds();
        let label_frame = CGRect::new(
            CGPoint::new(24.0, 96.0),
            CGSize::new(bounds.size.width - 48.0, 44.0),
        );
        let button_frame = CGRect::new(
            CGPoint::new(24.0, 152.0),
            CGSize::new(bounds.size.width - 48.0, 52.0),
        );

        let window = UIWindow::initWithFrame(UIWindow::alloc(mtm), bounds);
        let controller = UIViewController::new(mtm);
        let root_view = UIView::new(mtm);
        root_view.setFrame(bounds);
        root_view.setBackgroundColor(Some(&UIColor::systemBackgroundColor()));

        let args = std::env::args_os().collect::<Vec<_>>();
        let render_demo = args.iter().any(|arg| arg == "--custom-render-host-demo");
        let render_lifecycle = args
            .iter()
            .any(|arg| arg == "--exercise-custom-render-host-lifecycle");
        let exercise_text_input = args.iter().any(|arg| arg == "--exercise-custom-text-input");
        let metal_render_demo = args.iter().any(|arg| arg == "--custom-metal-render-demo");
        #[cfg(feature = "custom-render-metal-demo")]
        let metal_render_benchmark = args
            .iter()
            .any(|arg| arg == "--custom-metal-render-benchmark");
        #[cfg(feature = "custom-render-metal-demo")]
        if metal_render_demo || metal_render_benchmark {
            let Ok(mut host) = IosCustomMetalHost::new(mtm) else {
                return false;
            };
            host.attach_to(
                &root_view,
                CGRect::new(
                    CGPoint::new(24.0, 96.0),
                    CGSize::new(bounds.size.width - 48.0, 240.0),
                ),
            );
            if metal_render_benchmark {
                let mut cpu_host = IosCustomRenderHost::new(mtm);
                cpu_host.attach_to(
                    &root_view,
                    CGRect::new(
                        CGPoint::new(24.0, 352.0),
                        CGSize::new(bounds.size.width - 48.0, 240.0),
                    ),
                );
                *self.ivars().metal_benchmark_cpu_host.borrow_mut() = Some(cpu_host);
            }
            controller.setView(Some(&root_view));
            window.setRootViewController(Some(&controller));
            *self.ivars().metal_host.borrow_mut() = Some(host);
            if self.ivars().window.set(window).is_err() {
                return false;
            }
            self.ivars()
                .window
                .get()
                .expect("Metal demo retains its window")
                .makeKeyAndVisible();
            return true;
        }
        #[cfg(not(feature = "custom-render-metal-demo"))]
        if metal_render_demo
            || args
                .iter()
                .any(|arg| arg == "--custom-metal-render-benchmark")
        {
            return false;
        }
        if render_demo || render_lifecycle {
            let mut host = IosCustomRenderHost::new(mtm);
            host.attach_to(
                &root_view,
                CGRect::new(
                    CGPoint::new(24.0, 96.0),
                    CGSize::new(bounds.size.width - 48.0, 240.0),
                ),
            );
            for frame_number in 0..=1 {
                let pixels = demo_pixels(frame_number);
                let frame = Rgba8Frame::new(64, 32, 64 * 4, &pixels, None)
                    .expect("fixed demo frame is valid");
                if host.present(&frame).is_err() {
                    return false;
                }
            }
            if render_lifecycle {
                host.set_frame(CGRect::new(
                    CGPoint::new(36.0, 108.0),
                    CGSize::new(bounds.size.width - 72.0, 208.0),
                ));
                let _geometry_after_resize = host.geometry();
                host.set_hidden(true);
                host.set_hidden(false);
                host.detach();
                drop(host);
            } else if self.ivars().render_host.set(host).is_err() {
                return false;
            }
            controller.setView(Some(&root_view));
            window.setRootViewController(Some(&controller));
            if self.ivars().window.set(window).is_err() {
                return false;
            }
            self.ivars()
                .window
                .get()
                .expect("window retained")
                .makeKeyAndVisible();
            return true;
        }

        let label = UILabel::new(mtm);
        label.setFrame(label_frame);
        // SAFETY: a non-null adaptive UIKit system color is valid for UILabel text.
        unsafe {
            label.setTextColor(Some(&UIColor::labelColor()));
        }
        let initial_text = NSString::from_str("Rust created this UIKit screen");
        label.setText(Some(&initial_text));

        let button = UIButton::new(mtm);
        button.setFrame(button_frame);
        let title = NSString::from_str("Call Rust");
        button.setTitle_forState(Some(&title), UIControlState::Normal);
        button.setTitleColor_forState(Some(&UIColor::systemBlueColor()), UIControlState::Normal);

        let render_host_frame = CGRect::new(
            CGPoint::new(24.0, 208.0),
            CGSize::new(bounds.size.width - 48.0, 176.0),
        );
        let mut render_host = IosCustomRenderHost::new(mtm);
        render_host.attach_to(&root_view, render_host_frame);
        let pixels = demo_pixels(0);
        let initial_frame =
            Rgba8Frame::new(64, 32, 64 * 4, &pixels, None).expect("fixed demo frame is valid");
        if render_host.present(&initial_frame).is_err() {
            return false;
        }

        let text_input = if exercise_text_input {
            let initial_text = String::from("Rust-owned text");
            let mut rust_text = initial_text.clone();
            let Ok(input) = IosCustomTextInput::attach(
                render_host.native_view(),
                CGRect::new(
                    CGPoint::new(8.0, 116.0),
                    CGSize::new(bounds.size.width - 64.0, 48.0),
                ),
                &initial_text,
                IosTextInputTraits::default(),
                move |event| {
                    if let Ok(snapshot) = event {
                        if snapshot.marked_text.is_none() {
                            rust_text.clone_from(&snapshot.text);
                        }
                    }
                },
            ) else {
                return false;
            };
            Some(input)
        } else {
            None
        };
        let target = ButtonTarget::with_state(mtm, label.clone(), render_host, text_input);
        // SAFETY: `target` is the retained Rust target object and implements the exact `buttonTapped:` selector.
        unsafe {
            button.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(buttonTapped:),
                UIControlEvents::TouchUpInside,
            );
        }

        root_view.addSubview(&label);
        root_view.addSubview(&button);
        controller.setView(Some(&root_view));
        window.setRootViewController(Some(&controller));

        if self.ivars().target.set(target).is_err() || self.ivars().window.set(window).is_err() {
            return false;
        }
        self.ivars()
            .window
            .get()
            .expect("window retained")
            .makeKeyAndVisible();
        if exercise_text_input
            && let Some(target) = self.ivars().target.get()
            && let Ok(mut input) = target.ivars().text_input.try_borrow_mut()
            && let Some(input) = input.as_mut()
        {
            let _ = input.request_focus();
        }
        if args
            .iter()
            .any(|arg| arg == "--exercise-rust-button-callback")
        {
            button.sendActionsForControlEvents(UIControlEvents::TouchUpInside);
        }
        true
    }

    #[cfg(feature = "custom-render-metal-demo")]
    fn present_custom_metal_demo(&self) {
        let mut host = self.ivars().metal_host.borrow_mut();
        let Some(host) = host.as_mut() else {
            return;
        };
        for frame_number in 0..=1 {
            let pixels = demo_bgra_pixels(frame_number);
            let frame = Bgra8Frame::new(64, 32, 64 * 4, &pixels, None)
                .expect("fixed Metal demo frame is valid");
            if host.present(&frame).is_err() {
                return;
            }
        }
    }

    #[cfg(feature = "custom-render-metal-demo")]
    fn run_custom_metal_benchmark(&self) {
        if self.ivars().metal_benchmark_ran.replace(true) {
            return;
        }
        let cpu_host_slot = self.ivars().metal_benchmark_cpu_host.borrow();
        let Some(cpu_host) = cpu_host_slot.as_ref() else {
            return;
        };
        let mut metal_slot = self.ivars().metal_host.borrow_mut();
        let Some(metal_host) = metal_slot.as_mut() else {
            return;
        };
        const WIDTH: u32 = 1179;
        const HEIGHT: u32 = 2556;
        const ROW_BYTES: u32 = WIDTH * 4;
        const SAMPLES: usize = 8;
        let mut rgba = vec![0u8; (ROW_BYTES * HEIGHT) as usize];
        let mut bgra = vec![0u8; rgba.len()];
        for (rgba_pixel, bgra_pixel) in rgba.chunks_exact_mut(4).zip(bgra.chunks_exact_mut(4)) {
            rgba_pixel.copy_from_slice(&[0x28, 0x80, 0xd0, 0xff]);
            bgra_pixel.copy_from_slice(&[0xd0, 0x80, 0x28, 0xff]);
        }
        let full_rgba = Rgba8Frame::new(WIDTH, HEIGHT, ROW_BYTES, &rgba, None)
            .expect("fixed full benchmark frame is valid");
        let full_bgra = Bgra8Frame::new(WIDTH, HEIGHT, ROW_BYTES, &bgra, None)
            .expect("fixed full benchmark frame is valid");
        if cpu_host.present(&full_rgba).is_err()
            || metal_host.present(&full_bgra).is_err()
            || metal_host.synchronize().is_err()
        {
            eprintln!("custom-render benchmark could not prime both presenters");
            return;
        }
        let mut report = format!(
            "{WIDTH}x{HEIGHT}, 2 warmups + {SAMPLES} samples/case; requested pacing is not display refresh or vsync\n"
        );
        println!("custom-render benchmark: {report}");
        for rate_hz in [30u64, 60, 120] {
            let cadence = Duration::from_micros(1_000_000 / rate_hz);
            for (damage_name, damage) in [
                ("full", None),
                (
                    "sparse-64x64",
                    Some(ios_ui::PixelDamageRect {
                        x: 0,
                        y: 0,
                        width: 64,
                        height: 64,
                    }),
                ),
            ] {
                for path in ["cpu", "metal"] {
                    let mut elapsed = Vec::with_capacity(SAMPLES);
                    let mut intervals = Vec::with_capacity(SAMPLES - 1);
                    let mut previous_start = None;
                    let mut next_start = Instant::now();
                    for sample in 0..(SAMPLES + 2) {
                        std::thread::sleep(next_start.saturating_duration_since(Instant::now()));
                        rgba[0] = sample as u8;
                        bgra[0] = sample as u8;
                        let rgba_frame = Rgba8Frame::new(WIDTH, HEIGHT, ROW_BYTES, &rgba, damage)
                            .expect("fixed CPU benchmark frame is valid");
                        let bgra_frame = Bgra8Frame::new(WIDTH, HEIGHT, ROW_BYTES, &bgra, damage)
                            .expect("fixed Metal benchmark frame is valid");
                        let start = Instant::now();
                        let result = match path {
                            "cpu" => cpu_host
                                .present(&rgba_frame)
                                .map_err(|error| error.to_string()),
                            _ => metal_host
                                .present(&bgra_frame)
                                .and_then(|()| metal_host.synchronize())
                                .map_err(|error| error.to_string()),
                        };
                        if let Err(error) = result {
                            eprintln!(
                                "custom-render benchmark stopped at {path}/{damage_name}/{rate_hz}Hz: {error}"
                            );
                            return;
                        }
                        if sample >= 2 {
                            elapsed.push(start.elapsed());
                            if let Some(previous) = previous_start {
                                intervals.push(start.duration_since(previous));
                            }
                            previous_start = Some(start);
                        }
                        next_start = start + cadence;
                    }
                    elapsed.sort_unstable();
                    let total_nanos: u128 = elapsed.iter().map(Duration::as_nanos).sum();
                    let mean_micros = total_nanos / SAMPLES as u128 / 1_000;
                    let p95_micros = elapsed[SAMPLES - 1].as_micros();
                    let mean_interval_micros =
                        intervals.iter().map(Duration::as_micros).sum::<u128>()
                            / intervals.len() as u128;
                    let observed_hz = 1_000_000 / mean_interval_micros.max(1);
                    let line = format!(
                        "path={path} damage={damage_name} requested_hz={rate_hz} observed_hz={observed_hz} mean_submit_us={mean_micros} p95_submit_us={p95_micros}\n"
                    );
                    println!("custom-render benchmark: {line}");
                    report.push_str(&line);
                }
            }
        }
        if let Err(error) = std::fs::write(
            std::env::temp_dir().join("custom-render-metal-benchmark.txt"),
            report,
        ) {
            eprintln!("custom-render benchmark could not save results: {error}");
        }
    }
}

fn demo_pixels(frame_number: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(64 * 32 * 4);
    for y in 0..32u32 {
        for x in 0..64u32 {
            let red = ((x * 3 + frame_number * 47) % 256) as u8;
            let green = ((y * 7 + frame_number * 29) % 256) as u8;
            let blue = ((x + y * 5 + frame_number * 61) % 256) as u8;
            pixels.extend_from_slice(&[red, green, blue, 255]);
        }
    }
    pixels
}

#[cfg(feature = "custom-render-metal-demo")]
fn demo_bgra_pixels(frame_number: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(64 * 32 * 4);
    for y in 0..32u32 {
        for x in 0..64u32 {
            let red = ((x * 3 + frame_number * 47) % 256) as u8;
            let green = ((y * 7 + frame_number * 29) % 256) as u8;
            let blue = ((x + y * 5 + frame_number * 61) % 256) as u8;
            pixels.extend_from_slice(&[blue, green, red, 255]);
        }
    }
    pixels
}

pub fn run() {
    let mtm = MainThread::current()
        .expect("UIApplication::main must start on the main thread")
        .into_objc2();
    let delegate_name = NSString::from_class(AppDelegate::class());
    UIApplication::main(None, Some(&delegate_name), mtm);
}

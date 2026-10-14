use core::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use framework_core::Error;
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{ClassType, DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_foundation::{NSDictionary, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UIApplicationLaunchOptionsKey, UIScene,
    UISceneConfiguration, UISceneConnectionOptions, UISceneDelegate, UISceneSession, UIWindowScene,
};

use crate::callback_queue::CallbackQueue;
use crate::{
    IosSceneHost, SceneLifecycleEventKind, SceneSessionIdentifier, copy_session_identifier,
};

type Handler = Box<dyn IosApplicationHandler>;

thread_local! {
    static HANDLER_BOOTSTRAP: RefCell<Option<Handler>> = const { RefCell::new(None) };
}

/// Receives application launch and per-scene lifecycle callbacks on UIKit's main thread.
///
/// A single handler instance is owned by the app delegate. Each `IosSceneHost` is separately
/// owned by its corresponding UIKit scene delegate. Callback panics are contained before they
/// can unwind through Objective-C; `panic=abort` still terminates the process.
pub trait IosApplicationHandler: 'static {
    /// Called after UIKit has completed application launch and before scene setup begins.
    fn application_did_finish_launching(&mut self) {}

    /// Called after each lifecycle transition for a connected window scene.
    fn scene_event(&mut self, _scene: &IosSceneHost, _event: SceneLifecycleEventKind) {}

    /// Called if UIKit connects a non-window scene or framework host construction fails.
    fn scene_creation_failed(
        &mut self,
        _session_identifier: &SceneSessionIdentifier,
        _error: Error,
    ) {
    }
}

/// Starts UIKit with Rust-defined application and scene delegates.
pub struct IosApplicationHost;

impl IosApplicationHost {
    /// Transfer `handler` to UIKit's app delegate and enter `UIApplicationMain`.
    ///
    /// Call on the process main thread with its `MainThread` proof. The handler is placed in a
    /// one-shot thread-local bootstrap slot and moved into the app delegate during UIKit's
    /// delegate initialization; no persistent process-global application state is used.
    pub fn run(handler: impl IosApplicationHandler, main_thread: MainThread) -> ! {
        HANDLER_BOOTSTRAP.with(|slot| {
            let mut slot = slot.borrow_mut();
            assert!(
                slot.is_none(),
                "UIKit app host can only start once per thread"
            );
            *slot = Some(Box::new(handler));
        });
        let _ = SceneDelegate::class();
        let delegate_name = NSString::from_class(ApplicationDelegate::class());
        UIApplication::main(None, Some(&delegate_name), main_thread.into_objc2())
    }
}

struct ApplicationState {
    handler: Option<Handler>,
    dispatching: bool,
    pending: CallbackQueue<PendingCallback>,
}

impl ApplicationState {
    fn new(handler: Option<Handler>) -> Self {
        Self {
            handler,
            dispatching: false,
            pending: CallbackQueue::new(),
        }
    }
}

enum PendingCallback {
    ApplicationDidFinishLaunching,
    Scene {
        delegate: Retained<SceneDelegate>,
        event: SceneLifecycleEventKind,
    },
    SceneCreationFailed {
        identifier: SceneSessionIdentifier,
        error: Error,
    },
}

impl PendingCallback {
    fn on_dropped(&self) {
        if let Self::Scene { delegate, event } = self {
            delegate.record_dropped_lifecycle_event();
            if *event == SceneLifecycleEventKind::DidDisconnect {
                delegate.request_cleanup();
            }
        }
    }

    fn cleanup_after_delivery(&self) {
        if let Self::Scene { delegate, event } = self {
            if *event == SceneLifecycleEventKind::DidDisconnect {
                delegate.request_cleanup();
            }
        }
    }
}

define_class!(
    // SAFETY:
    // - NSObject has no subclass initialization requirements.
    // - The main-thread-only delegate owns all mutable app state in Rust ivars.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosApplicationHostDelegate"]
    #[ivars = RefCell<ApplicationState>]
    struct ApplicationDelegate;

    impl ApplicationDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let handler = HANDLER_BOOTSTRAP.with(|slot| slot.borrow_mut().take());
            let this = this.set_ivars(RefCell::new(ApplicationState::new(handler)));
            // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(application:didFinishLaunchingWithOptions:))]
        unsafe fn did_finish_launching(
            &self,
            _application: &UIApplication,
            _options: Option<&NSDictionary<UIApplicationLaunchOptionsKey, AnyObject>>,
        ) -> bool {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.dispatch(PendingCallback::ApplicationDidFinishLaunching)
            }));
            true
        }

        #[unsafe(method_id(application:configurationForConnectingSceneSession:options:))]
        fn configuration_for_connecting_scene(
            &self,
            _application: &UIApplication,
            session: &UISceneSession,
            _options: &UISceneConnectionOptions,
        ) -> Retained<UISceneConfiguration> {
            let role = session.role();
            catch_unwind(AssertUnwindSafe(|| {
                let mtm = MainThread::current()
                    .expect("UIKit scene configuration callback must run on the main thread")
                    .into_objc2();
                let name = NSString::from_str("Rust Scene");
                let configuration = UISceneConfiguration::configurationWithName_sessionRole(
                    Some(&name),
                    &role,
                    mtm,
                );
                // SAFETY: SceneDelegate is an Objective-C class that implements UISceneDelegate
                // and is registered before this public UIKit configuration is returned.
                unsafe { configuration.setDelegateClass(Some(SceneDelegate::class())) };
                configuration
            }))
            .unwrap_or_else(|_| {
                let mtm = MainThread::current()
                    .expect("UIKit scene configuration callback must run on the main thread")
                    .into_objc2();
                UISceneConfiguration::configurationWithName_sessionRole(None, &role, mtm)
            })
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for ApplicationDelegate {}
    // SAFETY: the class implements the generated public UIApplicationDelegate method signatures.
    unsafe impl UIApplicationDelegate for ApplicationDelegate {}
);

impl ApplicationDelegate {
    fn dispatch(&self, callback: PendingCallback) {
        let handler = {
            let mut state = self.ivars().borrow_mut();
            if state.dispatching {
                let dropped = state.pending.push(callback);
                if let Some(dropped) = dropped {
                    dropped.on_dropped();
                }
                return;
            }
            let Some(handler) = state.handler.take() else {
                callback.on_dropped();
                return;
            };
            state.dispatching = true;
            handler
        };
        self.dispatch_queue(handler, callback);
    }

    fn dispatch_queue(&self, mut handler: Handler, mut callback: PendingCallback) {
        loop {
            let _ = catch_unwind(AssertUnwindSafe(|| match &callback {
                PendingCallback::ApplicationDidFinishLaunching => {
                    handler.application_did_finish_launching();
                }
                PendingCallback::Scene { delegate, event } => {
                    delegate.deliver(&mut *handler, *event);
                }
                PendingCallback::SceneCreationFailed { identifier, error } => {
                    handler.scene_creation_failed(identifier, *error);
                }
            }));
            callback.cleanup_after_delivery();

            let next = {
                let mut state = self.ivars().borrow_mut();
                state.handler = Some(handler);
                match state.pending.pop() {
                    Some(next) => state.handler.take().map(|handler| (handler, next)),
                    None => {
                        state.dispatching = false;
                        None
                    }
                }
            };
            let Some((next_handler, next_callback)) = next else {
                break;
            };
            handler = next_handler;
            callback = next_callback;
        }
    }
}

struct SceneDelegateState {
    host: RefCell<Option<IosSceneHost>>,
    cleanup_pending: Cell<bool>,
}

impl SceneDelegateState {
    fn new() -> Self {
        Self {
            host: RefCell::new(None),
            cleanup_pending: Cell::new(false),
        }
    }
}

define_class!(
    // SAFETY:
    // - NSObject has no subclass initialization requirements.
    // - Each UIKit scene delegate owns exactly one main-thread-bound scene host.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosApplicationSceneDelegate"]
    #[ivars = SceneDelegateState]
    struct SceneDelegate;

    impl SceneDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(SceneDelegateState::new());
            // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(scene:willConnectToSession:options:))]
        fn will_connect(
            &self,
            scene: &UIScene,
            session: &UISceneSession,
            _options: &UISceneConnectionOptions,
        ) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let identifier = copy_session_identifier(&session.persistentIdentifier());
                let Some(window_scene) = scene.downcast_ref::<UIWindowScene>() else {
                    Self::dispatch_creation_failure(identifier, Error::new(framework_core::ErrorKind::Unsupported));
                    return;
                };
                if self.ivars().host.borrow().is_some() {
                    return;
                }
                let Some(main_thread) = MainThread::current() else {
                    Self::dispatch_creation_failure(identifier, Error::new(framework_core::ErrorKind::Unavailable));
                    return;
                };
                match IosSceneHost::new(main_thread, window_scene) {
                    Ok(host) => {
                        *self.ivars().host.borrow_mut() = Some(host);
                        self.dispatch_scene_event(SceneLifecycleEventKind::WillConnect);
                    }
                    Err(error) => Self::dispatch_creation_failure(identifier, error),
                }
            }));
        }

        #[unsafe(method(sceneDidDisconnect:))]
        fn did_disconnect(&self, _scene: &UIScene) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.dispatch_scene_event(SceneLifecycleEventKind::DidDisconnect)
            }));
        }

        #[unsafe(method(sceneDidBecomeActive:))]
        fn did_become_active(&self, _scene: &UIScene) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.dispatch_scene_event(SceneLifecycleEventKind::DidActivate)
            }));
        }

        #[unsafe(method(sceneWillResignActive:))]
        fn will_resign_active(&self, _scene: &UIScene) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.dispatch_scene_event(SceneLifecycleEventKind::WillDeactivate)
            }));
        }

        #[unsafe(method(sceneWillEnterForeground:))]
        fn will_enter_foreground(&self, _scene: &UIScene) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.dispatch_scene_event(SceneLifecycleEventKind::WillEnterForeground)
            }));
        }

        #[unsafe(method(sceneDidEnterBackground:))]
        fn did_enter_background(&self, _scene: &UIScene) {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.dispatch_scene_event(SceneLifecycleEventKind::DidEnterBackground)
            }));
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for SceneDelegate {}
    // SAFETY: the class implements the generated public UISceneDelegate method signatures.
    unsafe impl UISceneDelegate for SceneDelegate {}
);

impl SceneDelegate {
    fn retain_for_callback(&self) -> Retained<Self> {
        let object = self as *const Self as *mut Self;
        // SAFETY: UIKit keeps a strong reference to this delegate throughout every delegate call;
        // retain increments that live object's ownership before the callback returns.
        unsafe { Retained::retain(object) }.expect("live scene delegate must be retainable")
    }

    fn dispatch_scene_event(&self, event: SceneLifecycleEventKind) {
        let Some(delegate) = application_delegate() else {
            if event == SceneLifecycleEventKind::DidDisconnect {
                self.request_cleanup();
            }
            return;
        };
        delegate.dispatch(PendingCallback::Scene {
            delegate: self.retain_for_callback(),
            event,
        });
    }

    fn dispatch_creation_failure(identifier: SceneSessionIdentifier, error: Error) {
        if let Some(delegate) = application_delegate() {
            delegate.dispatch(PendingCallback::SceneCreationFailed { identifier, error });
        }
    }

    fn deliver(&self, handler: &mut dyn IosApplicationHandler, event: SceneLifecycleEventKind) {
        let host = self.ivars().host.borrow();
        if let Some(host) = host.as_ref() {
            host.update_phase(event);
            handler.scene_event(host, event);
        }
        drop(host);
        self.finish_pending_cleanup();
    }

    fn request_cleanup(&self) {
        match self.ivars().host.try_borrow_mut() {
            Ok(mut host) => {
                host.take();
                self.ivars().cleanup_pending.set(false);
            }
            Err(_) => self.ivars().cleanup_pending.set(true),
        }
    }

    fn finish_pending_cleanup(&self) {
        if self.ivars().cleanup_pending.replace(false) {
            self.request_cleanup();
        }
    }

    fn record_dropped_lifecycle_event(&self) {
        if let Some(host) = self.ivars().host.borrow().as_ref() {
            host.record_dropped_lifecycle_event();
        }
    }
}

fn application_delegate() -> Option<Retained<ApplicationDelegate>> {
    let main_thread = MainThread::current()?;
    let application = UIApplication::sharedApplication(main_thread.into_objc2());
    // SAFETY: UIKit retains the application delegate for the lifetime of the application. The
    // retained protocol object returned here keeps it alive while it is downcast and used.
    let delegate = unsafe { application.delegate()? };
    let delegate_ref: &ProtocolObject<dyn UIApplicationDelegate> = delegate.as_ref();
    let object: &AnyObject = delegate_ref.as_ref();
    let typed = object.downcast_ref::<ApplicationDelegate>()?;
    let object = typed as *const ApplicationDelegate as *mut ApplicationDelegate;
    // SAFETY: the protocol object above strongly retains this delegate while the pointer is
    // checked; taking another retain gives the returned handle an independent valid lifetime.
    unsafe { Retained::retain(object) }
}

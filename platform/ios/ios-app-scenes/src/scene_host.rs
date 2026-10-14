use core::cell::Cell;
use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use ios_runtime::main_thread::MainThread;
use objc2::rc::Retained;
use objc2_ui_kit::{UIView, UIViewController, UIWindow, UIWindowScene};

pub use crate::scene_phase::IosScenePhase;
use crate::{
    IosSceneWindow, SceneLifecycleEventKind, SceneSessionIdentifier, copy_session_identifier,
};

/// A main-thread-owned UIKit window, root controller and scene identity.
///
/// UIKit owns each attached child view after `attach_view` succeeds. The host owns the window,
/// scene and root controller, and hides the window and clears its root controller on drop. This
/// handle is neither `Send` nor `Sync`.
pub struct IosSceneHost {
    window: IosSceneWindow,
    scene: Retained<UIWindowScene>,
    root_view_controller: Retained<UIViewController>,
    session_identifier: SceneSessionIdentifier,
    phase: Cell<IosScenePhase>,
    dropped_lifecycle_events: Cell<u64>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosSceneHost {
    pub(crate) fn new(main_thread: MainThread, scene: &UIWindowScene) -> Result<Self> {
        let root_view_controller = UIViewController::new(main_thread.into_objc2());
        root_view_controller
            .view()
            .ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        let window_main_thread =
            MainThread::current().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        let window = IosSceneWindow::new(window_main_thread, scene, &root_view_controller)?;
        let scene_ptr = scene as *const UIWindowScene as *mut UIWindowScene;
        // SAFETY: `scene` is a live Objective-C object borrowed from UIKit's scene callback;
        // retaining it extends that valid object lifetime for this host.
        let scene = unsafe { Retained::retain(scene_ptr) }
            .ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        let session_identifier = copy_session_identifier(&scene.session().persistentIdentifier());
        Ok(Self {
            window,
            scene,
            root_view_controller,
            session_identifier,
            phase: Cell::new(IosScenePhase::Connecting),
            dropped_lifecycle_events: Cell::new(0),
            _not_send: PhantomData,
        })
    }

    /// Return the exact UTF-16 session identity supplied by UIKit.
    pub const fn session_identifier(&self) -> &SceneSessionIdentifier {
        &self.session_identifier
    }

    /// Return the last lifecycle phase delivered to this scene delegate.
    pub fn phase(&self) -> IosScenePhase {
        self.phase.get()
    }

    /// Return the number of reentrant lifecycle callbacks dropped for this scene.
    pub fn dropped_lifecycle_event_count(&self) -> u64 {
        self.dropped_lifecycle_events.get()
    }

    /// Borrow the native window scene for capability-scoped UIKit operations.
    pub fn native_scene(&self) -> &UIWindowScene {
        &self.scene
    }

    /// Borrow the root controller used to present native UIKit controllers.
    pub fn root_view_controller(&self) -> &UIViewController {
        &self.root_view_controller
    }

    /// Return the loaded root view retained by its view controller.
    pub fn root_view(&self) -> Result<Retained<UIView>> {
        self.root_view_controller
            .viewIfLoaded()
            .ok_or_else(|| Error::new(ErrorKind::Unavailable))
    }

    /// Borrow the window managed by this scene host.
    pub fn native_window(&self) -> &UIWindow {
        self.window.native_window()
    }

    /// Attach an unparented native child view to the root view.
    ///
    /// Returns `InvalidInput` when the view already has a superview or belongs to a window. UIKit
    /// retains the view through the native view hierarchy after attachment.
    pub fn attach_view(&self, view: &UIView) -> Result<()> {
        let root = self.root_view()?;
        if view.superview().is_some()
            || view.window().is_some()
            || core::ptr::eq(view as *const UIView, root.as_ref() as *const UIView)
        {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        root.addSubview(view);
        Ok(())
    }

    /// Detach a child view only when it is a direct child of this host's root view.
    pub fn detach_view(&self, view: &UIView) -> Result<()> {
        let root = self.root_view()?;
        let Some(parent) = view.superview() else {
            return Err(Error::new(ErrorKind::InvalidInput));
        };
        if !core::ptr::eq(
            parent.as_ref() as *const UIView,
            root.as_ref() as *const UIView,
        ) {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        view.removeFromSuperview();
        Ok(())
    }

    pub(crate) fn update_phase(&self, event: SceneLifecycleEventKind) {
        self.phase.set(crate::scene_phase::phase_for_event(event));
    }

    pub(crate) fn record_dropped_lifecycle_event(&self) {
        self.dropped_lifecycle_events
            .set(self.dropped_lifecycle_events.get().saturating_add(1));
    }
}

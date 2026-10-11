use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;
use std::vec::Vec;

use block2::RcBlock;
use framework_core::{Error, ErrorKind, Result};
use ios_runtime::ffi::catch_unwind;
use objc2::rc::{Retained, Weak, autoreleasepool};
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send};
use objc2_foundation::{NSArray, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIAction, UIContextMenuConfiguration, UIContextMenuInteraction,
    UIContextMenuInteractionDelegate, UIMenu, UIMenuElement, UIView,
};

pub use crate::context_menu_policy::IosContextMenuAction;
use crate::context_menu_policy::{ContextMenuCallback, invoke_action_callback, validate_actions};

use super::{IosUiBackend, main_thread_marker};

struct ContextMenuTargetState {
    actions: Vec<IosContextMenuAction>,
    callback: RefCell<ContextMenuCallback>,
    active: Cell<bool>,
    menu: RefCell<Option<Retained<UIMenu>>>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ContextMenuTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiContextMenuTarget"]
    #[ivars = ContextMenuTargetState]
    struct ContextMenuTarget;

    // SAFETY: The delegate is confined to the main thread and the callback returns a retained
    // configuration built from the installed owned action snapshot.
    unsafe impl UIContextMenuInteractionDelegate for ContextMenuTarget {
        #[allow(non_snake_case)]
        #[unsafe(method_id(contextMenuInteraction:configurationForMenuAtLocation:))]
        fn contextMenuInteraction_configurationForMenuAtLocation(
            &self,
            _interaction: &UIContextMenuInteraction,
            _location: objc2_core_foundation::CGPoint,
        ) -> Option<Retained<UIContextMenuConfiguration>> {
            let _keep_alive = self.retain();
            catch_unwind(|| self.configuration()).ok().flatten()
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for ContextMenuTarget {}
);

impl ContextMenuTarget {
    fn with_state(mtm: MainThreadMarker, state: ContextMenuTargetState) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(state);
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }

    fn build_menu(&self, mtm: MainThreadMarker) -> Result<Retained<UIMenu>> {
        let mut elements: Vec<Retained<UIMenuElement>> = Vec::new();
        elements
            .try_reserve_exact(self.ivars().actions.len())
            .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
        for action in &self.ivars().actions {
            let weak_target = Weak::from(self);
            let action_id = action.id;
            let handler: RcBlock<dyn Fn(NonNull<UIAction>)> = RcBlock::new(move |_selected| {
                if let Some(target) = weak_target.load() {
                    target.invoke_action(action_id);
                }
            });
            let title = NSString::from_str(&action.title);
            // SAFETY: `handler` is a valid block pointer for this call; UIKit copies and retains
            // UIAction's handler. The interaction is only attached after the iOS 13 check.
            let native_action = unsafe {
                UIAction::actionWithTitle_image_identifier_handler(
                    &title,
                    None,
                    None,
                    RcBlock::as_ptr(&handler),
                    mtm,
                )
            };
            elements.push(native_action.into_super());
        }
        let children = NSArray::from_retained_slice(&elements);
        let title = NSString::from_str("");
        Ok(UIMenu::menuWithTitle_children(&title, &children, mtm))
    }

    fn set_menu(&self, menu: Retained<UIMenu>) -> Result<()> {
        let mut slot = self
            .ivars()
            .menu
            .try_borrow_mut()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?;
        *slot = Some(menu);
        Ok(())
    }

    fn configuration(&self) -> Option<Retained<UIContextMenuConfiguration>> {
        if !self.ivars().active.get() {
            return None;
        }
        let marker = main_thread_marker().ok()?;
        let menu = self.ivars().menu.try_borrow().ok()?.as_ref()?.clone();
        autoreleasepool(|_| {
            let action_provider: RcBlock<dyn Fn(NonNull<NSArray<UIMenuElement>>) -> *mut UIMenu> =
                RcBlock::new(move |_suggested_actions| Retained::as_ptr(&menu) as *mut UIMenu);
            // SAFETY: the provider is a valid block pointer for this call, and UIKit copies and
            // retains it in the returned configuration. It returns the strongly captured menu.
            Some(unsafe {
                UIContextMenuConfiguration::configurationWithIdentifier_previewProvider_actionProvider(
                    None,
                    core::ptr::null_mut(),
                    RcBlock::as_ptr(&action_provider),
                    marker,
                )
            })
        })
    }

    fn invoke_action(&self, id: u64) {
        if self.ivars().active.get() {
            invoke_action_callback(&self.ivars().callback, id);
        }
    }
}

/// A fixed UIKit context menu attached to a caller-owned native view.
///
/// Keep the handle alive while the menu and its selection callback should remain active. Dropping
/// it disables future selections and removes its exact `UIContextMenuInteraction` from the parent
/// when that parent is still alive. Actions are an owned snapshot with unique IDs; this handle does
/// not provide dynamic updates, custom previews, or declarative state observation.
pub struct IosContextMenu {
    parent: Weak<UIView>,
    native: Retained<UIContextMenuInteraction>,
    _target: Retained<ContextMenuTarget>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Attaches a fixed native context menu to a caller-owned `UIView`.
    ///
    /// Action IDs must be distinct and titles nonempty. The callback receives the selected action
    /// ID. Context menus require iOS 13.0; on older systems this returns
    /// [`ErrorKind::Unavailable`] before creating or attaching a UIKit interaction. Keep the
    /// returned handle alive while the menu should remain active.
    pub fn add_context_menu<F>(
        &mut self,
        parent: &UIView,
        actions: Vec<IosContextMenuAction>,
        selection: F,
    ) -> Result<IosContextMenu>
    where
        F: FnMut(u64) + 'static,
    {
        let marker = main_thread_marker()?;
        if !objc2::available!(ios = 13.0, ..) {
            return Err(Error::new(ErrorKind::Unavailable));
        }
        validate_actions(&actions)?;

        let target = ContextMenuTarget::with_state(
            marker,
            ContextMenuTargetState {
                actions,
                callback: RefCell::new(Box::new(selection)),
                active: Cell::new(true),
                menu: RefCell::new(None),
            },
        );
        let menu = autoreleasepool(|_| target.build_menu(marker))?;
        target.set_menu(menu)?;
        let delegate = ProtocolObject::from_ref(&*target);
        let native = UIContextMenuInteraction::initWithDelegate(
            UIContextMenuInteraction::alloc(marker),
            delegate,
        );
        let interaction = ProtocolObject::from_ref(&*native);
        parent.addInteraction(interaction);
        Ok(IosContextMenu {
            parent: Weak::from(parent),
            native,
            _target: target,
            _not_send: PhantomData,
        })
    }
}

impl IosContextMenu {
    /// Borrows the native context-menu interaction for UIKit configuration.
    pub fn native_interaction(&self) -> &UIContextMenuInteraction {
        &self.native
    }
}

impl Drop for IosContextMenu {
    fn drop(&mut self) {
        self._target.ivars().active.set(false);
        if main_thread_marker().is_ok() {
            if let Some(parent) = self.parent.load() {
                let interaction = ProtocolObject::from_ref(&*self.native);
                parent.removeInteraction(interaction);
            }
        }
    }
}

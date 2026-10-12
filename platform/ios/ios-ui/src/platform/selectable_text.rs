use core::cell::RefCell;
use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::Result;
use framework_ui::Frame;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send};
use objc2_foundation::{NSObjectProtocol, NSString};
use objc2_ui_kit::{UIScrollViewDelegate, UITextView, UITextViewDelegate};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame, native_string_to_rust};
use crate::text_policy::{
    TextEditorCallback, invoke_text_editor_callback, replace_text_editor_callback,
};

struct TextEditorTargetState {
    callback: RefCell<TextEditorCallback>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - TextEditorTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiTextEditorTarget"]
    #[ivars = TextEditorTargetState]
    struct TextEditorTarget;

    // SAFETY: UITextView does not need optional scroll callbacks for this edit action.
    unsafe impl UIScrollViewDelegate for TextEditorTarget {}

    // SAFETY: The editor target is main-thread-bound and the handle retains it while installed.
    #[allow(non_snake_case)]
    unsafe impl UITextViewDelegate for TextEditorTarget {
        #[unsafe(method(textViewDidChange:))]
        fn textViewDidChange(&self, text_view: &UITextView) {
            let _keep_alive = self.retain();
            let text = native_string_to_rust(&text_view.text());
            invoke_text_editor_callback(&self.ivars().callback, text);
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for TextEditorTarget {}
);

impl TextEditorTarget {
    fn with_action(mtm: MainThreadMarker, action: TextEditorCallback) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TextEditorTargetState {
            callback: RefCell::new(action),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

/// A retained, read-only, selectable multiline UIKit text view.
///
/// The handle is main-thread-bound. UIKit may retain the visual view after this handle drops.
pub struct IosSelectableTextView {
    native: Retained<UITextView>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained, editable multiline UIKit text view.
///
/// The handle and edit callback are main-thread-bound. Keep the handle alive while its callback
/// should remain active; dropping it detaches its delegate even if UIKit retains the visual view.
pub struct IosTextEditor {
    native: Retained<UITextView>,
    target: Retained<TextEditorTarget>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds a read-only, selectable multiline text view to `parent`.
    ///
    /// The frame and text are caller-owned inputs. The returned handle owns the `UITextView`;
    /// UIKit may retain the visual view through its parent after the handle drops. Text selection
    /// uses UIKit's native selection behavior; app-specific copy actions and content policy remain
    /// with the caller.
    pub fn add_selectable_text_view(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        text: &str,
    ) -> Result<IosSelectableTextView> {
        let marker = main_thread_marker()?;
        let native = UITextView::new(marker);
        native.setFrame(native_frame(frame));
        native.setEditable(false);
        native.setSelectable(true);
        native.setScrollEnabled(true);
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            native.setText(Some(&text));
        });
        parent.native.addSubview(&native);
        Ok(IosSelectableTextView {
            native,
            _not_send: PhantomData,
        })
    }
}

impl IosSelectableTextView {
    /// Returns the current native text as a strict UTF-16 snapshot.
    ///
    /// Embedded NUL and supplementary Unicode scalars are preserved; malformed UTF-16 returns
    /// `InvalidInput` rather than replacement characters.
    pub fn text(&self) -> Result<String> {
        let _text_view_read_thread = main_thread_marker()?;
        let text = self.native.text();
        native_string_to_rust(&text)
    }

    /// Replaces the native text without enabling editing.
    ///
    /// An empty string clears the text. Rust `str` input is encoded with its explicit length,
    /// preserving embedded NUL characters.
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        let _text_view_write_thread = main_thread_marker()?;
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            self.native.setText(Some(&text));
        });
        Ok(())
    }

    /// Borrows the native text view for caller-owned UIKit integration.
    pub fn native_text_view(&self) -> &UITextView {
        &self.native
    }
}

impl IosUiBackend {
    /// Adds an editable, selectable multiline text view to `parent`.
    ///
    /// The initial text is copied exactly, including embedded NUL characters. The callback
    /// receives a fresh strict UTF-16 snapshot after UIKit reports a user/native edit. Panics are
    /// contained and reentrant invocation is ignored. Programmatic `set_text` calls do not emit
    /// edit events.
    pub fn add_text_editor<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        text: &str,
        action: F,
    ) -> Result<IosTextEditor>
    where
        F: FnMut(Result<String>) + 'static,
    {
        let marker = main_thread_marker()?;
        let native = UITextView::new(marker);
        native.setFrame(native_frame(frame));
        native.setEditable(true);
        native.setSelectable(true);
        native.setScrollEnabled(true);
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            native.setText(Some(&text));
        });
        let target = TextEditorTarget::with_action(marker, Box::new(action));
        // SAFETY: `target` is retained for the editor lifetime, both objects are main-thread-only,
        // and UITextView stores its delegate weakly. Drop detaches this exact delegate first.
        unsafe { native.setDelegate(Some(ProtocolObject::from_ref(&*target))) };
        parent.native.addSubview(&native);
        Ok(IosTextEditor {
            native,
            target,
            _not_send: PhantomData,
        })
    }
}

impl IosTextEditor {
    /// Returns a fresh strict UTF-16 snapshot of the current editor text.
    ///
    /// Embedded NUL and supplementary Unicode scalars are preserved; malformed UTF-16 returns
    /// `InvalidInput` rather than replacement characters.
    pub fn text(&self) -> Result<String> {
        let _text_editor_read_thread = main_thread_marker()?;
        native_string_to_rust(&self.native.text())
    }

    /// Replaces the editor text without emitting a user-edit callback.
    ///
    /// Rust `str` input is encoded with its explicit length, preserving embedded NUL characters.
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        let _text_editor_write_thread = main_thread_marker()?;
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            self.native.setText(Some(&text));
        });
        Ok(())
    }

    /// Replaces the callback invoked after a UIKit text edit.
    ///
    /// The callback runs synchronously on the main thread. Replacing it while it is active returns
    /// `Unavailable` and leaves the existing callback unchanged.
    pub fn set_edit_action<F>(&mut self, action: F) -> Result<()>
    where
        F: FnMut(Result<String>) + 'static,
    {
        let _text_editor_action_thread = main_thread_marker()?;
        replace_text_editor_callback(&self.target.ivars().callback, Box::new(action))
    }

    /// Requests first-responder status and returns UIKit's result.
    ///
    /// The returned value does not establish that the software keyboard became visible.
    pub fn request_first_responder(&mut self) -> Result<bool> {
        let _text_editor_focus_thread = main_thread_marker()?;
        Ok(self.native.becomeFirstResponder())
    }

    /// Requests that the editor resign first-responder status and returns UIKit's result.
    pub fn resign_first_responder(&mut self) -> Result<bool> {
        let _text_editor_resign_thread = main_thread_marker()?;
        Ok(self.native.resignFirstResponder())
    }

    /// Borrows the native text view for caller-owned UIKit integration.
    ///
    /// The editor owns the text-view delegate while this handle is alive.
    pub fn native_text_view(&self) -> &UITextView {
        &self.native
    }
}

impl Drop for IosTextEditor {
    fn drop(&mut self) {
        let current = self.native.delegate();
        let editor_delegate = ProtocolObject::from_ref(&*self.target);
        if current
            .as_deref()
            .is_some_and(|delegate| core::ptr::eq(delegate, editor_delegate))
        {
            // SAFETY: The handle is main-thread-bound; this removes only its own delegate before
            // releasing the retained target. A replacement delegate is left untouched.
            unsafe { self.native.setDelegate(None) };
        }
    }
}

use core::cell::RefCell;
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use framework_ui::{
    ButtonControl, Frame, LabelControl, TextFieldControl, TextFieldInputMode, ToggleControl,
    UiBackend,
};
use ios_runtime::ffi::catch_unwind;
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObjectProtocol, NSRange, NSString};
use objc2_ui_kit::{
    UIButton, UIControlEvents, UIControlState, UIImage, UIImageView, UIKeyboardType, UILabel,
    UILayoutConstraintAxis, UIScrollView, UIStackView, UISwitch, UITextField, UITextInputTraits,
    UIView,
};

use crate::scroll_policy::validated_content_size;
use crate::stack_spacing::is_valid_stack_spacing;
use crate::text_conversion::{Utf16Error, decode_utf16};
use crate::text_policy::{KeyboardHint, TextFieldCallback, input_traits, invoke_text_callback};
use crate::toggle_policy::invoke_toggle_callback;

/// A UIKit backend for caller-owned native controls.
///
/// Construct it on the main thread. It owns no window, controller, scene, or global service.
pub struct IosUiBackend {
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit container view.
///
/// The caller owns its hierarchy placement and lifetime. Use [`Self::native_view`] to attach it
/// to a caller-owned `UIViewController` or another native parent.
pub struct IosView {
    native: Retained<UIView>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit text label.
pub struct IosLabel {
    native: Retained<UILabel>,
    _not_send: PhantomData<Rc<()>>,
}

/// A UIKit image-view handle with one native `UIImageView`.
pub struct IosImageView {
    native: Retained<UIImageView>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit single-line text field.
pub struct IosTextField {
    native: Retained<UITextField>,
    _target: Option<Retained<TextFieldTarget>>,
    _text_field_not_send: PhantomData<Rc<()>>,
}

/// An iOS-native arranged-subview container backed by UIKit's `UIStackView`.
pub struct IosStackView {
    native: Retained<UIStackView>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit scroll view with an owned content host.
pub struct IosScrollView {
    native: Retained<UIScrollView>,
    content: IosView,
    _not_send: PhantomData<Rc<()>>,
}

/// The axis UIKit uses to arrange an `IosStackView`'s children.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosStackAxis {
    /// Arrange children left to right.
    Horizontal,
    /// Arrange children top to bottom.
    Vertical,
}

/// A retained UIKit button and its Rust target/action callback owner.
///
/// Keep this handle alive while the button action should remain active. The native parent view may
/// keep the visual `UIButton` alive after this handle drops, but this handle is the Rust owner of
/// the callback target.
pub struct IosButton {
    native: Retained<UIButton>,
    _target: Retained<ButtonTarget>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit switch and its Rust `ValueChanged` callback owner.
///
/// Keep this handle alive while the callback should remain active. The parent may keep displaying
/// the native `UISwitch` after this handle drops; dropping this handle unregisters its exact
/// target/action before releasing the callback.
pub struct IosToggle {
    native: Retained<UISwitch>,
    _target: Retained<ToggleTarget>,
    _not_send: PhantomData<Rc<()>>,
}

struct ButtonTargetState {
    action: RefCell<Box<dyn FnMut()>>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ButtonTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiButtonTarget"]
    #[ivars = ButtonTargetState]
    struct ButtonTarget;

    impl ButtonTarget {
        #[unsafe(method(buttonTapped:))]
        fn button_tapped(&self, _sender: &UIButton) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| {
                if let Ok(mut action) = self.ivars().action.try_borrow_mut() {
                    action();
                }
            });
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for ButtonTarget {}
);

impl ButtonTarget {
    fn with_action(mtm: MainThreadMarker, action: Box<dyn FnMut()>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ButtonTargetState {
            action: RefCell::new(action),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

struct ToggleTargetState {
    callback: RefCell<Box<dyn FnMut(bool)>>,
}

struct TextFieldTargetState {
    callback: RefCell<TextFieldCallback>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - TextFieldTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiTextFieldTarget"]
    #[ivars = TextFieldTargetState]
    struct TextFieldTarget;

    impl TextFieldTarget {
        #[unsafe(method(textFieldChanged:))]
        fn text_field_changed(&self, sender: &UITextField) {
            let _keep_alive = self.retain();
            let text = sender.text().as_deref().map(native_string_to_rust).transpose();
            invoke_text_callback(&self.ivars().callback, text);
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for TextFieldTarget {}
);

impl TextFieldTarget {
    fn with_action(mtm: MainThreadMarker, action: TextFieldCallback) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TextFieldTargetState {
            callback: RefCell::new(action),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ToggleTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiToggleTarget"]
    #[ivars = ToggleTargetState]
    struct ToggleTarget;

    impl ToggleTarget {
        #[unsafe(method(toggleChanged:))]
        fn toggle_changed(&self, sender: &UISwitch) {
            let _keep_alive = self.retain();
            let on = sender.isOn();
            invoke_toggle_callback(&self.ivars().callback, on);
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for ToggleTarget {}
);

impl ToggleTarget {
    fn with_action(mtm: MainThreadMarker, action: Box<dyn FnMut(bool)>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ToggleTargetState {
            callback: RefCell::new(action),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

impl IosUiBackend {
    /// Creates a backend from a typed proof that the current call is on UIKit's main thread.
    pub fn new(main_thread: MainThread) -> Self {
        Self {
            _main_thread: main_thread,
            _not_send: PhantomData,
        }
    }

    /// Adds a UIKit stack view to a caller-owned container.
    ///
    /// UIKit arranges the stack's child views along `axis` with `spacing` points between them.
    /// Any finite spacing is passed to UIKit; negative values can overlap arranged children. This
    /// configures only the stack's own frame and arranged children; the caller remains responsible
    /// for placing or resizing the stack in its parent hierarchy.
    pub fn add_stack_view(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        axis: IosStackAxis,
        spacing: f64,
    ) -> Result<IosStackView> {
        if !is_valid_stack_spacing(spacing) {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        let marker = main_thread_marker()?;
        let native = UIStackView::new(marker);
        native.setFrame(native_frame(frame));
        native.setAxis(match axis {
            IosStackAxis::Horizontal => UILayoutConstraintAxis::Horizontal,
            IosStackAxis::Vertical => UILayoutConstraintAxis::Vertical,
        });
        native.setSpacing(spacing as _);
        parent.native.addSubview(&native);
        Ok(IosStackView {
            native,
            _not_send: PhantomData,
        })
    }

    /// Adds a `UIScrollView` and zero-origin content host to `parent`.
    ///
    /// Add controls to [`IosScrollView::content_view_mut`] with the existing backend methods, then
    /// set the total content size. Child placement and content size remain caller-owned.
    pub fn add_scroll_view(&mut self, parent: &mut IosView, frame: Frame) -> Result<IosScrollView> {
        let marker = main_thread_marker()?;
        let native = UIScrollView::new(marker);
        native.setFrame(native_frame(frame));
        let content = UIView::new(marker);
        content.setFrame(CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(0.0, 0.0)));
        native.addSubview(&content);
        parent.native.addSubview(&native);
        Ok(IosScrollView {
            native,
            content: IosView {
                native: content,
                _not_send: PhantomData,
            },
            _not_send: PhantomData,
        })
    }

    /// Adds a native image view to `parent` with an optional app `UIImage`.
    ///
    /// The image reference lasts only for this call. `UIImageView.image` owns its non-null value.
    /// The return handle owns the `UIImageView` and has main-thread affinity.
    pub fn add_image_view(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        image: Option<&UIImage>,
    ) -> Result<IosImageView> {
        let marker = main_thread_marker()?;
        let native = UIImageView::new(marker);
        native.setFrame(native_frame(frame));
        native.setImage(image);
        parent.native.addSubview(&native);
        Ok(IosImageView {
            native,
            _not_send: PhantomData,
        })
    }

    /// Adds a native text field to `parent` with optional text and placeholder values.
    ///
    /// `None` clears a native value; `Some("")` sets an empty string. Text values use explicit
    /// lengths and preserve embedded NUL characters.
    pub fn add_text_field(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        text: Option<&str>,
        placeholder: Option<&str>,
    ) -> Result<IosTextField> {
        let field_marker = main_thread_marker()?;
        let native = UITextField::new(field_marker);
        native.setFrame(native_frame(frame));
        autoreleasepool(|_| {
            let text = text.map(NSString::from_str);
            native.setText(text.as_deref());
            let placeholder = placeholder.map(NSString::from_str);
            native.setPlaceholder(placeholder.as_deref());
        });
        parent.native.addSubview(&native);
        Ok(IosTextField {
            native,
            _target: None,
            _text_field_not_send: PhantomData,
        })
    }
}

impl IosView {
    /// Borrows the native container view for UIKit integration.
    pub fn native_view(&self) -> &UIView {
        &self.native
    }
}

impl IosLabel {
    /// Borrows the native label for UIKit integration.
    pub fn native_label(&self) -> &UILabel {
        &self.native
    }
}

impl IosImageView {
    /// Sets or clears the image on this view.
    ///
    /// `UIImageView.image` owns each non-null image value.
    pub fn set_image(&mut self, image: Option<&UIImage>) -> Result<()> {
        let _image_thread = main_thread_marker()?;
        self.native.setImage(image);
        Ok(())
    }

    /// Clears the image.
    pub fn clear_image(&mut self) -> Result<()> {
        self.set_image(None)
    }

    /// Borrows the native image view for UIKit use.
    pub fn native_image_view(&self) -> &UIImageView {
        &self.native
    }
}

impl IosTextField {
    /// Returns the current field text, preserving `nil`, empty strings, and embedded NUL values.
    pub fn text(&self) -> Result<Option<String>> {
        let _field_text_thread = main_thread_marker()?;
        self.native
            .text()
            .as_deref()
            .map(native_string_to_rust)
            .transpose()
    }

    /// Replaces or clears the field text. `None` sets native `nil`; `Some("")` sets empty text.
    pub fn set_text(&mut self, text: Option<&str>) -> Result<()> {
        let _field_text_write_thread = main_thread_marker()?;
        autoreleasepool(|_| {
            let text = text.map(NSString::from_str);
            self.native.setText(text.as_deref());
        });
        Ok(())
    }

    /// Returns the current placeholder, preserving `nil`, empty strings, and embedded NUL values.
    pub fn placeholder(&self) -> Result<Option<String>> {
        let _field_placeholder_read_thread = main_thread_marker()?;
        self.native
            .placeholder()
            .as_deref()
            .map(native_string_to_rust)
            .transpose()
    }

    /// Replaces or clears the placeholder. `None` sets native `nil`; `Some("")` sets empty text.
    pub fn set_placeholder(&mut self, placeholder: Option<&str>) -> Result<()> {
        let _field_placeholder_write_thread = main_thread_marker()?;
        autoreleasepool(|_| {
            let placeholder = placeholder.map(NSString::from_str);
            self.native.setPlaceholder(placeholder.as_deref());
        });
        Ok(())
    }

    /// Requests first-responder status and returns UIKit's result.
    ///
    /// The returned value does not establish that a keyboard became visible.
    pub fn request_first_responder(&mut self) -> Result<bool> {
        let _field_focus_thread = main_thread_marker()?;
        Ok(self.native.becomeFirstResponder())
    }

    /// Requests that the field resign first-responder status and returns UIKit's result.
    pub fn resign_first_responder(&mut self) -> Result<bool> {
        let _field_resign_thread = main_thread_marker()?;
        Ok(self.native.resignFirstResponder())
    }

    /// Borrows the native text field for UIKit integration.
    pub fn native_text_field(&self) -> &UITextField {
        &self.native
    }
}

impl TextFieldControl for IosTextField {
    fn text(&self) -> Result<Option<String>> {
        IosTextField::text(self)
    }

    fn set_text(&mut self, text: Option<&str>) -> Result<()> {
        IosTextField::set_text(self, text)
    }
}

impl IosStackView {
    /// Adds a native view as an arranged child. UIKit retains and reparents the view.
    ///
    /// Keep its `ios-ui` handle alive when it owns behavior such as a button action. This method
    /// does not change the stack's placement in its parent or add parent constraints.
    pub fn add_arranged_subview(&mut self, view: &UIView) -> Result<()> {
        let _stack_thread = main_thread_marker()?;
        self.native.addArrangedSubview(view);
        Ok(())
    }

    /// Sets the spacing between arranged children in UIKit points.
    /// Negative finite values can overlap arranged children.
    pub fn set_spacing(&mut self, spacing: f64) -> Result<()> {
        if !is_valid_stack_spacing(spacing) {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        let _stack_thread = main_thread_marker()?;
        self.native.setSpacing(spacing as _);
        Ok(())
    }

    /// Borrows the native stack view for caller-owned UIKit layout and integration.
    pub fn native_stack_view(&self) -> &UIStackView {
        &self.native
    }
}

impl IosScrollView {
    /// Borrows the content host for insertion through existing `IosUiBackend` methods.
    pub fn content_view_mut(&mut self) -> &mut IosView {
        &mut self.content
    }

    /// Sets the scroll content dimensions in UIKit points without clamping.
    ///
    /// Both values must be finite and non-negative. The owned content host is resized at origin
    /// `(0, 0)` to match UIKit's `contentSize`. The caller remains responsible for positioning
    /// child views within that host.
    pub fn set_content_size(&mut self, width: f64, height: f64) -> Result<()> {
        let (width, height) = validated_content_size(width, height)?;
        let _scroll_thread = main_thread_marker()?;
        self.content.native.setFrame(CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(width, height),
        ));
        self.native.setContentSize(CGSize::new(width, height));
        Ok(())
    }

    /// Borrows the native scroll view for caller-owned UIKit integration.
    pub fn native_scroll_view(&self) -> &UIScrollView {
        &self.native
    }
}

impl IosToggle {
    /// Reads the current state on UIKit's main thread.
    pub fn is_on(&self) -> Result<bool> {
        let _toggle_thread = main_thread_marker()?;
        Ok(self.native.isOn())
    }

    /// Sets the state on UIKit's main thread with the requested native animation.
    ///
    /// UIKit's `setOn:animated:` does not send a `ValueChanged` action; this programmatic update
    /// therefore does not invoke the user-change callback.
    pub fn set_on(&mut self, on: bool, animated: bool) -> Result<()> {
        let _toggle_thread = main_thread_marker()?;
        self.native.setOn_animated(on, animated);
        Ok(())
    }

    /// Borrows the native switch for caller-owned UIKit integration.
    pub fn native_switch(&self) -> &UISwitch {
        &self.native
    }
}

impl ToggleControl for IosToggle {
    fn is_on(&self) -> Result<bool> {
        IosToggle::is_on(self)
    }

    fn set_state(&mut self, on: bool) -> Result<()> {
        self.set_on(on, false)
    }
}

impl IosButton {
    /// Borrows the native button for UIKit integration.
    pub fn native_button(&self) -> &UIButton {
        &self.native
    }
}

impl Drop for IosToggle {
    fn drop(&mut self) {
        // SAFETY: the handle is main-thread-only and removes only its retained action for the
        // exact selector and ValueChanged event registered below.
        unsafe {
            let value_changed_event = UIControlEvents::ValueChanged;
            self.native.removeTarget_action_forControlEvents(
                Some(&*self._target),
                Some(sel!(toggleChanged:)),
                value_changed_event,
            );
        }
    }
}

impl Drop for IosTextField {
    fn drop(&mut self) {
        if let Some(target) = &self._target {
            // SAFETY: this main-thread-only handle removes only its exact retained target, selector,
            // and EditingChanged event before releasing callback ownership.
            unsafe {
                self.native.removeTarget_action_forControlEvents(
                    Some(&**target),
                    Some(sel!(textFieldChanged:)),
                    UIControlEvents::EditingChanged,
                );
            }
        }
    }
}

impl Drop for IosButton {
    fn drop(&mut self) {
        // SAFETY: the button handle is main-thread-only, and the target is the exact retained
        // receiver registered for this selector and event. Remove only this action before release.
        unsafe {
            self.native.removeTarget_action_forControlEvents(
                Some(&*self._target),
                Some(sel!(buttonTapped:)),
                UIControlEvents::TouchUpInside,
            );
        }
    }
}

impl LabelControl for IosLabel {
    fn set_text(&mut self, text: &str) -> Result<()> {
        let _marker = main_thread_marker()?;
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            self.native.setText(Some(&text));
        });
        Ok(())
    }
}

impl ButtonControl for IosButton {
    fn set_title(&mut self, title: &str) -> Result<()> {
        let _marker = main_thread_marker()?;
        autoreleasepool(|_| {
            let title = NSString::from_str(title);
            self.native
                .setTitle_forState(Some(&title), UIControlState::Normal);
        });
        Ok(())
    }
}

impl UiBackend for IosUiBackend {
    type View = IosView;
    type Label = IosLabel;
    type Button = IosButton;
    type Toggle = IosToggle;
    type TextField = IosTextField;

    fn create_view(&mut self, frame: Frame) -> Result<Self::View> {
        let marker = main_thread_marker()?;
        let native = UIView::new(marker);
        native.setFrame(native_frame(frame));
        Ok(IosView {
            native,
            _not_send: PhantomData,
        })
    }

    fn add_label(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        text: &str,
    ) -> Result<Self::Label> {
        let marker = main_thread_marker()?;
        let native = UILabel::new(marker);
        native.setFrame(native_frame(frame));
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            native.setText(Some(&text));
        });
        parent.native.addSubview(&native);
        Ok(IosLabel {
            native,
            _not_send: PhantomData,
        })
    }

    fn add_button<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        title: &str,
        action: F,
    ) -> Result<Self::Button>
    where
        F: FnMut() + 'static,
    {
        let marker = main_thread_marker()?;
        let native = UIButton::new(marker);
        native.setFrame(native_frame(frame));
        autoreleasepool(|_| {
            let title = NSString::from_str(title);
            native.setTitle_forState(Some(&title), UIControlState::Normal);
        });
        let target = ButtonTarget::with_action(marker, Box::new(action));
        // SAFETY: `target` is strongly retained by the returned `IosButton`, and the target
        // implements the exact `buttonTapped:` selector registered for `TouchUpInside`.
        unsafe {
            native.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(buttonTapped:),
                UIControlEvents::TouchUpInside,
            );
        }
        parent.native.addSubview(&native);
        Ok(IosButton {
            native,
            _target: target,
            _not_send: PhantomData,
        })
    }

    fn add_toggle<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        initial_on: bool,
        action: F,
    ) -> Result<Self::Toggle>
    where
        F: FnMut(bool) + 'static,
    {
        let marker = main_thread_marker()?;
        let native = UISwitch::new(marker);
        native.setFrame(native_frame(frame));
        native.setOn_animated(initial_on, false);
        let target = ToggleTarget::with_action(marker, Box::new(action));
        // SAFETY: `target` is strongly retained by IosToggle and implements the exact
        // `toggleChanged:` selector registered for UIControlEvents::ValueChanged.
        unsafe {
            native.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(toggleChanged:),
                UIControlEvents::ValueChanged,
            );
        }
        parent.native.addSubview(&native);
        Ok(IosToggle {
            native,
            _target: target,
            _not_send: PhantomData,
        })
    }

    fn add_text_field<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        initial_text: Option<&str>,
        placeholder: Option<&str>,
        action: F,
    ) -> Result<Self::TextField>
    where
        F: FnMut(Result<Option<String>>) + 'static,
    {
        self.add_text_field_with_mode(
            parent,
            frame,
            TextFieldInputMode::Text,
            initial_text,
            placeholder,
            action,
        )
    }

    fn add_text_field_with_mode<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        input_mode: TextFieldInputMode,
        initial_text: Option<&str>,
        placeholder: Option<&str>,
        action: F,
    ) -> Result<Self::TextField>
    where
        F: FnMut(Result<Option<String>>) + 'static,
    {
        let marker = main_thread_marker()?;
        let native = UITextField::new(marker);
        native.setFrame(native_frame(frame));
        let (secure_text_entry, keyboard_hint) = input_traits(input_mode);
        native.setSecureTextEntry(secure_text_entry);
        native.setKeyboardType(match keyboard_hint {
            KeyboardHint::Default => UIKeyboardType::Default,
            KeyboardHint::NumberPad => UIKeyboardType::NumberPad,
            KeyboardHint::DecimalPad => UIKeyboardType::DecimalPad,
        });
        autoreleasepool(|_| {
            let text = initial_text.map(NSString::from_str);
            native.setText(text.as_deref());
            let placeholder = placeholder.map(NSString::from_str);
            native.setPlaceholder(placeholder.as_deref());
        });
        let target = TextFieldTarget::with_action(marker, Box::new(action));
        // SAFETY: `target` is retained by IosTextField and implements the exact selector registered
        // for UIControlEvents::EditingChanged.
        unsafe {
            native.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(textFieldChanged:),
                UIControlEvents::EditingChanged,
            );
        }
        parent.native.addSubview(&native);
        Ok(IosTextField {
            native,
            _target: Some(target),
            _text_field_not_send: PhantomData,
        })
    }
}

fn main_thread_marker() -> Result<MainThreadMarker> {
    MainThread::current()
        .map(MainThread::into_objc2)
        .ok_or_else(|| Error::new(ErrorKind::Unavailable))
}

fn native_frame(frame: Frame) -> CGRect {
    CGRect::new(
        CGPoint::new(frame.x(), frame.y()),
        CGSize::new(frame.width(), frame.height()),
    )
}

fn native_string_to_rust(value: &NSString) -> Result<String> {
    let length = value.len_utf16();
    let mut units = Vec::new();
    units
        .try_reserve_exact(length)
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    units.resize(length, 0);
    if length != 0 {
        let buffer = NonNull::new(units.as_mut_ptr())
            .ok_or_else(|| Error::new(ErrorKind::ResourceExhausted))?;
        // SAFETY: `buffer` points to `length` initialized writable UTF-16 units, and the requested
        // range is exactly the live NSString's full `length_utf16()` range.
        unsafe { value.getCharacters_range(buffer, NSRange::new(0, length)) };
    }
    decode_utf16(&units).map_err(|error| match error {
        Utf16Error::Invalid => Error::new(ErrorKind::InvalidInput),
        Utf16Error::ResourceExhausted => Error::new(ErrorKind::ResourceExhausted),
    })
}

//! UIKit's native text input system exposed as full UTF-16 snapshots for custom renderers.

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_core_foundation::CGRect;
use objc2_foundation::{
    NSNotification, NSNotificationCenter, NSObjectProtocol, NSRange, NSString, NSValue,
};
use objc2_ui_kit::{
    NSValueUIGeometryExtensions, UIKeyboardType, UIKeyboardWillChangeFrameNotification,
    UIScrollViewDelegate, UITextAutocapitalizationType, UITextAutocorrectionType, UITextInput,
    UITextInputTraits, UITextView, UITextViewDelegate, UIView,
};

use crate::custom_render_text_policy::{
    CallbackGate, CommittedText, Utf16Range, Utf16RangeError, utf16_range_to_bytes,
};
use crate::text_conversion::decode_utf16;

type TextInputCallback = Box<dyn FnMut(Result<IosCustomTextInputSnapshot>)>;

/// Native text-input keyboard layout preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IosTextInputKeyboardType {
    /// The standard system keyboard.
    #[default]
    Default,
    /// A keyboard with digits and punctuation.
    NumbersAndPunctuation,
    /// A URL-oriented keyboard.
    Url,
    /// A numeric keypad.
    NumberPad,
    /// A telephone keypad.
    PhonePad,
    /// An email-address keyboard.
    EmailAddress,
    /// A decimal keypad.
    DecimalPad,
}

impl IosTextInputKeyboardType {
    fn native(self) -> UIKeyboardType {
        match self {
            Self::Default => UIKeyboardType::Default,
            Self::NumbersAndPunctuation => UIKeyboardType::NumbersAndPunctuation,
            Self::Url => UIKeyboardType::URL,
            Self::NumberPad => UIKeyboardType::NumberPad,
            Self::PhonePad => UIKeyboardType::PhonePad,
            Self::EmailAddress => UIKeyboardType::EmailAddress,
            Self::DecimalPad => UIKeyboardType::DecimalPad,
        }
    }
}

/// Input traits configured on the native `UITextView`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IosTextInputTraits {
    /// Requested native keyboard layout.
    pub keyboard_type: IosTextInputKeyboardType,
    /// Whether UIKit may offer autocorrection.
    pub autocorrection: bool,
    /// Requested initial-capitalization behavior.
    pub capitalization: IosTextCapitalization,
}

impl Default for IosTextInputTraits {
    fn default() -> Self {
        Self {
            keyboard_type: IosTextInputKeyboardType::Default,
            autocorrection: true,
            capitalization: IosTextCapitalization::Sentences,
        }
    }
}

/// Native initial-capitalization behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IosTextCapitalization {
    /// Do not request automatic capitalization.
    None,
    /// Capitalize each new word.
    Words,
    /// Capitalize sentences.
    #[default]
    Sentences,
    /// Capitalize all characters.
    AllCharacters,
}

impl IosTextCapitalization {
    fn native(self) -> UITextAutocapitalizationType {
        match self {
            Self::None => UITextAutocapitalizationType::None,
            Self::Words => UITextAutocapitalizationType::Words,
            Self::Sentences => UITextAutocapitalizationType::Sentences,
            Self::AllCharacters => UITextAutocapitalizationType::AllCharacters,
        }
    }
}

/// Keyboard frame in the screen coordinate space, measured in UIKit points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IosKeyboardFrame {
    /// Left edge in screen points.
    pub x: f64,
    /// Top edge in screen points.
    pub y: f64,
    /// Width in screen points.
    pub width: f64,
    /// Height in screen points.
    pub height: f64,
}

/// Full native text and input state at one synchronous callback boundary.
///
/// `marked_text` distinguishes provisional IME text from committed text. While marked text is
/// present, consumers must retain the snapshot as composing state and must not reinterpret it as
/// a sequence of key presses. `selection` and `marked_text` are UTF-16 code-unit ranges, matching
/// UIKit's `UITextInput` offsets exactly.
#[derive(Clone, Debug, PartialEq)]
pub struct IosCustomTextInputSnapshot {
    /// Complete current text, including any active marked/composing substring.
    pub text: String,
    /// Current selection/caret range in UTF-16 code units.
    pub selection: Utf16Range,
    /// Current marked/composing range in UTF-16 code units, if UIKit has one.
    pub marked_text: Option<Utf16Range>,
    /// Whether this text view is the current first responder.
    pub focused: bool,
    /// Whether UIKit reports a software keyboard for this input view.
    pub keyboard_visible: bool,
    /// Latest keyboard frame in screen points when a frame is available.
    pub keyboard_frame: Option<IosKeyboardFrame>,
}

struct TextInputTargetState {
    native: Retained<UITextView>,
    callback: RefCell<TextInputCallback>,
    callback_gate: Cell<CallbackGate>,
    committed: RefCell<CommittedText>,
    keyboard_visible: Cell<bool>,
    keyboard_frame: Cell<Option<IosKeyboardFrame>>,
    suppress_events: Cell<bool>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - TextInputTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosCustomRenderTextInputTarget"]
    #[ivars = TextInputTargetState]
    struct TextInputTarget;

    // SAFETY: UITextView does not need scroll callbacks for text input.
    unsafe impl UIScrollViewDelegate for TextInputTarget {}

    // SAFETY: UIKit sends these delegate calls on the main thread while the handle retains target.
    #[allow(non_snake_case)]
    unsafe impl UITextViewDelegate for TextInputTarget {
        #[unsafe(method(textViewDidChange:))]
        fn textViewDidChange(&self, _text_view: &UITextView) {
            let _keep_alive = self.retain();
            self.emit_snapshot();
        }

        #[unsafe(method(textViewDidChangeSelection:))]
        fn textViewDidChangeSelection(&self, _text_view: &UITextView) {
            let _keep_alive = self.retain();
            self.emit_snapshot();
        }

        #[unsafe(method(textViewDidBeginEditing:))]
        fn textViewDidBeginEditing(&self, _text_view: &UITextView) {
            let _keep_alive = self.retain();
            self.emit_snapshot();
        }

        #[unsafe(method(textViewDidEndEditing:))]
        fn textViewDidEndEditing(&self, _text_view: &UITextView) {
            let _keep_alive = self.retain();
            self.ivars().keyboard_visible.set(false);
            self.ivars().keyboard_frame.set(None);
            self.emit_snapshot();
        }
    }

    // SAFETY: The class supplies the exact public selector registered for UIKit keyboard notifications.
    #[allow(non_snake_case)]
    impl TextInputTarget {
        #[unsafe(method(keyboardWillChangeFrame:))]
        fn keyboard_will_change_frame(&self, notification: &NSNotification) {
            let _keep_alive = self.retain();
            self.keyboard_frame_changed(notification);
        }

        #[unsafe(method(keyboardWillShow:))]
        fn keyboard_will_show(&self, notification: &NSNotification) {
            let _keep_alive = self.retain();
            if !self.ivars().native.isFirstResponder() {
                return;
            }
            self.ivars().keyboard_visible.set(true);
            self.ivars()
                .keyboard_frame
                .set(notification_keyboard_frame(notification));
            self.emit_snapshot();
        }

        #[unsafe(method(keyboardWillHide:))]
        fn keyboard_will_hide(&self, _notification: &NSNotification) {
            let _keep_alive = self.retain();
            let state = self.ivars();
            if !state.native.isFirstResponder() && !state.keyboard_visible.get() {
                return;
            }
            state.keyboard_visible.set(false);
            state.keyboard_frame.set(None);
            self.emit_snapshot();
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for TextInputTarget {}
);

impl TextInputTarget {
    fn with_state(
        marker: MainThreadMarker,
        native: Retained<UITextView>,
        initial_text: &str,
        initial_selection: Utf16Range,
        callback: TextInputCallback,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(TextInputTargetState {
            native,
            callback: RefCell::new(callback),
            callback_gate: Cell::new(CallbackGate::active()),
            committed: RefCell::new(CommittedText::new(initial_text, initial_selection)),
            keyboard_visible: Cell::new(false),
            keyboard_frame: Cell::new(None),
            suppress_events: Cell::new(false),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }

    fn emit_snapshot(&self) {
        let _ = ios_runtime::ffi::catch_unwind(|| self.emit_snapshot_inner());
    }

    fn emit_snapshot_inner(&self) {
        let state = self.ivars();
        if !state.callback_gate.get().is_active() || state.suppress_events.get() {
            return;
        }
        let snapshot = snapshot_from_native(state);
        if let Ok(snapshot) = &snapshot {
            if let Ok(mut committed) = state.committed.try_borrow_mut() {
                committed.observe(&snapshot.text, snapshot.selection, snapshot.marked_text);
            }
        }
        let _ = ios_runtime::ffi::catch_unwind(|| {
            if let Ok(mut callback) = state.callback.try_borrow_mut() {
                callback(snapshot);
            }
        });
    }

    fn keyboard_frame_changed(&self, notification: &NSNotification) {
        let state = self.ivars();
        let focused = state.native.isFirstResponder();
        let keyboard_was_visible = state.keyboard_visible.get();
        if !focused && !keyboard_was_visible {
            return;
        }
        let frame = notification_keyboard_frame(notification);
        state.keyboard_frame.set(frame);
        if focused && frame.is_some() {
            state.keyboard_visible.set(true);
        }
        self.emit_snapshot();
    }
}

/// A main-thread UIKit text-input bridge attached to a caller-owned view hierarchy.
///
/// UIKit's `UITextView` owns the live editing buffer and performs insertion, deletion,
/// replacement, selection, and IME composition. The callback receives complete ordered snapshots
/// after UIKit text, selection, focus, or keyboard notifications; it does not receive synthetic
/// key events. During composition `marked_text` remains set and the full candidate buffer is
/// available through `text`. Rust may mirror committed snapshots into its own model. Programmatic
/// `set_text` is a silent replacement that cancels any marked range and moves selection to the end.
///
/// The handle owns one `UITextView`, its exact delegate, and its keyboard-notification observer.
/// Dropping it disables callbacks first, resigns first responder, removes only its observer/delegate,
/// and removes only its owned view. It never replaces a parent controller or scene delegate.
pub struct IosCustomTextInput {
    target: Retained<TextInputTarget>,
    notification_center: Retained<NSNotificationCenter>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosCustomTextInput {
    /// Add an editable native text-input surface to a caller-owned UIKit parent.
    ///
    /// The supplied frame uses parent-view points. Initial Rust text is copied into UIKit; the
    /// native `UITextView` then owns edits and marked text until Rust applies a later `set_text`.
    /// Keep the returned handle alive while composition, focus, and snapshots should remain active.
    pub fn attach<F>(
        parent: &UIView,
        frame: CGRect,
        initial_text: &str,
        traits: IosTextInputTraits,
        callback: F,
    ) -> Result<Self>
    where
        F: FnMut(Result<IosCustomTextInputSnapshot>) + 'static,
    {
        let marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        let native = UITextView::new(marker);
        native.setFrame(frame);
        native.setEditable(true);
        native.setSelectable(true);
        native.setScrollEnabled(false);
        native.setKeyboardType(traits.keyboard_type.native());
        native.setAutocorrectionType(if traits.autocorrection {
            UITextAutocorrectionType::Yes
        } else {
            UITextAutocorrectionType::No
        });
        native.setAutocapitalizationType(traits.capitalization.native());
        let initial_length = utf16_length(initial_text)?;
        objc2::rc::autoreleasepool(|_| {
            let text = NSString::from_str(initial_text);
            native.setText(Some(&text));
        });
        set_native_selection(&native, Utf16Range::new(initial_length, 0))?;

        let target = TextInputTarget::with_state(
            marker,
            native.clone(),
            initial_text,
            Utf16Range::new(initial_length, 0),
            Box::new(callback),
        );
        // SAFETY: target is retained here, both objects are main-thread-only, and UITextView stores
        // its delegate weakly. Drop removes this exact delegate before releasing the target.
        unsafe { native.setDelegate(Some(ProtocolObject::from_ref(&*target))) };
        parent.addSubview(&native);

        let center = NSNotificationCenter::defaultCenter();
        register_keyboard_observer(&center, &target);
        Ok(Self {
            target,
            notification_center: center,
            _not_send: PhantomData,
        })
    }

    /// Return a complete point-in-time text, selection, composition, and keyboard snapshot.
    pub fn snapshot(&self) -> Result<IosCustomTextInputSnapshot> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        snapshot_from_native(self.target.ivars())
    }

    /// Replace native text from Rust and put the caret at its UTF-16 end without an edit callback.
    ///
    /// This replaces active composing text. Use [`Self::cancel_composition`] when the last
    /// committed buffer must be restored instead.
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        let selection = Utf16Range::new(utf16_length(text)?, 0);
        replace_native_text(self.target.ivars(), text, selection)?;
        if let Ok(mut committed) = self.target.ivars().committed.try_borrow_mut() {
            *committed = CommittedText::new(text, selection);
        } else {
            return Err(Error::new(ErrorKind::Unavailable));
        }
        Ok(())
    }

    /// Set the native selection using exact UTF-16 offsets without an edit callback.
    ///
    /// The range must be in bounds and may not split a UTF-16 surrogate pair. UIKit's own
    /// user-driven selection remains available through `snapshot` and the callback.
    pub fn set_selection(&mut self, selection: Utf16Range) -> Result<()> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        let snapshot = snapshot_from_native(self.target.ivars())?;
        utf16_range_to_bytes(&snapshot.text, selection).map_err(range_error)?;
        let state = self.target.ivars();
        state.suppress_events.set(true);
        let result = set_native_selection(&state.native, selection);
        state.suppress_events.set(false);
        result?;
        let updated = snapshot_from_native(state)?;
        state
            .committed
            .try_borrow_mut()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?
            .observe(&updated.text, updated.selection, updated.marked_text);
        Ok(())
    }

    /// Request first-responder status and return UIKit's synchronous acceptance result.
    ///
    /// Acceptance does not prove that a software keyboard is visible; keyboard state arrives from
    /// public UIKit keyboard notifications and is reflected later in snapshots.
    pub fn request_focus(&mut self) -> Result<bool> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        Ok(self.target.ivars().native.becomeFirstResponder())
    }

    /// Resign first-responder status and return UIKit's synchronous result.
    pub fn resign_focus(&mut self) -> Result<bool> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        Ok(self.target.ivars().native.resignFirstResponder())
    }

    /// Cancel an active IME composition by restoring the last unmarked text and selection.
    ///
    /// Returns `false` when UIKit has no marked range. A successful cancellation emits one final
    /// full snapshot after restoring the committed buffer; callbacks caused by intermediate UIKit
    /// mutations are suppressed.
    pub fn cancel_composition(&mut self) -> Result<bool> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        if UITextInput::markedTextRange(&*self.target.ivars().native).is_none() {
            return Ok(false);
        }
        let committed = self
            .target
            .ivars()
            .committed
            .try_borrow()
            .map_err(|_| Error::new(ErrorKind::Unavailable))?;
        let (text, selection) = committed.value();
        let text = text.to_owned();
        replace_native_text(self.target.ivars(), &text, selection)?;
        self.target.emit_snapshot();
        Ok(true)
    }

    /// Change the native text-input surface frame in parent-view points.
    pub fn set_frame(&mut self, frame: CGRect) -> Result<()> {
        let _marker = MainThreadMarker::new().ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
        self.target.ivars().native.setFrame(frame);
        Ok(())
    }

    /// Borrow the native `UITextView` for additional public UIKit configuration.
    pub fn native_text_view(&self) -> &UITextView {
        &self.target.ivars().native
    }
}

impl Drop for IosCustomTextInput {
    fn drop(&mut self) {
        let state = self.target.ivars();
        let mut gate = state.callback_gate.get();
        gate.deactivate();
        state.callback_gate.set(gate);
        if let Ok(mut callback) = state.callback.try_borrow_mut() {
            *callback = Box::new(|_| {});
        }
        let native = &state.native;
        let _ = native.resignFirstResponder();
        native.unmarkText();
        let current = native.delegate();
        let own_delegate = ProtocolObject::from_ref(&*self.target);
        if current
            .as_deref()
            .is_some_and(|delegate| core::ptr::eq(delegate, own_delegate))
        {
            // SAFETY: the handle is main-thread-bound; only its exact weak delegate is removed.
            unsafe { native.setDelegate(None) };
        }
        unregister_keyboard_observer(&self.notification_center, &self.target);
        state.keyboard_visible.set(false);
        state.keyboard_frame.set(None);
        native.removeFromSuperview();
    }
}

fn snapshot_from_native(state: &TextInputTargetState) -> Result<IosCustomTextInputSnapshot> {
    let native = &*state.native;
    let text = native_text_to_rust(&native.text())?;
    let selected =
        UITextInput::selectedTextRange(native).ok_or_else(|| Error::new(ErrorKind::Unavailable))?;
    let selection = native_text_range(native, &selected)?;
    let marked_text = native_marked_range(native)?;
    Ok(IosCustomTextInputSnapshot {
        text,
        selection,
        marked_text,
        focused: native.isFirstResponder(),
        keyboard_visible: state.keyboard_visible.get(),
        keyboard_frame: state.keyboard_frame.get(),
    })
}

fn native_marked_range(native: &UITextView) -> Result<Option<Utf16Range>> {
    let Some(marked) = UITextInput::markedTextRange(native) else {
        return Ok(None);
    };
    native_text_range(native, &marked).map(Some)
}

fn native_text_range(native: &UITextView, range: &objc2_ui_kit::UITextRange) -> Result<Utf16Range> {
    let beginning = UITextInput::beginningOfDocument(native);
    let start = UITextInput::offsetFromPosition_toPosition(native, &beginning, &range.start());
    let end = UITextInput::offsetFromPosition_toPosition(native, &beginning, &range.end());
    let location = u32::try_from(start).map_err(|_| Error::new(ErrorKind::InvalidInput))?;
    let end = u32::try_from(end).map_err(|_| Error::new(ErrorKind::InvalidInput))?;
    let length = end
        .checked_sub(location)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput))?;
    Ok(Utf16Range::new(location, length))
}

fn set_native_selection(native: &UITextView, selection: Utf16Range) -> Result<()> {
    let start =
        isize::try_from(selection.location).map_err(|_| Error::new(ErrorKind::InvalidInput))?;
    let end = isize::try_from(
        selection
            .end()
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput))?,
    )
    .map_err(|_| Error::new(ErrorKind::InvalidInput))?;
    let beginning = UITextInput::beginningOfDocument(native);
    let start = UITextInput::positionFromPosition_offset(native, &beginning, start)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput))?;
    let end = UITextInput::positionFromPosition_offset(native, &beginning, end)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput))?;
    let range = UITextInput::textRangeFromPosition_toPosition(native, &start, &end)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput))?;
    UITextInput::setSelectedTextRange(native, Some(&range));
    Ok(())
}

fn replace_native_text(
    state: &TextInputTargetState,
    text: &str,
    selection: Utf16Range,
) -> Result<()> {
    state.suppress_events.set(true);
    state.native.unmarkText();
    objc2::rc::autoreleasepool(|_| {
        let native_text = NSString::from_str(text);
        state.native.setText(Some(&native_text));
    });
    let result = set_native_selection(&state.native, selection);
    state.suppress_events.set(false);
    result
}

fn native_text_to_rust(value: &NSString) -> Result<String> {
    let length = value.len_utf16();
    let mut units = Vec::new();
    units
        .try_reserve_exact(length)
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))?;
    units.resize(length, 0);
    if length != 0 {
        let buffer = NonNull::new(units.as_mut_ptr())
            .ok_or_else(|| Error::new(ErrorKind::ResourceExhausted))?;
        // SAFETY: buffer has `length` initialized writable UTF-16 units and the range equals the
        // live NSString's exact UTF-16 length.
        unsafe { value.getCharacters_range(buffer, NSRange::new(0, length)) };
    }
    decode_utf16(&units).map_err(|error| match error {
        crate::text_conversion::Utf16Error::Invalid => Error::new(ErrorKind::InvalidInput),
        crate::text_conversion::Utf16Error::ResourceExhausted => {
            Error::new(ErrorKind::ResourceExhausted)
        }
    })
}

fn utf16_length(text: &str) -> Result<u32> {
    text.encode_utf16()
        .count()
        .try_into()
        .map_err(|_| Error::new(ErrorKind::ResourceExhausted))
}

fn range_error(error: Utf16RangeError) -> Error {
    match error {
        Utf16RangeError::OutOfBounds | Utf16RangeError::SplitScalar | Utf16RangeError::Overflow => {
            Error::new(ErrorKind::InvalidInput)
        }
    }
}

fn notification_keyboard_frame(notification: &NSNotification) -> Option<IosKeyboardFrame> {
    let user_info = notification.userInfo()?;
    // SAFETY: this is the public UIKit user-info key for a keyboard frame encoded as an NSValue.
    let frame = user_info.objectForKey(unsafe { objc2_ui_kit::UIKeyboardFrameEndUserInfoKey })?;
    let frame = frame.downcast_ref::<NSValue>()?;
    // SAFETY: UIKit documents this key's value as an NSValue containing a CGRect.
    let rect = unsafe { frame.CGRectValue() };
    Some(IosKeyboardFrame {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    })
}

fn register_keyboard_observer(center: &NSNotificationCenter, target: &Retained<TextInputTarget>) {
    let observer: &AnyObject = target.as_ref();
    // SAFETY: target implements all exact public selectors registered here and is retained by the
    // handle. UIKit posts these notifications on its main thread; Drop removes this observer.
    unsafe {
        for (name, action) in [
            (
                UIKeyboardWillChangeFrameNotification,
                sel!(keyboardWillChangeFrame:),
            ),
            (
                objc2_ui_kit::UIKeyboardWillShowNotification,
                sel!(keyboardWillShow:),
            ),
            (
                objc2_ui_kit::UIKeyboardWillHideNotification,
                sel!(keyboardWillHide:),
            ),
        ] {
            center.addObserver_selector_name_object(observer, action, Some(name), None);
        }
    }
}

fn unregister_keyboard_observer(center: &NSNotificationCenter, target: &Retained<TextInputTarget>) {
    let observer: &AnyObject = target.as_ref();
    // SAFETY: target is exactly the observer registered by this handle.
    unsafe { center.removeObserver(observer) };
}

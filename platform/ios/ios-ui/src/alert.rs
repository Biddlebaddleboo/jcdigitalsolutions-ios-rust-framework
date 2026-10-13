use core::cell::RefCell;
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::rc::Rc;

use block2::RcBlock;
use ios_runtime::main_thread::MainThread;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObjectProtocol, NSRange, NSString};
use objc2_ui_kit::{
    UIAdaptivePresentationControllerDelegate, UIAlertAction, UIAlertActionStyle, UIAlertController,
    UIAlertControllerStyle, UIDevice, UIKeyboardType, UIPopoverPresentationController,
    UIPopoverPresentationControllerDelegate, UIPresentationController, UITextField,
    UITextFieldDelegate, UITextInputTraits, UIUserInterfaceIdiom, UIView, UIViewController,
};

use crate::alert_policy::{
    DEFAULT_TEXT_FIELD_MAX_UTF16_UNITS, MAX_TEXT_FIELD_MAX_UTF16_UNITS, allows_utf16_replacement,
    next_action_id, next_text_field_count, valid_alert_text, valid_initial_text,
    valid_optional_text, valid_rect_components, valid_text_field_limit,
};
use crate::alert_state::AlertCompletion;
pub use crate::alert_state::{IosAlertError, IosAlertResult, IosAlertStartError};
use crate::text_conversion::{Utf16Error, decode_utf16};

/// The native UIKit controller presentation style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosAlertStyle {
    /// A modal alert with an optional title, message, and text fields.
    Alert,
    /// A native action sheet; on iPad it uses a popover presentation.
    ActionSheet,
}

/// The public style of one alert action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosAlertActionStyle {
    /// A normal action.
    Default,
    /// The single optional cancellation action.
    Cancel,
    /// An action that UIKit marks as destructive.
    Destructive,
}

/// A supported UIKit keyboard layout for an alert text field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosAlertKeyboardType {
    /// The system's default keyboard.
    Default,
    /// A numeric keypad.
    NumberPad,
    /// A decimal keypad.
    DecimalPad,
    /// A phone keypad.
    PhonePad,
    /// An email-address keyboard.
    EmailAddress,
    /// A URL keyboard.
    Url,
}

/// Configuration for one bounded alert text field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IosAlertTextFieldConfig {
    /// Optional placeholder text.
    pub placeholder: Option<String>,
    /// Optional initial value.
    pub initial_text: Option<String>,
    /// Whether UIKit hides entered characters.
    pub secure_text_entry: bool,
    /// The native keyboard layout.
    pub keyboard_type: IosAlertKeyboardType,
    /// Maximum UTF-16 code units accepted by the native text-field delegate.
    pub maximum_utf16_units: u16,
}

impl Default for IosAlertTextFieldConfig {
    fn default() -> Self {
        Self {
            placeholder: None,
            initial_text: None,
            secure_text_entry: false,
            keyboard_type: IosAlertKeyboardType::Default,
            maximum_utf16_units: DEFAULT_TEXT_FIELD_MAX_UTF16_UNITS,
        }
    }
}

struct PopoverAnchor {
    view: Retained<UIView>,
    rect: CGRect,
}

/// A main-thread-bound native alert/action-sheet session.
///
/// The caller supplies and owns the presenter. Its borrow prevents the scene host from being
/// safely destroyed while the session exists. The session is neither Send nor Sync. Keep it alive
/// while the alert should remain interactive; dropping it detaches the callback, unregisters its
/// delegates, and dismisses an active presentation. All public UIKit operations run on the main
/// thread and do not block.
pub struct IosAlertSession<'ctx> {
    presenter: &'ctx UIViewController,
    alert: Retained<UIAlertController>,
    style: IosAlertStyle,
    action_count: u32,
    has_cancel_action: bool,
    text_field_count: u8,
    text_fields: Rc<RefCell<Vec<Retained<UITextField>>>>,
    text_field_delegates: Vec<Retained<AlertTextFieldDelegate>>,
    presentation_delegate: Retained<AlertPresentationDelegate>,
    anchor: Option<PopoverAnchor>,
    completion: AlertCompletion,
    presented: bool,
    _not_send: PhantomData<Rc<()>>,
}

struct TextFieldLimitState {
    maximum_utf16_units: u16,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - The class is main-thread-only and its fixed-size ivars do not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiAlertTextFieldLimit"]
    #[ivars = TextFieldLimitState]
    struct AlertTextFieldDelegate;

    // SAFETY: UIKit calls this delegate on the main thread while the alert session retains it.
    #[allow(non_snake_case, deprecated)]
    unsafe impl UITextFieldDelegate for AlertTextFieldDelegate {
        #[unsafe(method(textField:shouldChangeCharactersInRange:replacementString:))]
        fn textField_shouldChangeCharactersInRange_replacementString(
            &self,
            text_field: &UITextField,
            range: NSRange,
            replacement: &NSString,
        ) -> bool {
            let _keep_alive = self.retain();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let current_length = text_field.text().map_or(0, |text| text.len_utf16());
                allows_utf16_replacement(
                    current_length,
                    range.location,
                    range.length,
                    replacement.len_utf16(),
                    self.ivars().maximum_utf16_units.into(),
                )
            }));
            outcome.unwrap_or(false)
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for AlertTextFieldDelegate {}
);

impl AlertTextFieldDelegate {
    fn new(marker: MainThreadMarker, maximum_utf16_units: u16) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(TextFieldLimitState {
            maximum_utf16_units,
        });
        // SAFETY: The ivars are initialized before NSObject's initializer is called.
        unsafe { msg_send![super(this), init] }
    }
}

struct AlertPresentationState {
    completion: AlertCompletion,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - The class is main-thread-only and owns only main-thread callback state.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiAlertPresentation"]
    #[ivars = AlertPresentationState]
    struct AlertPresentationDelegate;

    // SAFETY: UIKit calls the optional dismissal callback on the main thread.
    #[allow(non_snake_case)]
    unsafe impl UIAdaptivePresentationControllerDelegate for AlertPresentationDelegate {
        #[unsafe(method(presentationControllerDidDismiss:))]
        fn presentationControllerDidDismiss(
            &self,
            _presentation_controller: &UIPresentationController,
        ) {
            let _keep_alive = self.retain();
            self.ivars().completion.complete(IosAlertResult::Cancelled);
        }
    }

    // SAFETY: The inherited adaptive callbacks above are implemented, and UIKit uses this
    // conformance for action-sheet popover dismissal callbacks.
    unsafe impl UIPopoverPresentationControllerDelegate for AlertPresentationDelegate {
        #[allow(deprecated)]
        #[unsafe(method(popoverPresentationControllerDidDismissPopover:))]
        fn popover_presentation_controller_did_dismiss_popover(
            &self,
            _popover_presentation_controller: &UIPopoverPresentationController,
        ) {
            let _keep_alive = self.retain();
            self.ivars().completion.complete(IosAlertResult::Cancelled);
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for AlertPresentationDelegate {}
);

impl AlertPresentationDelegate {
    fn new(marker: MainThreadMarker, completion: AlertCompletion) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(AlertPresentationState { completion });
        // SAFETY: The ivars are initialized before NSObject's initializer is called.
        unsafe { msg_send![super(this), init] }
    }
}

impl<'ctx> IosAlertSession<'ctx> {
    /// Creates an alert for a borrowed, caller-owned UIKit presenter.
    ///
    /// The main-thread proof is consumed to construct UIKit objects. Title and message strings
    /// are copied into NSString values after a 4096-byte bound check. The presenter must belong
    /// to the scene that should display the alert.
    pub fn new(
        main_thread: MainThread,
        presenter: &'ctx UIViewController,
        style: IosAlertStyle,
        title: Option<&str>,
        message: Option<&str>,
    ) -> Result<Self, IosAlertError> {
        if !valid_optional_text(title) || !valid_optional_text(message) {
            return Err(IosAlertError::InvalidInput);
        }
        let marker = main_thread.into_objc2();
        let title = title.map(NSString::from_str);
        let message = message.map(NSString::from_str);
        let native_style = match style {
            IosAlertStyle::Alert => UIAlertControllerStyle::Alert,
            IosAlertStyle::ActionSheet => UIAlertControllerStyle::ActionSheet,
        };
        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            title.as_deref(),
            message.as_deref(),
            native_style,
            marker,
        );
        let completion = AlertCompletion::default();
        let presentation_delegate = AlertPresentationDelegate::new(marker, completion.clone());
        Ok(Self {
            presenter,
            alert,
            style,
            action_count: 0,
            has_cancel_action: false,
            text_field_count: 0,
            text_fields: Rc::default(),
            text_field_delegates: Vec::new(),
            presentation_delegate,
            anchor: None,
            completion,
            presented: false,
            _not_send: PhantomData,
        })
    }

    /// Adds a bounded native action and returns its stable insertion-order ID.
    ///
    /// An alert can contain at most 32 actions and one cancellation action. The callback is
    /// installed when present is called; action Blocks retain only shared one-shot state.
    pub fn add_action(
        &mut self,
        title: &str,
        style: IosAlertActionStyle,
    ) -> Result<u32, IosAlertError> {
        self.ensure_mutable()?;
        if !valid_alert_text(title) {
            return Err(IosAlertError::InvalidInput);
        }
        if style == IosAlertActionStyle::Cancel && self.has_cancel_action {
            return Err(IosAlertError::CancelActionAlreadyAdded);
        }
        let action_id = next_action_id(self.action_count)?;
        let marker = MainThreadMarker::new().ok_or(IosAlertError::MainThreadUnavailable)?;
        let title = NSString::from_str(title);
        let native_style = match style {
            IosAlertActionStyle::Default => UIAlertActionStyle::Default,
            IosAlertActionStyle::Cancel => UIAlertActionStyle::Cancel,
            IosAlertActionStyle::Destructive => UIAlertActionStyle::Destructive,
        };
        let completion = self.completion.clone();
        let text_fields = Rc::clone(&self.text_fields);
        let handler = RcBlock::new(move |_action: NonNull<UIAlertAction>| {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                snapshot_text_fields(&text_fields)
            }));
            match result {
                Ok(Ok(text_fields)) => {
                    completion.complete(IosAlertResult::ActionSelected {
                        action_id,
                        text_fields,
                    });
                }
                Ok(Err(error)) => {
                    completion.complete(IosAlertResult::Failed(error));
                }
                Err(_) => {
                    completion.complete(IosAlertResult::Failed(IosAlertError::ResourceExhausted));
                }
            }
        });
        let action = UIAlertAction::actionWithTitle_style_handler(
            Some(&title),
            native_style,
            Some(&handler),
            marker,
        );
        self.alert.addAction(&action);
        self.action_count += 1;
        self.has_cancel_action |= style == IosAlertActionStyle::Cancel;
        Ok(action_id)
    }

    /// Adds a UIKit alert text field with secure-entry and keyboard options.
    ///
    /// At most two fields are accepted. Each field has a UTF-16 maximum no greater than 4096
    /// units; the default is 256. Action sheets do not support text fields.
    pub fn add_text_field(&mut self, config: IosAlertTextFieldConfig) -> Result<(), IosAlertError> {
        self.ensure_mutable()?;
        if self.style != IosAlertStyle::Alert {
            return Err(IosAlertError::TextFieldsRequireAlertStyle);
        }
        if !valid_optional_text(config.placeholder.as_deref())
            || !valid_optional_text(config.initial_text.as_deref())
        {
            return Err(IosAlertError::InvalidInput);
        }
        if !valid_text_field_limit(config.maximum_utf16_units) {
            return Err(IosAlertError::InvalidTextFieldLimit);
        }
        if !valid_initial_text(config.initial_text.as_deref(), config.maximum_utf16_units) {
            return Err(IosAlertError::InvalidInput);
        }
        let next_count = next_text_field_count(self.text_field_count)?;
        self.text_field_delegates
            .try_reserve(1)
            .map_err(|_| IosAlertError::ResourceExhausted)?;
        self.text_fields
            .try_borrow_mut()
            .map_err(|_| IosAlertError::AlreadyPresented)?
            .try_reserve(1)
            .map_err(|_| IosAlertError::ResourceExhausted)?;

        self.alert.addTextFieldWithConfigurationHandler(None);
        let fields = self
            .alert
            .textFields()
            .ok_or(IosAlertError::TextFieldUnavailable)?;
        let field = fields
            .lastObject()
            .ok_or(IosAlertError::TextFieldUnavailable)?;
        let placeholder = config.placeholder.as_deref().map(NSString::from_str);
        let initial_text = config.initial_text.as_deref().map(NSString::from_str);
        field.setPlaceholder(placeholder.as_deref());
        field.setText(initial_text.as_deref());
        field.setSecureTextEntry(config.secure_text_entry);
        field.setKeyboardType(native_keyboard_type(config.keyboard_type));

        let marker = MainThreadMarker::new().ok_or(IosAlertError::MainThreadUnavailable)?;
        let delegate = AlertTextFieldDelegate::new(marker, config.maximum_utf16_units);
        // SAFETY: The session retains the delegate for the field's lifetime; UITextField stores
        // this delegate weakly, and session teardown clears the exact delegate before release.
        field.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        self.text_fields
            .try_borrow_mut()
            .map_err(|_| IosAlertError::AlreadyPresented)?
            .push(field);
        self.text_field_delegates.push(delegate);
        self.text_field_count = next_count;
        Ok(())
    }

    /// Sets the action-sheet popover anchor in the supplied view's local coordinates.
    ///
    /// The view is retained for the session to keep UIKit's weak popover source valid. At
    /// presentation time it must belong to the same window as the presenter.
    pub fn set_popover_anchor(
        &mut self,
        view: &UIView,
        source_rect: CGRect,
    ) -> Result<(), IosAlertError> {
        self.ensure_mutable()?;
        if self.style != IosAlertStyle::ActionSheet {
            return Err(IosAlertError::InvalidPopoverAnchor);
        }
        if !valid_rect(source_rect) {
            return Err(IosAlertError::InvalidPopoverAnchor);
        }
        self.anchor = Some(PopoverAnchor {
            view: view.retain(),
            rect: source_rect,
        });
        Ok(())
    }

    /// Presents the configured alert from the caller-owned controller.
    ///
    /// Synchronous validation failures return both the error and the unconsumed callback. A
    /// successful return means UIKit accepted a presentation request; it is not runtime proof
    /// that the alert became visible. The callback runs at most once after an action, interactive
    /// dismissal, explicit dismiss, or scene/session drop. Callback panics are contained.
    pub fn present<F>(&mut self, callback: F) -> Result<(), IosAlertStartError<F>>
    where
        F: FnOnce(IosAlertResult) + 'static,
    {
        let failure = |error, callback| IosAlertStartError { error, callback };
        if self.presented {
            return Err(failure(IosAlertError::AlreadyPresented, callback));
        }
        if self.action_count == 0 {
            return Err(failure(IosAlertError::NoActions, callback));
        }
        let Some(marker) = MainThreadMarker::new() else {
            return Err(failure(IosAlertError::MainThreadUnavailable, callback));
        };
        let Some(presenter_view) = self.presenter.viewIfLoaded() else {
            return Err(failure(IosAlertError::PresenterUnavailable, callback));
        };
        let Some(presenter_window) = presenter_view.window() else {
            return Err(failure(IosAlertError::PresenterUnavailable, callback));
        };
        if self.presenter.isBeingPresented()
            || self.presenter.isBeingDismissed()
            || self.presenter.presentedViewController().is_some()
        {
            return Err(failure(IosAlertError::PresenterUnavailable, callback));
        }

        if self.style == IosAlertStyle::ActionSheet {
            let popover = self.alert.popoverPresentationController();
            if popover.is_none()
                && (UIDevice::currentDevice(marker).userInterfaceIdiom()
                    == UIUserInterfaceIdiom::Pad
                    || self.anchor.is_some())
            {
                return Err(failure(IosAlertError::PopoverUnavailable, callback));
            }
            if let Some(popover) = popover {
                let (source_view, source_rect) = match &self.anchor {
                    Some(anchor) => {
                        let Some(source_window) = anchor.view.window() else {
                            return Err(failure(IosAlertError::PresenterUnavailable, callback));
                        };
                        if !core::ptr::eq(
                            Retained::as_ptr(&presenter_window),
                            Retained::as_ptr(&source_window),
                        ) {
                            return Err(failure(IosAlertError::PresenterUnavailable, callback));
                        }
                        (&*anchor.view, anchor.rect)
                    }
                    None => {
                        let bounds = presenter_view.bounds();
                        let center = CGPoint::new(
                            bounds.origin.x + bounds.size.width / 2.0,
                            bounds.origin.y + bounds.size.height / 2.0,
                        );
                        (&*presenter_view, CGRect::new(center, CGSize::new(1.0, 1.0)))
                    }
                };
                popover.setSourceView(Some(source_view));
                popover.setSourceRect(source_rect);
                // SAFETY: The session retains this delegate while UIKit stores the property weakly.
                unsafe {
                    popover
                        .setDelegate(Some(ProtocolObject::from_ref(&*self.presentation_delegate)));
                }
            }
        }

        if let Err(callback) = self.completion.install(callback) {
            return Err(failure(IosAlertError::AlreadyPresented, callback));
        }
        self.presenter
            .presentViewController_animated_completion(&self.alert, true, None);
        self.presented = true;
        if let Some(presentation) = self.alert.presentationController() {
            // SAFETY: The session retains this delegate while UIKit stores the property weakly.
            unsafe {
                presentation
                    .setDelegate(Some(ProtocolObject::from_ref(&*self.presentation_delegate)));
            }
        }
        Ok(())
    }

    /// Dismisses the alert and completes an active callback with Cancelled.
    ///
    /// Returns true if an active callback was consumed. The alert may remain in UIKit's
    /// dismissal animation after this method returns.
    pub fn dismiss(&mut self, animated: bool) -> Result<bool, IosAlertError> {
        if MainThreadMarker::new().is_none() {
            return Err(IosAlertError::MainThreadUnavailable);
        }
        if self.presented && self.alert.presentingViewController().is_some() {
            self.alert
                .dismissViewControllerAnimated_completion(animated, None);
        }
        let completed = self.completion.complete(IosAlertResult::Cancelled);
        if self.presented {
            self.clear_delegates();
        }
        Ok(completed)
    }

    /// Borrows the native UIAlertController for UIKit operations not covered by this wrapper.
    pub fn native_alert_controller(&self) -> &UIAlertController {
        &self.alert
    }

    fn ensure_mutable(&self) -> Result<(), IosAlertError> {
        if self.presented {
            return Err(IosAlertError::AlreadyPresented);
        }
        if MainThreadMarker::new().is_none() {
            return Err(IosAlertError::MainThreadUnavailable);
        }
        Ok(())
    }

    fn clear_delegates(&self) {
        if let Some(popover) = self.alert.popoverPresentationController() {
            // SAFETY: This clears the weak delegate while the session is on the main thread.
            unsafe { popover.setDelegate(None) };
        }
        if let Some(presentation) = self.alert.presentationController() {
            // SAFETY: This clears the weak delegate while the session is on the main thread.
            unsafe { presentation.setDelegate(None) };
        }
        if let Ok(fields) = self.text_fields.try_borrow() {
            for field in fields.iter() {
                // SAFETY: The session is on the main thread and the exact weak delegate is being
                // cleared before its retained owner is released.
                field.setDelegate(None);
            }
        }
    }
}

impl Drop for IosAlertSession<'_> {
    fn drop(&mut self) {
        let had_callback = self.completion.detach();
        self.clear_delegates();
        if had_callback
            && self.presented
            && MainThreadMarker::new().is_some()
            && self.alert.presentingViewController().is_some()
        {
            self.alert
                .dismissViewControllerAnimated_completion(false, None);
        }
    }
}

fn native_keyboard_type(value: IosAlertKeyboardType) -> UIKeyboardType {
    match value {
        IosAlertKeyboardType::Default => UIKeyboardType::Default,
        IosAlertKeyboardType::NumberPad => UIKeyboardType::NumberPad,
        IosAlertKeyboardType::DecimalPad => UIKeyboardType::DecimalPad,
        IosAlertKeyboardType::PhonePad => UIKeyboardType::PhonePad,
        IosAlertKeyboardType::EmailAddress => UIKeyboardType::EmailAddress,
        IosAlertKeyboardType::Url => UIKeyboardType::URL,
    }
}

fn valid_rect(rect: CGRect) -> bool {
    valid_rect_components(
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    )
}

fn snapshot_text_fields(
    fields: &Rc<RefCell<Vec<Retained<UITextField>>>>,
) -> Result<Vec<Option<String>>, IosAlertError> {
    let fields = fields
        .try_borrow()
        .map_err(|_| IosAlertError::ResourceExhausted)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(fields.len())
        .map_err(|_| IosAlertError::ResourceExhausted)?;
    for field in fields.iter() {
        values.push(
            field
                .text()
                .as_deref()
                .map(native_string_to_rust)
                .transpose()?,
        );
    }
    Ok(values)
}

fn native_string_to_rust(value: &NSString) -> Result<String, IosAlertError> {
    let length = value.len_utf16();
    if length > usize::from(MAX_TEXT_FIELD_MAX_UTF16_UNITS) {
        return Err(IosAlertError::InvalidNativeText);
    }
    let mut units = Vec::new();
    units
        .try_reserve_exact(length)
        .map_err(|_| IosAlertError::ResourceExhausted)?;
    units.resize(length, 0);
    if length != 0 {
        let buffer = NonNull::new(units.as_mut_ptr()).ok_or(IosAlertError::ResourceExhausted)?;
        // SAFETY: buffer points to length initialized writable UTF-16 units, and the requested
        // range covers exactly the live NSString's UTF-16 contents.
        unsafe { value.getCharacters_range(buffer, NSRange::new(0, length)) };
    }
    decode_utf16(&units).map_err(|error| match error {
        Utf16Error::Invalid => IosAlertError::InvalidNativeText,
        Utf16Error::ResourceExhausted => IosAlertError::ResourceExhausted,
    })
}

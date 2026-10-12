use core::cell::RefCell;

use framework_core::Result;
use framework_ui::TextFieldInputMode;

pub(crate) type TextFieldCallback = Box<dyn FnMut(Result<Option<String>>)>;
pub(crate) type TextEditorCallback = Box<dyn FnMut(Result<String>)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KeyboardHint {
    Default,
    NumberPad,
    DecimalPad,
}

pub(crate) const fn input_traits(mode: TextFieldInputMode) -> (bool, KeyboardHint) {
    match mode {
        TextFieldInputMode::Text => (false, KeyboardHint::Default),
        TextFieldInputMode::Password => (true, KeyboardHint::Default),
        TextFieldInputMode::Number => (false, KeyboardHint::NumberPad),
        TextFieldInputMode::Decimal => (false, KeyboardHint::DecimalPad),
    }
}

#[cfg(target_os = "ios")]
fn catch_text_panic(callback: impl FnOnce()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(not(target_os = "ios"))]
fn catch_text_panic(callback: impl FnOnce()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

pub(crate) fn invoke_text_callback(
    callback: &RefCell<TextFieldCallback>,
    text: Result<Option<String>>,
) {
    catch_text_panic(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback(text);
        }
    });
}

pub(crate) fn invoke_text_editor_callback(
    callback: &RefCell<TextEditorCallback>,
    text: Result<String>,
) {
    catch_text_panic(|| {
        if let Ok(mut callback) = callback.try_borrow_mut() {
            callback(text);
        }
    });
}

pub(crate) fn replace_text_editor_callback(
    callback: &RefCell<TextEditorCallback>,
    replacement: TextEditorCallback,
) -> Result<()> {
    let mut callback = callback
        .try_borrow_mut()
        .map_err(|_| framework_core::Error::new(framework_core::ErrorKind::Unavailable))?;
    *callback = replacement;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        KeyboardHint, TextEditorCallback, TextFieldCallback, input_traits, invoke_text_callback,
        invoke_text_editor_callback, replace_text_editor_callback,
    };
    use core::cell::{Cell, RefCell};
    use framework_core::{Error, ErrorKind, Result};
    use framework_ui::{
        ButtonControl, Frame, LabelControl, TextFieldControl, TextFieldInputMode, ToggleControl,
        UiBackend, UiClient,
    };
    use std::boxed::Box;
    use std::rc::{Rc, Weak};

    type CallbackCell = RefCell<TextFieldCallback>;
    type Callback = Rc<CallbackCell>;
    type WeakCallback = Weak<CallbackCell>;
    type CallbackSlot = Rc<RefCell<Option<WeakCallback>>>;

    type EditorCallbackCell = RefCell<TextEditorCallback>;
    type EditorCallback = Rc<EditorCallbackCell>;
    type WeakEditorCallback = Weak<EditorCallbackCell>;
    type EditorCallbackSlot = Rc<RefCell<Option<WeakEditorCallback>>>;

    struct ModeField {
        input_mode: TextFieldInputMode,
        text: Option<String>,
        action: Box<dyn FnMut(Result<Option<String>>)>,
        submit_action: Option<TextFieldCallback>,
    }

    impl ModeField {
        fn user_edit(&mut self, text: Option<&str>) {
            self.text = text.map(String::from);
            (self.action)(Ok(self.text.clone()));
        }

        fn user_submit(&mut self, text: Option<&str>) {
            self.text = text.map(String::from);
            if let Some(action) = &mut self.submit_action {
                action(Ok(self.text.clone()));
            }
        }
    }

    impl TextFieldControl for ModeField {
        fn text(&self) -> Result<Option<String>> {
            Ok(self.text.clone())
        }

        fn set_text(&mut self, text: Option<&str>) -> Result<()> {
            self.text = text.map(String::from);
            Ok(())
        }

        fn set_submit_action<F>(&mut self, action: F) -> Result<()>
        where
            F: FnMut(Result<Option<String>>) + 'static,
        {
            self.submit_action = Some(Box::new(action));
            Ok(())
        }
    }

    struct UnsupportedControl;
    struct ModeView;

    impl LabelControl for UnsupportedControl {
        fn set_text(&mut self, _text: &str) -> Result<()> {
            Err(Error::new(ErrorKind::Unsupported))
        }
    }

    impl ButtonControl for UnsupportedControl {
        fn set_title(&mut self, _title: &str) -> Result<()> {
            Err(Error::new(ErrorKind::Unsupported))
        }
    }

    impl ToggleControl for UnsupportedControl {
        fn is_on(&self) -> Result<bool> {
            Err(Error::new(ErrorKind::Unsupported))
        }

        fn set_state(&mut self, _on: bool) -> Result<()> {
            Err(Error::new(ErrorKind::Unsupported))
        }
    }

    impl TextFieldControl for UnsupportedControl {
        fn text(&self) -> Result<Option<String>> {
            Ok(None)
        }

        fn set_text(&mut self, _text: Option<&str>) -> Result<()> {
            Err(Error::new(ErrorKind::Unsupported))
        }
    }

    struct ModeBackend;

    impl UiBackend for ModeBackend {
        type View = ModeView;
        type Label = UnsupportedControl;
        type Button = UnsupportedControl;
        type Toggle = UnsupportedControl;
        type TextField = ModeField;

        fn create_view(&mut self, _frame: Frame) -> Result<Self::View> {
            Ok(ModeView)
        }

        fn add_label(
            &mut self,
            _parent: &mut Self::View,
            _frame: Frame,
            _text: &str,
        ) -> Result<Self::Label> {
            Err(Error::new(ErrorKind::Unsupported))
        }

        fn add_button<F>(
            &mut self,
            _parent: &mut Self::View,
            _frame: Frame,
            _title: &str,
            action: F,
        ) -> Result<Self::Button>
        where
            F: FnMut() + 'static,
        {
            drop(action);
            Err(Error::new(ErrorKind::Unsupported))
        }

        fn add_toggle<F>(
            &mut self,
            _parent: &mut Self::View,
            _frame: Frame,
            _initial_on: bool,
            action: F,
        ) -> Result<Self::Toggle>
        where
            F: FnMut(bool) + 'static,
        {
            drop(action);
            Err(Error::new(ErrorKind::Unsupported))
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
            _parent: &mut Self::View,
            _frame: Frame,
            input_mode: TextFieldInputMode,
            initial_text: Option<&str>,
            _placeholder: Option<&str>,
            action: F,
        ) -> Result<Self::TextField>
        where
            F: FnMut(Result<Option<String>>) + 'static,
        {
            Ok(ModeField {
                input_mode,
                text: initial_text.map(String::from),
                action: Box::new(action),
                submit_action: None,
            })
        }
    }

    struct EnabledControl {
        enabled: bool,
        on: bool,
        text: Option<String>,
        action: Box<dyn FnMut()>,
    }

    impl EnabledControl {
        fn new(action: impl FnMut() + 'static) -> Self {
            Self {
                enabled: true,
                on: false,
                text: None,
                action: Box::new(action),
            }
        }

        fn user_event(&mut self) {
            if self.enabled {
                (self.action)();
            }
        }

        fn user_change(&mut self, on: bool) {
            if self.enabled {
                self.on = on;
                (self.action)();
            }
        }

        fn user_edit(&mut self, text: &str) {
            if self.enabled {
                self.text = Some(String::from(text));
                (self.action)();
            }
        }
    }

    impl ButtonControl for EnabledControl {
        fn is_enabled(&self) -> Result<bool> {
            Ok(self.enabled)
        }

        fn set_enabled(&mut self, enabled: bool) -> Result<()> {
            self.enabled = enabled;
            Ok(())
        }

        fn set_title(&mut self, _title: &str) -> Result<()> {
            Ok(())
        }
    }

    impl ToggleControl for EnabledControl {
        fn is_enabled(&self) -> Result<bool> {
            Ok(self.enabled)
        }

        fn set_enabled(&mut self, enabled: bool) -> Result<()> {
            self.enabled = enabled;
            Ok(())
        }

        fn is_on(&self) -> Result<bool> {
            Ok(self.on)
        }

        fn set_state(&mut self, on: bool) -> Result<()> {
            self.on = on;
            Ok(())
        }
    }

    impl TextFieldControl for EnabledControl {
        fn is_enabled(&self) -> Result<bool> {
            Ok(self.enabled)
        }

        fn set_enabled(&mut self, enabled: bool) -> Result<()> {
            self.enabled = enabled;
            Ok(())
        }

        fn text(&self) -> Result<Option<String>> {
            Ok(self.text.clone())
        }

        fn set_text(&mut self, text: Option<&str>) -> Result<()> {
            self.text = text.map(String::from);
            Ok(())
        }
    }

    #[test]
    fn input_modes_are_hints_and_creation_does_not_emit_edits() {
        let mut ui = UiClient::new(ModeBackend);
        let frame = Frame::new(0.0, 0.0, 120.0, 32.0).unwrap();
        let mut parent = ui.create_view(frame).unwrap();

        for input_mode in [
            TextFieldInputMode::Text,
            TextFieldInputMode::Password,
            TextFieldInputMode::Number,
            TextFieldInputMode::Decimal,
        ] {
            let calls = Rc::new(Cell::new(0));
            let action_calls = calls.clone();
            let mut field = ui
                .add_text_field_with_mode(
                    &mut parent,
                    frame,
                    input_mode,
                    Some("not numeric"),
                    None,
                    move |_| action_calls.set(action_calls.get() + 1),
                )
                .unwrap();

            assert_eq!(field.input_mode, input_mode);
            assert_eq!(field.text(), Ok(Some(String::from("not numeric"))));
            assert_eq!(calls.get(), 0);
            field.user_edit(Some("still text"));
            assert_eq!(calls.get(), 1);
        }
    }

    #[test]
    fn input_modes_map_to_secure_entry_and_keyboard_hints() {
        assert_eq!(
            input_traits(TextFieldInputMode::Text),
            (false, KeyboardHint::Default)
        );
        assert_eq!(
            input_traits(TextFieldInputMode::Password),
            (true, KeyboardHint::Default)
        );
        assert_eq!(
            input_traits(TextFieldInputMode::Number),
            (false, KeyboardHint::NumberPad)
        );
        assert_eq!(
            input_traits(TextFieldInputMode::Decimal),
            (false, KeyboardHint::DecimalPad)
        );
    }

    #[test]
    fn control_enabled_state_is_silent_and_rejects_disabled_user_events() {
        let button_calls = Rc::new(Cell::new(0));
        let button_action_calls = button_calls.clone();
        let mut button =
            EnabledControl::new(move || button_action_calls.set(button_action_calls.get() + 1));
        let toggle_calls = Rc::new(Cell::new(0));
        let toggle_action_calls = toggle_calls.clone();
        let mut toggle =
            EnabledControl::new(move || toggle_action_calls.set(toggle_action_calls.get() + 1));
        let field_calls = Rc::new(Cell::new(0));
        let field_action_calls = field_calls.clone();
        let mut field =
            EnabledControl::new(move || field_action_calls.set(field_action_calls.get() + 1));
        field.set_text(Some("initial")).unwrap();

        assert_eq!(ButtonControl::is_enabled(&button), Ok(true));
        assert_eq!(ToggleControl::is_enabled(&toggle), Ok(true));
        assert_eq!(TextFieldControl::is_enabled(&field), Ok(true));
        ButtonControl::set_enabled(&mut button, false).unwrap();
        ToggleControl::set_enabled(&mut toggle, false).unwrap();
        TextFieldControl::set_enabled(&mut field, false).unwrap();
        button.user_event();
        toggle.user_change(true);
        field.user_edit("ignored");
        assert_eq!(toggle.is_on(), Ok(false));
        assert_eq!(field.text(), Ok(Some(String::from("initial"))));
        assert_eq!(button_calls.get(), 0);
        assert_eq!(toggle_calls.get(), 0);
        assert_eq!(field_calls.get(), 0);

        toggle.set_state(true).unwrap();
        field.set_text(Some("programmatic")).unwrap();
        assert_eq!(toggle.is_on(), Ok(true));
        assert_eq!(field.text(), Ok(Some(String::from("programmatic"))));
        assert_eq!(toggle_calls.get(), 0);
        assert_eq!(field_calls.get(), 0);

        ButtonControl::set_enabled(&mut button, true).unwrap();
        ToggleControl::set_enabled(&mut toggle, true).unwrap();
        TextFieldControl::set_enabled(&mut field, true).unwrap();
        button.user_event();
        toggle.user_change(false);
        field.user_edit("user");
        assert_eq!(button_calls.get(), 1);
        assert_eq!(toggle_calls.get(), 1);
        assert_eq!(field_calls.get(), 1);

        let mut unsupported = UnsupportedControl;
        assert_eq!(
            ButtonControl::is_enabled(&unsupported),
            Err(Error::new(ErrorKind::Unsupported))
        );
        assert_eq!(
            ButtonControl::set_enabled(&mut unsupported, false),
            Err(Error::new(ErrorKind::Unsupported))
        );
        assert_eq!(
            ToggleControl::is_enabled(&unsupported),
            Err(Error::new(ErrorKind::Unsupported))
        );
        assert_eq!(
            ToggleControl::set_enabled(&mut unsupported, false),
            Err(Error::new(ErrorKind::Unsupported))
        );
        assert_eq!(
            TextFieldControl::is_enabled(&unsupported),
            Err(Error::new(ErrorKind::Unsupported))
        );
        assert_eq!(
            TextFieldControl::set_enabled(&mut unsupported, false),
            Err(Error::new(ErrorKind::Unsupported))
        );
    }

    #[test]
    fn submit_action_receives_owned_nil_empty_and_unicode_snapshots() {
        let snapshots = Rc::new(RefCell::new(std::vec::Vec::new()));
        let action_snapshots = snapshots.clone();
        let mut ui = UiClient::new(ModeBackend);
        let frame = Frame::new(0.0, 0.0, 120.0, 32.0).unwrap();
        let mut parent = ui.create_view(frame).unwrap();
        let mut field = ui
            .add_text_field(&mut parent, frame, None, None, |_| {})
            .unwrap();
        let replaced_calls = Rc::new(Cell::new(false));
        let old_action_calls = replaced_calls.clone();
        ui.set_submit_action(&mut field, move |_| old_action_calls.set(true))
            .unwrap();

        ui.set_submit_action(&mut field, move |value| {
            action_snapshots.borrow_mut().push(value);
        })
        .unwrap();
        field.user_submit(Some("A\0😀"));
        field.user_submit(Some(""));
        field.user_submit(None);
        assert_eq!(
            *snapshots.borrow(),
            [
                Ok(Some(String::from("A\0😀"))),
                Ok(Some(String::new())),
                Ok(None)
            ]
        );
        assert!(!replaced_calls.get());
    }

    #[test]
    fn unsupported_submit_action_is_dropped_and_reported() {
        struct DropSignal(Rc<Cell<bool>>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let dropped = Rc::new(Cell::new(false));
        let signal = DropSignal(dropped.clone());
        let mut field = UnsupportedControl;
        let result = field.set_submit_action(move |_| {
            let _ = &signal;
        });

        assert_eq!(result, Err(Error::new(ErrorKind::Unsupported)));
        assert!(dropped.get());
    }

    #[test]
    fn callback_panic_is_contained_and_releases_its_borrow() {
        let callback = RefCell::new(
            Box::new(|_| -> () { panic!("text callback panic") }) as TextFieldCallback
        );
        invoke_text_callback(&callback, Ok(Some(String::from("edit"))));
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn reentrant_callback_is_ignored_while_mutably_borrowed() {
        let callback_slot: CallbackSlot = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = callback_slot.clone();
        let nested_calls = calls.clone();
        let callback: Callback = Rc::new(RefCell::new(Box::new(
            move |text: Result<Option<String>>| {
                nested_calls.set(nested_calls.get() + 1);
                if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                    invoke_text_callback(&callback, text.clone());
                }
            },
        )));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&callback));

        invoke_text_callback(&callback, Ok(Some(String::from("edit"))));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn invalid_utf16_error_reaches_callback_unchanged() {
        let received = Rc::new(Cell::new(None::<Error>));
        let action_received = received.clone();
        let callback = RefCell::new(Box::new(move |text: Result<Option<String>>| {
            action_received.set(Some(text.unwrap_err()));
        }) as TextFieldCallback);
        let invalid = Error::new(ErrorKind::InvalidInput);

        invoke_text_callback(&callback, Err(invalid));
        assert_eq!(received.get(), Some(invalid));
    }

    #[test]
    fn editor_callback_panic_is_contained_and_releases_its_borrow() {
        let callback = RefCell::new(
            Box::new(|_| -> () { panic!("text editor callback panic") }) as TextEditorCallback
        );
        invoke_text_editor_callback(&callback, Ok(String::from("edit")));
        assert!(callback.try_borrow_mut().is_ok());
    }

    #[test]
    fn editor_reentrant_callback_is_ignored_while_mutably_borrowed() {
        let callback_slot: EditorCallbackSlot = Rc::new(RefCell::new(None));
        let calls = Rc::new(Cell::new(0));
        let nested_slot = callback_slot.clone();
        let nested_calls = calls.clone();
        let callback: EditorCallback =
            Rc::new(RefCell::new(Box::new(move |text: Result<String>| {
                nested_calls.set(nested_calls.get() + 1);
                if let Some(callback) = nested_slot.borrow().as_ref().and_then(Weak::upgrade) {
                    invoke_text_editor_callback(&callback, text.clone());
                }
            })));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&callback));

        invoke_text_editor_callback(&callback, Ok(String::from("edit")));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn editor_callback_receives_invalid_utf16_error_unchanged() {
        let received = Rc::new(Cell::new(None::<Error>));
        let action_received = received.clone();
        let callback = RefCell::new(Box::new(move |text: Result<String>| {
            action_received.set(Some(text.unwrap_err()));
        }) as TextEditorCallback);
        let invalid = Error::new(ErrorKind::InvalidInput);

        invoke_text_editor_callback(&callback, Err(invalid));
        assert_eq!(received.get(), Some(invalid));
    }

    #[test]
    fn editor_callback_replacement_releases_old_callback_and_drop_releases_new() {
        struct DropSignal(Rc<Cell<bool>>);
        impl Drop for DropSignal {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let old_dropped = Rc::new(Cell::new(false));
        let old_signal = DropSignal(old_dropped.clone());
        let callback = RefCell::new(Box::new(move |_| {
            let _ = &old_signal;
        }) as TextEditorCallback);
        let new_dropped = Rc::new(Cell::new(false));
        let new_signal = DropSignal(new_dropped.clone());
        replace_text_editor_callback(
            &callback,
            Box::new(move |_| {
                let _ = &new_signal;
            }),
        )
        .unwrap();
        assert!(old_dropped.get());
        assert!(!new_dropped.get());

        drop(callback);
        assert!(new_dropped.get());
    }

    #[test]
    fn editor_replacement_while_active_is_rejected_and_preserves_current_callback() {
        let calls = Rc::new(Cell::new(0));
        let current_calls = calls.clone();
        let callback = RefCell::new(Box::new(move |_| current_calls.set(1)) as TextEditorCallback);
        let _active = callback.borrow_mut();

        assert_eq!(
            replace_text_editor_callback(&callback, Box::new(|_| {})),
            Err(Error::new(ErrorKind::Unavailable))
        );
        assert_eq!(calls.get(), 0);
    }
}

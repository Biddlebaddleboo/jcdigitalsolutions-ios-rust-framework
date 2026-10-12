#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable frame values and a static contract for native UI controls."]

use alloc::string::String;
use framework_core::{Error, ErrorKind, Result};

extern crate alloc;

#[cfg(test)]
extern crate std;

/// A rectangular frame in a parent's local coordinate system.
///
/// The backend defines the logical coordinate unit. A frame has a finite origin and finite,
/// non-negative dimensions. This value does not define layout, scaling, insets, or pixel rounding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Frame {
    /// Creates a frame, rejecting non-finite fields and negative dimensions.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self> {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width < 0.0
            || height < 0.0
        {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    /// Returns the horizontal origin in backend-defined logical units.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the vertical origin in backend-defined logical units.
    pub const fn y(self) -> f64 {
        self.y
    }

    /// Returns the non-negative width in backend-defined logical units.
    pub const fn width(self) -> f64 {
        self.width
    }

    /// Returns the non-negative height in backend-defined logical units.
    pub const fn height(self) -> f64 {
        self.height
    }

    /// Returns the positive-area intersection with another frame in the same coordinate space.
    ///
    /// Zero-size frames and edge- or corner-touching frames return `Ok(None)`. The maximum edges
    /// are computed as origin plus dimension; non-finite edges or positive dimensions too small
    /// to advance their origin return `InvalidInput`. The returned dimensions and their maximum
    /// edges must also be finite and representable. This operation does not transform coordinate
    /// spaces.
    pub fn intersection(self, other: Self) -> Result<Option<Self>> {
        let (self_max_x, self_max_y) = self.checked_max_edges()?;
        let (other_max_x, other_max_y) = other.checked_max_edges()?;

        if self.width == 0.0 || self.height == 0.0 || other.width == 0.0 || other.height == 0.0 {
            return Ok(None);
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let max_x = self_max_x.min(other_max_x);
        let max_y = self_max_y.min(other_max_y);
        if max_x <= x || max_y <= y {
            return Ok(None);
        }

        let width = max_x - x;
        let height = max_y - y;
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Err(Error::new(ErrorKind::InvalidInput));
        }

        let intersection = Self::new(x, y, width, height)?;
        let (intersection_max_x, intersection_max_y) = intersection.checked_max_edges()?;
        if intersection_max_x != max_x || intersection_max_y != max_y {
            return Err(Error::new(ErrorKind::InvalidInput));
        }

        Ok(Some(intersection))
    }

    fn checked_max_edges(self) -> Result<(f64, f64)> {
        let max_x = self.x + self.width;
        let max_y = self.y + self.height;
        if !max_x.is_finite()
            || !max_y.is_finite()
            || (self.width > 0.0 && max_x <= self.x)
            || (self.height > 0.0 && max_y <= self.y)
        {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok((max_x, max_y))
    }
}

/// Scaled vertical metrics for one system font size.
///
/// Metric values use the backend's logical units. This value does not describe glyph shaping,
/// text width, line breaks, or paragraph layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontMetrics {
    point_size: f64,
    ascent: f64,
    descent: f64,
    leading: f64,
}

impl FontMetrics {
    /// Creates metrics with a finite positive point size and finite native metric values.
    pub fn new(point_size: f64, ascent: f64, descent: f64, leading: f64) -> Result<Self> {
        if !point_size.is_finite()
            || point_size <= 0.0
            || !ascent.is_finite()
            || !descent.is_finite()
            || !leading.is_finite()
        {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(Self {
            point_size,
            ascent,
            descent,
            leading,
        })
    }

    /// Returns the requested font size in points.
    pub const fn point_size(self) -> f64 {
        self.point_size
    }

    /// Returns the scaled ascent in backend-defined logical units.
    pub const fn ascent(self) -> f64 {
        self.ascent
    }

    /// Returns the scaled descent in backend-defined logical units.
    pub const fn descent(self) -> f64 {
        self.descent
    }

    /// Returns the scaled leading in backend-defined logical units.
    pub const fn leading(self) -> f64 {
        self.leading
    }
}

/// A backend that can query vertical metrics for its system UI font.
pub trait TextMetricsBackend {
    /// Returns system-font ascent, descent, and leading at `point_size`.
    ///
    /// The point size must be finite and positive. Metric units and font selection are
    /// backend-defined; this operation does not shape text or compute line layout.
    fn system_font_metrics(&mut self, point_size: f64) -> Result<FontMetrics>;
}

/// A mutable text-label handle returned by a [`UiBackend`].
pub trait LabelControl {
    /// Replaces the label text during this synchronous call.
    ///
    /// The backend must not retain the borrowed `text`; it must copy or convert it to a
    /// backend-owned representation before this call returns. An empty string requests empty
    /// text, not a backend-specific `None` value.
    fn set_text(&mut self, text: &str) -> Result<()>;
}

/// A mutable button handle returned by a [`UiBackend`].
pub trait ButtonControl {
    /// Returns whether the button currently accepts user interaction.
    ///
    /// Backends that do not expose enabled state return `Unsupported`.
    fn is_enabled(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Unsupported))
    }

    /// Enables or disables user interaction without invoking the button action.
    ///
    /// Backends that do not expose enabled state return `Unsupported`.
    fn set_enabled(&mut self, _enabled: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Unsupported))
    }

    /// Replaces the button title during this synchronous call.
    ///
    /// The backend must not retain the borrowed `title`; it must copy or convert it to a
    /// backend-owned representation before this call returns. An empty string requests an empty
    /// title, not a backend-specific `None` value.
    fn set_title(&mut self, title: &str) -> Result<()>;
}

/// A mutable boolean toggle handle returned by a [`UiBackend`].
pub trait ToggleControl {
    /// Returns whether the toggle currently accepts user interaction.
    ///
    /// Backends that do not expose enabled state return `Unsupported`.
    fn is_enabled(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Unsupported))
    }

    /// Enables or disables user interaction without invoking the value-change callback.
    ///
    /// Backends that do not expose enabled state return `Unsupported`.
    fn set_enabled(&mut self, _enabled: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Unsupported))
    }

    /// Returns the current toggle state.
    fn is_on(&self) -> Result<bool>;

    /// Changes the state without invoking the user-change callback.
    fn set_state(&mut self, on: bool) -> Result<()>;
}

/// A mutable single-line text field returned by a [`UiBackend`].
pub trait TextFieldControl {
    /// Returns whether the text field currently accepts user interaction.
    ///
    /// Backends that do not expose enabled state return `Unsupported`.
    fn is_enabled(&self) -> Result<bool> {
        Err(Error::new(ErrorKind::Unsupported))
    }

    /// Enables or disables user interaction without invoking the edit or submit callback.
    ///
    /// Backends that do not expose enabled state return `Unsupported`.
    fn set_enabled(&mut self, _enabled: bool) -> Result<()> {
        Err(Error::new(ErrorKind::Unsupported))
    }

    /// Returns an owned snapshot of the current text, preserving `None` versus `Some("")`.
    fn text(&self) -> Result<Option<String>>;

    /// Replaces or clears the field text without invoking its user-edit callback.
    fn set_text(&mut self, text: Option<&str>) -> Result<()>;

    /// Replaces the action invoked when the native field submits its current value.
    ///
    /// On success, the field owns `action` until replacement or handle drop. A submit event passes
    /// a fresh owned snapshot, preserving `None` versus `Some("")`; malformed native text is
    /// reported as `Err`. Callback thread, reentrancy, panic, and native submit-event semantics are
    /// backend-defined. If unsupported or otherwise unavailable, the backend consumes and drops
    /// `action`, returns an error, and keeps any prior action unchanged.
    fn set_submit_action<F>(&mut self, action: F) -> Result<()>
    where
        F: FnMut(Result<Option<String>>) + 'static,
    {
        drop(action);
        Err(Error::new(ErrorKind::Unsupported))
    }
}

/// Semantic input and keyboard hints for a native single-line text field.
///
/// These modes do not parse, validate, or format field values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFieldInputMode {
    /// Ordinary text with the platform's default keyboard.
    Text,
    /// Secure text entry with the platform's default keyboard.
    Password,
    /// Integer-oriented keyboard hint; the value remains text.
    Number,
    /// Decimal-oriented keyboard hint; the value remains text.
    Decimal,
}

/// A compile-time-selected backend for a native container view and controls.
///
/// The contract starts no process service, event loop, executor, or global registration. Each
/// method is synchronous. Backends document their native thread and callback behavior.
pub trait UiBackend {
    /// The native container handle chosen by the backend.
    type View;
    /// The mutable label handle chosen by the backend.
    type Label: LabelControl;
    /// The mutable button handle chosen by the backend.
    type Button: ButtonControl;
    /// The mutable boolean toggle handle chosen by the backend.
    type Toggle: ToggleControl;
    /// The mutable text-field handle chosen by the backend.
    type TextField: TextFieldControl;

    /// Creates a native container view with the supplied frame.
    fn create_view(&mut self, frame: Frame) -> Result<Self::View>;

    /// Creates and attaches a native text label to a caller-owned view.
    ///
    /// `text` is borrowed only for this call. A successful result returns the handle used for
    /// later text updates.
    fn add_label(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        text: &str,
    ) -> Result<Self::Label>;

    /// Creates and attaches a native button with a Rust action callback.
    ///
    /// The backend owns `action` if it returns `Ok`; the returned button handle must keep the
    /// action alive for that handle's lifetime. Dropping the handle ends the callback ownership.
    /// If this method returns `Err`, it consumes and drops `action`. The callback is not required
    /// to be `Send`; callback thread, reentrancy, and panic behavior are backend-defined.
    fn add_button<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        title: &str,
        action: F,
    ) -> Result<Self::Button>
    where
        F: FnMut() + 'static;

    /// Creates and attaches a native boolean toggle with a Rust value-change callback.
    ///
    /// `initial_on` is installed during synchronous setup. The backend owns `action` if it returns
    /// `Ok`; the returned handle must keep it alive for that handle's lifetime. The backend calls
    /// `action` only after a user/native change and passes the resulting value; `set_state` must not
    /// call it. If this method returns `Err`, it consumes and drops `action`. Callback thread,
    /// reentrancy, and panic behavior are backend-defined; the callback need not be `Send`.
    fn add_toggle<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        initial_on: bool,
        action: F,
    ) -> Result<Self::Toggle>
    where
        F: FnMut(bool) + 'static;

    /// Creates and attaches a native single-line text field with an owned edit callback.
    ///
    /// The callback runs after a user/native edit and receives a fresh owned text snapshot for
    /// that event. `None` and `Some("")` remain distinct; malformed native text is reported as
    /// `Err` instead of being replaced. Programmatic [`TextFieldControl::set_text`] calls must
    /// not invoke the callback. The backend owns `action` if it returns `Ok`; dropping the
    /// returned handle ends callback ownership. If this method returns `Err`, it consumes and
    /// drops `action`. Callback thread, reentrancy, and panic behavior are backend-defined; the
    /// callback need not be `Send`.
    fn add_text_field<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        initial_text: Option<&str>,
        placeholder: Option<&str>,
        action: F,
    ) -> Result<Self::TextField>
    where
        F: FnMut(Result<Option<String>>) + 'static;

    /// Creates a text field with a semantic input mode.
    ///
    /// Backends that do not implement a non-`Text` mode return `Unsupported`. Modes are input
    /// hints only and must not parse or rewrite the field value. The default implementation
    /// preserves existing backends for ordinary `Text` input. Returning an error consumes and
    /// drops `action`, as with `add_text_field`.
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
        if input_mode == TextFieldInputMode::Text {
            self.add_text_field(parent, frame, initial_text, placeholder, action)
        } else {
            Err(Error::new(ErrorKind::Unsupported))
        }
    }
}

/// A thin facade over caller-owned, statically selected native UI backend state.
pub struct UiClient<B> {
    backend: B,
}

impl<B: UiBackend> UiClient<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Creates a native container view with the supplied frame.
    pub fn create_view(&mut self, frame: Frame) -> Result<B::View> {
        self.backend.create_view(frame)
    }

    /// Creates and attaches a native text label to a caller-owned view.
    pub fn add_label(
        &mut self,
        parent: &mut B::View,
        frame: Frame,
        text: &str,
    ) -> Result<B::Label> {
        self.backend.add_label(parent, frame, text)
    }

    /// Creates and attaches a native button with a Rust action callback.
    pub fn add_button<F>(
        &mut self,
        parent: &mut B::View,
        frame: Frame,
        title: &str,
        action: F,
    ) -> Result<B::Button>
    where
        F: FnMut() + 'static,
    {
        self.backend.add_button(parent, frame, title, action)
    }

    /// Creates and attaches a native boolean toggle with a Rust value-change callback.
    pub fn add_toggle<F>(
        &mut self,
        parent: &mut B::View,
        frame: Frame,
        initial_on: bool,
        action: F,
    ) -> Result<B::Toggle>
    where
        F: FnMut(bool) + 'static,
    {
        self.backend.add_toggle(parent, frame, initial_on, action)
    }

    /// Creates and attaches a native text field with an owned user-edit callback.
    pub fn add_text_field<F>(
        &mut self,
        parent: &mut B::View,
        frame: Frame,
        initial_text: Option<&str>,
        placeholder: Option<&str>,
        action: F,
    ) -> Result<B::TextField>
    where
        F: FnMut(Result<Option<String>>) + 'static,
    {
        self.backend
            .add_text_field(parent, frame, initial_text, placeholder, action)
    }

    /// Creates and attaches a text field with the supplied semantic input mode.
    pub fn add_text_field_with_mode<F>(
        &mut self,
        parent: &mut B::View,
        frame: Frame,
        input_mode: TextFieldInputMode,
        initial_text: Option<&str>,
        placeholder: Option<&str>,
        action: F,
    ) -> Result<B::TextField>
    where
        F: FnMut(Result<Option<String>>) + 'static,
    {
        self.backend.add_text_field_with_mode(
            parent,
            frame,
            input_mode,
            initial_text,
            placeholder,
            action,
        )
    }

    /// Replaces the action invoked when `field` receives a native submit event.
    pub fn set_submit_action<F>(&mut self, field: &mut B::TextField, action: F) -> Result<()>
    where
        F: FnMut(Result<Option<String>>) + 'static,
    {
        field.set_submit_action(action)
    }

    /// Borrows the explicitly supplied backend.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the explicitly supplied backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::boxed::Box;
    use std::string::String;

    struct TestView(Frame);

    struct TestLabel(String);

    impl LabelControl for TestLabel {
        fn set_text(&mut self, text: &str) -> Result<()> {
            self.0.clear();
            self.0.push_str(text);
            Ok(())
        }
    }

    struct TestButton(Box<dyn FnMut()>);

    impl TestButton {
        fn press(&mut self) {
            (self.0)();
        }
    }

    impl ButtonControl for TestButton {
        fn set_title(&mut self, _title: &str) -> Result<()> {
            Ok(())
        }
    }

    struct TestToggle {
        on: bool,
        action: Box<dyn FnMut(bool)>,
    }

    impl TestToggle {
        fn user_change(&mut self, on: bool) {
            self.on = on;
            (self.action)(on);
        }
    }

    impl ToggleControl for TestToggle {
        fn is_on(&self) -> Result<bool> {
            Ok(self.on)
        }

        fn set_state(&mut self, on: bool) -> Result<()> {
            self.on = on;
            Ok(())
        }
    }

    struct TestTextField {
        text: Option<String>,
        placeholder: Option<String>,
        action: Box<dyn FnMut(Result<Option<String>>)>,
    }

    impl TestTextField {
        fn user_edit(&mut self, text: Option<&str>) {
            self.text = text.map(String::from);
            (self.action)(Ok(self.text.clone()));
        }
    }

    impl TextFieldControl for TestTextField {
        fn text(&self) -> Result<Option<String>> {
            Ok(self.text.clone())
        }

        fn set_text(&mut self, text: Option<&str>) -> Result<()> {
            self.text = text.map(String::from);
            Ok(())
        }
    }

    struct TestBackend;

    impl UiBackend for TestBackend {
        type View = TestView;
        type Label = TestLabel;
        type Button = TestButton;
        type Toggle = TestToggle;
        type TextField = TestTextField;

        fn create_view(&mut self, frame: Frame) -> Result<Self::View> {
            Ok(TestView(frame))
        }

        fn add_label(
            &mut self,
            _parent: &mut Self::View,
            _frame: Frame,
            text: &str,
        ) -> Result<Self::Label> {
            Ok(TestLabel(String::from(text)))
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
            Ok(TestButton(Box::new(action)))
        }

        fn add_toggle<F>(
            &mut self,
            _parent: &mut Self::View,
            _frame: Frame,
            initial_on: bool,
            action: F,
        ) -> Result<Self::Toggle>
        where
            F: FnMut(bool) + 'static,
        {
            Ok(TestToggle {
                on: initial_on,
                action: Box::new(action),
            })
        }

        fn add_text_field<F>(
            &mut self,
            _parent: &mut Self::View,
            _frame: Frame,
            initial_text: Option<&str>,
            placeholder: Option<&str>,
            action: F,
        ) -> Result<Self::TextField>
        where
            F: FnMut(Result<Option<String>>) + 'static,
        {
            Ok(TestTextField {
                text: initial_text.map(String::from),
                placeholder: placeholder.map(String::from),
                action: Box::new(action),
            })
        }
    }

    #[test]
    fn frame_accepts_finite_origin_and_nonnegative_dimensions() {
        let frame = Frame::new(-3.5, 2.25, 0.0, 44.0).unwrap();
        assert_eq!(frame.x(), -3.5);
        assert_eq!(frame.y(), 2.25);
        assert_eq!(frame.width(), 0.0);
        assert_eq!(frame.height(), 44.0);
    }

    #[test]
    fn frame_rejects_nonfinite_fields_and_negative_dimensions() {
        for fields in [
            (f64::NAN, 0.0, 1.0, 1.0),
            (0.0, f64::INFINITY, 1.0, 1.0),
            (0.0, 0.0, f64::NEG_INFINITY, 1.0),
            (0.0, 0.0, 1.0, -1.0),
        ] {
            assert_eq!(
                Frame::new(fields.0, fields.1, fields.2, fields.3),
                Err(Error::new(ErrorKind::InvalidInput))
            );
        }
    }

    #[test]
    fn frame_intersection_returns_positive_area_overlap() {
        let first = Frame::new(-2.0, 1.0, 7.0, 6.0).unwrap();
        let second = Frame::new(1.0, -2.0, 5.0, 6.0).unwrap();

        assert_eq!(
            first.intersection(second),
            Ok(Some(Frame::new(1.0, 1.0, 4.0, 3.0).unwrap()))
        );
        assert_eq!(first.intersection(first), Ok(Some(first)));
    }

    #[test]
    fn frame_intersection_returns_none_for_disjoint_touching_and_empty_frames() {
        let frame = Frame::new(0.0, 0.0, 2.0, 2.0).unwrap();
        let disjoint = Frame::new(3.0, 0.0, 1.0, 1.0).unwrap();
        let horizontal_touch = Frame::new(2.0, 0.0, 1.0, 1.0).unwrap();
        let vertical_touch = Frame::new(0.0, 2.0, 1.0, 1.0).unwrap();
        let corner_touch = Frame::new(2.0, 2.0, 1.0, 1.0).unwrap();
        let zero_width = Frame::new(1.0, 0.0, 0.0, 1.0).unwrap();
        let negative_zero_width = Frame::new(1.0, 0.0, -0.0, 1.0).unwrap();
        let zero_height = Frame::new(0.0, 1.0, 1.0, 0.0).unwrap();

        for other in [
            disjoint,
            horizontal_touch,
            vertical_touch,
            corner_touch,
            zero_width,
            negative_zero_width,
            zero_height,
        ] {
            assert_eq!(frame.intersection(other), Ok(None));
        }
    }

    #[test]
    fn frame_intersection_rejects_unrepresentable_edges() {
        let overflowing_edge = Frame::new(f64::MAX, 0.0, f64::MAX, 1.0).unwrap();
        let precision_collapsed_edge = Frame::new(1.0e308, 0.0, 1.0, 1.0).unwrap();
        let ordinary = Frame::new(0.0, 0.0, 1.0, 1.0).unwrap();
        let expected = Err(Error::new(ErrorKind::InvalidInput));

        assert_eq!(ordinary.intersection(overflowing_edge), expected);
        assert_eq!(ordinary.intersection(precision_collapsed_edge), expected);
    }

    #[test]
    fn frame_intersection_handles_finite_extreme_edges() {
        let first = Frame::new(-f64::MAX, -f64::MAX, f64::MAX, f64::MAX).unwrap();
        let second = Frame::new(
            -f64::MAX / 2.0,
            -f64::MAX / 2.0,
            f64::MAX / 2.0,
            f64::MAX / 2.0,
        )
        .unwrap();

        assert_eq!(first.intersection(second), Ok(Some(second)));
    }

    #[test]
    fn font_metrics_accept_finite_native_values() {
        let metrics = FontMetrics::new(16.0, 18.25, 4.5, 1.0).unwrap();
        assert_eq!(metrics.point_size(), 16.0);
        assert_eq!(metrics.ascent(), 18.25);
        assert_eq!(metrics.descent(), 4.5);
        assert_eq!(metrics.leading(), 1.0);

        let signed = FontMetrics::new(12.0, -1.0, 0.0, -0.5).unwrap();
        assert_eq!(signed.ascent(), -1.0);
        assert_eq!(signed.leading(), -0.5);
    }

    #[test]
    fn font_metrics_reject_nonpositive_size_and_nonfinite_values() {
        let expected = Err(Error::new(ErrorKind::InvalidInput));
        for point_size in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(FontMetrics::new(point_size, 1.0, 1.0, 0.0), expected);
        }
        for values in [
            (f64::NAN, 1.0, 0.0),
            (1.0, f64::INFINITY, 0.0),
            (1.0, 1.0, f64::NEG_INFINITY),
        ] {
            assert_eq!(
                FontMetrics::new(12.0, values.0, values.1, values.2),
                expected
            );
        }
    }

    #[test]
    fn toggle_initial_state_and_programmatic_changes_do_not_call_action() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let action_calls = calls.clone();
        let mut ui = UiClient::new(TestBackend);
        let frame = Frame::new(0.0, 0.0, 30.0, 40.0).unwrap();
        let mut view = ui.create_view(frame).unwrap();
        let mut toggle = ui
            .add_toggle(&mut view, frame, false, move |_| {
                action_calls.set(action_calls.get() + 1);
            })
            .unwrap();

        assert_eq!(toggle.is_on(), Ok(false));
        toggle.set_state(true).unwrap();
        assert_eq!(toggle.is_on(), Ok(true));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn toggle_callback_receives_each_new_user_value() {
        let values = std::rc::Rc::new(std::cell::RefCell::new(std::vec::Vec::new()));
        let action_values = values.clone();
        let mut ui = UiClient::new(TestBackend);
        let frame = Frame::new(0.0, 0.0, 30.0, 40.0).unwrap();
        let mut view = ui.create_view(frame).unwrap();
        let mut toggle = ui
            .add_toggle(&mut view, frame, false, move |on| {
                action_values.borrow_mut().push(on);
            })
            .unwrap();

        toggle.user_change(true);
        toggle.user_change(false);
        assert_eq!(*values.borrow(), [true, false]);
    }

    #[test]
    fn text_field_preserves_initial_nil_and_empty_text() {
        let mut ui = UiClient::new(TestBackend);
        let frame = Frame::new(0.0, 0.0, 120.0, 32.0).unwrap();
        let mut view = ui.create_view(frame).unwrap();
        let nil = ui
            .add_text_field(&mut view, frame, None, Some("Name"), |_| {})
            .unwrap();
        let empty = ui
            .add_text_field(&mut view, frame, Some(""), None, |_| {})
            .unwrap();

        assert_eq!(nil.text(), Ok(None));
        assert_eq!(nil.placeholder.as_deref(), Some("Name"));
        assert_eq!(empty.text(), Ok(Some(String::new())));
        assert_eq!(empty.placeholder, None);
    }

    #[test]
    fn text_field_programmatic_changes_are_silent() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let action_calls = calls.clone();
        let mut ui = UiClient::new(TestBackend);
        let frame = Frame::new(0.0, 0.0, 120.0, 32.0).unwrap();
        let mut view = ui.create_view(frame).unwrap();
        let mut field = ui
            .add_text_field(&mut view, frame, None, None, move |_| {
                action_calls.set(action_calls.get() + 1);
            })
            .unwrap();

        field.set_text(Some("programmatic")).unwrap();
        field.set_text(None).unwrap();
        assert_eq!(field.text(), Ok(None));
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn text_field_callback_receives_owned_exact_edit_snapshots() {
        let snapshots = std::rc::Rc::new(std::cell::RefCell::new(std::vec::Vec::new()));
        let action_snapshots = snapshots.clone();
        let mut ui = UiClient::new(TestBackend);
        let frame = Frame::new(0.0, 0.0, 120.0, 32.0).unwrap();
        let mut view = ui.create_view(frame).unwrap();
        let mut field = ui
            .add_text_field(&mut view, frame, None, None, move |value| {
                action_snapshots.borrow_mut().push(value);
            })
            .unwrap();

        field.user_edit(Some("A\0😀"));
        field.user_edit(Some(""));
        field.user_edit(None);
        assert_eq!(
            *snapshots.borrow(),
            [
                Ok(Some(String::from("A\0😀"))),
                Ok(Some(String::new())),
                Ok(None)
            ]
        );
    }

    #[test]
    fn client_uses_supplied_backend_and_control_handles() {
        let mut ui = UiClient::new(TestBackend);
        let frame = Frame::new(1.0, 2.0, 30.0, 40.0).unwrap();
        let mut view = ui.create_view(frame).unwrap();
        assert_eq!(view.0, frame);

        let mut label = ui.add_label(&mut view, frame, "before").unwrap();
        label.set_text("after").unwrap();
        assert_eq!(label.0, "after");

        let presses = std::rc::Rc::new(std::cell::Cell::new(0));
        let action_presses = presses.clone();
        let mut button = ui
            .add_button(&mut view, frame, "Run", move || {
                action_presses.set(action_presses.get() + 1);
            })
            .unwrap();
        button.press();
        assert_eq!(presses.get(), 1);
    }
}

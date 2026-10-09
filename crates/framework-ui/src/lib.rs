#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable frame values and a static contract for native UI controls."]

use framework_core::{Error, ErrorKind, Result};

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
    /// Replaces the button title during this synchronous call.
    ///
    /// The backend must not retain the borrowed `title`; it must copy or convert it to a
    /// backend-owned representation before this call returns. An empty string requests an empty
    /// title, not a backend-specific `None` value.
    fn set_title(&mut self, title: &str) -> Result<()>;
}

/// A compile-time-selected backend for a native container view, label, and button.
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

    struct TestBackend;

    impl UiBackend for TestBackend {
        type View = TestView;
        type Label = TestLabel;
        type Button = TestButton;

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

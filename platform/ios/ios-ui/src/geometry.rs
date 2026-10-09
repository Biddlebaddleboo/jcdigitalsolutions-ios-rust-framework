use framework_core::{Error, ErrorKind, Result};
use framework_ui::Frame;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    #[link_name = "CGRectIntersection"]
    fn cg_rect_intersection(first: CGRect, second: CGRect) -> CGRect;
    #[link_name = "CGRectIsNull"]
    fn cg_rect_is_null(rect: CGRect) -> bool;
    #[link_name = "CGRectIsEmpty"]
    fn cg_rect_is_empty(rect: CGRect) -> bool;
}

/// Returns the positive-area intersection of two frames using CoreGraphics.
///
/// Both frames must use the same coordinate space and logical units. Zero-size frames and
/// edge- or corner-touching frames return `Ok(None)`, matching the portable `Frame::intersection`
/// contract. Unrepresentable input edges return `InvalidInput`. An invalid or nonmatching native
/// result returns `Platform`.
pub fn intersection(first: Frame, second: Frame) -> Result<Option<Frame>> {
    let expected = first.intersection(second)?;
    let first = native_rect(first);
    let second = native_rect(second);

    // SAFETY: Both rectangles are built from finite Frame fields after the portable intersection
    // validates their maximum edges. The SDK declares these by-value C functions for CGRect.
    let native = unsafe { cg_rect_intersection(first, second) };
    // SAFETY: `native` is the CGRect returned by the immediately preceding CoreGraphics call.
    let is_null = unsafe { cg_rect_is_null(native) };
    // SAFETY: `native` is a CGRect returned by CoreGraphics; empty includes zero-size and null.
    let is_empty = unsafe { cg_rect_is_empty(native) };

    if is_null || is_empty {
        return if expected.is_none() {
            Ok(None)
        } else {
            Err(platform_error())
        };
    }

    let Some(expected) = expected else {
        return Err(platform_error());
    };
    let native = frame_from_rect(native)?;
    if native == expected {
        Ok(Some(native))
    } else {
        Err(platform_error())
    }
}

fn native_rect(frame: Frame) -> CGRect {
    CGRect::new(
        CGPoint::new(frame.x(), frame.y()),
        CGSize::new(frame.width(), frame.height()),
    )
}

fn frame_from_rect(rect: CGRect) -> Result<Frame> {
    let x = rect.origin.x;
    let y = rect.origin.y;
    let width = rect.size.width;
    let height = rect.size.height;
    if !x.is_finite()
        || !y.is_finite()
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return Err(platform_error());
    }

    let frame = Frame::new(x, y, width, height).map_err(|_| platform_error())?;
    match frame.intersection(frame).map_err(|_| platform_error())? {
        Some(representable) if representable == frame => Ok(frame),
        _ => Err(platform_error()),
    }
}

fn platform_error() -> Error {
    Error::new(ErrorKind::Platform)
}

//! A main-thread UIKit host for caller-rendered RGBA framebuffer content.

use core::fmt;
use core::ptr;

use objc2::MainThreadMarker;
use objc2_core_foundation::{CFData, CFIndex, CGRect};
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
    CGImageByteOrderInfo,
};
use objc2_ui_kit::{
    UIApplication, UIApplicationState, UIImage, UIImageView, UIView, UIViewContentMode,
};

use crate::custom_render_frame::{self, DamageShape, FrameShape, FrameShapeError, MAX_FRAME_BYTES};

/// Pixel damage supplied by the framebuffer producer, in top-left-origin pixel coordinates.
///
/// The current UIImageView adapter replaces the whole image on each submit; the rectangle is
/// validated and retained as producer metadata, but does not currently reduce the bytes copied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelDamageRect {
    /// Left edge in pixels.
    pub x: u32,
    /// Top edge in pixels.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// A borrowed, length-checked RGBA8 image with premultiplied alpha.
///
/// Rows use an explicit byte stride. The submitted slice is copied synchronously into
/// CoreFoundation-owned bytes before UIKit retains its image; no borrowed Rust memory survives
/// [`IosCustomRenderHost::present`]. Color data uses CoreGraphics device RGB, with one byte each
/// for red, green, blue, and premultiplied alpha in that order.
pub struct Rgba8Frame<'a> {
    width: u32,
    height: u32,
    bytes_per_row: u32,
    pixels: &'a [u8],
    damage: Option<PixelDamageRect>,
    required_bytes: usize,
}

impl<'a> Rgba8Frame<'a> {
    /// Validate one frame without allocating or copying its pixel data.
    pub fn new(
        width: u32,
        height: u32,
        bytes_per_row: u32,
        pixels: &'a [u8],
        damage: Option<PixelDamageRect>,
    ) -> Result<Self, Rgba8FrameError> {
        let shape = FrameShape {
            width,
            height,
            bytes_per_row,
            bytes_available: pixels.len(),
            damage: damage.map(|rect| DamageShape {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }),
        };
        let validated = custom_render_frame::validate_frame(shape)?;
        Ok(Self {
            width,
            height,
            bytes_per_row,
            pixels,
            damage,
            required_bytes: validated.required_bytes,
        })
    }

    /// Frame width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Frame height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Byte distance between adjacent rows.
    pub const fn bytes_per_row(&self) -> u32 {
        self.bytes_per_row
    }

    /// Optional producer damage rectangle, already checked against frame bounds.
    pub const fn damage(&self) -> Option<PixelDamageRect> {
        self.damage
    }
}

/// Deterministic errors for invalid 32-bit framebuffer dimensions, row layout, or bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameValidationError {
    /// Width or height is zero.
    EmptyDimension,
    /// Width or height exceeds the supported maximum of 16,384 pixels.
    DimensionTooLarge,
    /// The stride is smaller than four bytes per pixel.
    RowTooShort,
    /// The stride is not aligned to a four-byte pixel boundary.
    RowMisaligned,
    /// Dimension, stride, or extent arithmetic overflowed.
    SizeOverflow,
    /// A frame exceeds the 64 MiB synchronous copy cap.
    FrameTooLarge,
    /// The slice does not contain every byte required by its height and stride.
    BufferTooShort,
    /// The optional damage rectangle has a zero width or height.
    EmptyDamage,
    /// The optional damage rectangle overflows or extends beyond the frame.
    DamageOutOfBounds,
}

/// Backward-compatible validation error name for [`Rgba8Frame`].
pub type Rgba8FrameError = FrameValidationError;

impl From<FrameShapeError> for FrameValidationError {
    fn from(error: FrameShapeError) -> Self {
        match error {
            FrameShapeError::EmptyDimension => Self::EmptyDimension,
            FrameShapeError::DimensionTooLarge => Self::DimensionTooLarge,
            FrameShapeError::RowTooShort => Self::RowTooShort,
            FrameShapeError::RowMisaligned => Self::RowMisaligned,
            FrameShapeError::SizeOverflow => Self::SizeOverflow,
            FrameShapeError::FrameTooLarge => Self::FrameTooLarge,
            FrameShapeError::BufferTooShort => Self::BufferTooShort,
            FrameShapeError::EmptyDamage => Self::EmptyDamage,
            FrameShapeError::DamageOutOfBounds => Self::DamageOutOfBounds,
        }
    }
}

impl fmt::Display for FrameValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid 32-bit framebuffer: {self:?}")
    }
}

impl std::error::Error for FrameValidationError {}

/// A point-in-time UIKit application lifecycle state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostApplicationState {
    /// UIKit reports the application active in the foreground.
    Active,
    /// UIKit reports a foreground transition or interruption.
    Inactive,
    /// UIKit reports the application in the background.
    Background,
    /// UIKit returned an unknown future state.
    Unknown,
}

/// Current host bounds, scale, safe-area insets, attachment, and app state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostGeometry {
    /// Host frame origin X in parent-view points.
    pub x: f64,
    /// Host frame origin Y in parent-view points.
    pub y: f64,
    /// Host bounds width in points.
    pub width: f64,
    /// Host bounds height in points.
    pub height: f64,
    /// UIKit content scale factor in pixels per point.
    pub scale: f64,
    /// Top safe-area inset in points.
    pub safe_top: f64,
    /// Left safe-area inset in points.
    pub safe_left: f64,
    /// Bottom safe-area inset in points.
    pub safe_bottom: f64,
    /// Right safe-area inset in points.
    pub safe_right: f64,
    /// Whether the image view is attached to a window, not hidden, and has nonzero alpha.
    pub visible: bool,
    /// The current point-in-time UIApplication state.
    pub application_state: HostApplicationState,
}

/// Native UIKit errors that prevent image construction or presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomRenderHostError {
    /// Framebuffer validation failed before native allocation.
    InvalidFrame(Rgba8FrameError),
    /// CoreFoundation or CoreGraphics could not create the image backing.
    GraphicsResourceUnavailable,
}

impl fmt::Display for CustomRenderHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "custom render host: {self:?}")
    }
}

impl std::error::Error for CustomRenderHostError {}

impl From<Rgba8FrameError> for CustomRenderHostError {
    fn from(error: Rgba8FrameError) -> Self {
        Self::InvalidFrame(error)
    }
}

/// One UIKit image view that embeds a caller-owned renderer in a native view hierarchy.
///
/// The handle is main-thread-affine and !Send/!Sync. It does not implement widget layout,
/// state observation, a declarative tree, drawing callbacks, or a scheduler. Callers own those
/// policies and can use [`Self::native_view`] for additional public UIKit integration.
pub struct IosCustomRenderHost {
    view: objc2::rc::Retained<UIImageView>,
    marker: MainThreadMarker,
}

impl IosCustomRenderHost {
    /// Create one unattached transparent UIImageView host on the UIKit main thread.
    pub fn new(marker: MainThreadMarker) -> Self {
        let view = UIImageView::new(marker);
        view.setContentMode(UIViewContentMode::ScaleToFill);
        view.setOpaque(false);
        view.setUserInteractionEnabled(true);
        Self { view, marker }
    }

    /// Attach this host to a caller-owned UIView and set its initial frame in parent points.
    pub fn attach_to(&mut self, parent: &UIView, frame: CGRect) {
        self.view.setFrame(frame);
        parent.addSubview(&self.view);
    }

    /// Replace the host frame in parent-view points; UIKit layout and orientation update bounds.
    pub fn set_frame(&mut self, frame: CGRect) {
        self.view.setFrame(frame);
    }

    /// Set UIKit's hidden flag for this native host.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.view.setHidden(hidden);
    }

    /// Return a point-in-time snapshot of UIKit geometry and app lifecycle state.
    pub fn geometry(&self) -> HostGeometry {
        let frame = self.view.frame();
        let bounds = self.view.bounds();
        let safe = self.view.safeAreaInsets();
        let visible =
            self.view.window().is_some() && !self.view.isHidden() && self.view.alpha() > 0.0;
        let state = UIApplication::sharedApplication(self.marker).applicationState();
        let application_state = if state == UIApplicationState::Active {
            HostApplicationState::Active
        } else if state == UIApplicationState::Inactive {
            HostApplicationState::Inactive
        } else if state == UIApplicationState::Background {
            HostApplicationState::Background
        } else {
            HostApplicationState::Unknown
        };
        HostGeometry {
            x: frame.origin.x,
            y: frame.origin.y,
            width: bounds.size.width,
            height: bounds.size.height,
            scale: self.view.contentScaleFactor(),
            safe_top: safe.top,
            safe_left: safe.left,
            safe_bottom: safe.bottom,
            safe_right: safe.right,
            visible,
            application_state,
        }
    }

    /// Copy and present one complete image on the main thread.
    ///
    /// This operation is synchronous: it copies at most 64 MiB before returning, retains no
    /// borrowed pointer, does not queue stale frames, and replaces the full image even if damage
    /// metadata is present. Callers should defer submissions while the application is backgrounded
    /// and submit the newest frame after foreground resume.
    pub fn present(&self, frame: &Rgba8Frame<'_>) -> Result<(), CustomRenderHostError> {
        if frame.required_bytes > MAX_FRAME_BYTES {
            return Err(Rgba8FrameError::FrameTooLarge.into());
        }
        let length =
            CFIndex::try_from(frame.required_bytes).map_err(|_| Rgba8FrameError::SizeOverflow)?;
        // SAFETY: CFDataCreate copies exactly the validated slice prefix before returning. The
        // slice is live and at least `required_bytes` long; its borrowed pointer is not retained.
        let data = unsafe { CFData::new(None, frame.pixels.as_ptr(), length) }
            .ok_or(CustomRenderHostError::GraphicsResourceUnavailable)?;
        let provider = CGDataProvider::with_cf_data(Some(&data))
            .ok_or(CustomRenderHostError::GraphicsResourceUnavailable)?;
        let color_space = CGColorSpace::new_device_rgb()
            .ok_or(CustomRenderHostError::GraphicsResourceUnavailable)?;
        let bitmap_info = CGBitmapInfo(
            CGImageAlphaInfo::PremultipliedLast.0 | CGImageByteOrderInfo::Order32Big.0,
        );
        // SAFETY: Dimensions, row stride and provider byte length were checked by Rgba8Frame::new;
        // the pixel layout is 8-bit premultiplied-last RGBA with explicit 32-bit big byte order.
        let image = unsafe {
            CGImage::new(
                frame.width as usize,
                frame.height as usize,
                8,
                32,
                frame.bytes_per_row as usize,
                Some(&color_space),
                bitmap_info,
                Some(&provider),
                ptr::null(),
                true,
                CGColorRenderingIntent::RenderingIntentDefault,
            )
        }
        .ok_or(CustomRenderHostError::GraphicsResourceUnavailable)?;
        let image = UIImage::imageWithCGImage(&image);
        self.view.setImage(Some(&image));
        Ok(())
    }

    /// Remove the host from its parent without destroying this handle.
    pub fn detach(&mut self) {
        self.view.removeFromSuperview();
    }

    /// Borrow the underlying native UIImageView for UIKit APIs not wrapped here.
    pub fn native_view(&self) -> &UIImageView {
        &self.view
    }
}

impl Drop for IosCustomRenderHost {
    fn drop(&mut self) {
        self.view.setImage(None);
        self.view.removeFromSuperview();
    }
}

//! Optional Metal framebuffer presentation for custom native renderers.

use core::fmt;
use core::ptr::NonNull;

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_metal::{
    MTLBlitCommandEncoder, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue,
    MTLCreateSystemDefaultDevice, MTLDevice, MTLDrawable, MTLOrigin, MTLPixelFormat, MTLRegion,
    MTLSize, MTLStorageMode, MTLTexture, MTLTextureDescriptor,
};
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use objc2_ui_kit::UIView;

use crate::custom_render_frame::{self, DamageShape, FrameShape};
use crate::custom_render_host::{
    HostApplicationState, IosCustomRenderHost, PixelDamageRect, Rgba8FrameError,
};

/// Validation errors for a BGRA8 framebuffer.
pub type Bgra8FrameError = Rgba8FrameError;

/// Borrowed BGRA8 pixels for the optional Metal presenter.
///
/// Pixels use blue, green, red, premultiplied alpha bytes. The slice must contain at least
/// `bytes_per_row * height` bytes even when `damage` names only part of the image. A damaged
/// update changes only that rectangle in the retained Metal texture; pass `None` for a full-frame
/// upload. After invalidation, the next present must be a full-frame upload.
pub struct Bgra8Frame<'a> {
    width: u32,
    height: u32,
    bytes_per_row: u32,
    pixels: &'a [u8],
    damage: Option<PixelDamageRect>,
}

impl<'a> Bgra8Frame<'a> {
    /// Validate a borrowed BGRA8 frame and optional top-left-origin pixel damage rectangle.
    pub fn new(
        width: u32,
        height: u32,
        bytes_per_row: u32,
        pixels: &'a [u8],
        damage: Option<PixelDamageRect>,
    ) -> Result<Self, Bgra8FrameError> {
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
        custom_render_frame::validate_frame(shape)?;
        Ok(Self {
            width,
            height,
            bytes_per_row,
            pixels,
            damage,
        })
    }
}

/// Errors returned by the optional Metal framebuffer presenter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomMetalHostError {
    /// No system-default Metal device is available.
    DeviceUnavailable,
    /// Metal could not allocate the command queue, texture, or command buffer.
    ResourceUnavailable,
    /// UIKit host is detached, hidden, or not visible in a window.
    HostNotVisible,
    /// UIKit is inactive or backgrounded; submit a full frame after it becomes active.
    HostInactive,
    /// A new or invalidated texture requires an undamaged full frame before partial updates.
    FullFrameRequired,
    /// No drawable could be acquired from the layer without waiting for the display.
    DrawableUnavailable,
    /// The layer drawable has an unexpected size or pixel format.
    DrawableMismatch,
    /// The submitted Metal command buffer completed with an error.
    CommandFailed,
    /// Frame shape, stride, damage, or data length is invalid.
    InvalidFrame(Bgra8FrameError),
}

impl From<Bgra8FrameError> for CustomMetalHostError {
    fn from(error: Bgra8FrameError) -> Self {
        Self::InvalidFrame(error)
    }
}

impl fmt::Display for CustomMetalHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "custom Metal host error: {self:?}")
    }
}

impl std::error::Error for CustomMetalHostError {}

/// Optional, main-thread Metal presenter backed by one caller-controlled native UIKit view.
///
/// Metal is not linked unless the `custom-render-metal` crate feature is enabled. This path uses
/// public `CAMetalLayer` and `MTLTexture` interfaces, keeps the synchronous CPU
/// [`crate::IosCustomRenderHost`] fallback available, and accepts BGRA8 to match the supported
/// drawable pixel format without a CPU channel-swizzle copy. `present` copies the submitted
/// rectangle into a retained shared-storage texture, then asynchronously blits it to one layer
/// drawable. The previous command is completed before its source texture is reused; this can
/// block the caller if the GPU has not finished. Call [`invalidate`][Self::invalidate] on
/// background/suspend and submit a full frame after resume.
pub struct IosCustomMetalHost {
    host: IosCustomRenderHost,
    layer: Retained<CAMetalLayer>,
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    texture: Option<Retained<ProtocolObject<dyn MTLTexture>>>,
    pending_command: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
    pending_drawable: Option<Retained<ProtocolObject<dyn objc2_quartz_core::CAMetalDrawable>>>,
    texture_size: Option<(u32, u32)>,
    initialized: bool,
}

impl IosCustomMetalHost {
    /// Create an unattached native host and Metal layer on the UIKit main thread.
    pub fn new(marker: MainThreadMarker) -> Result<Self, CustomMetalHostError> {
        let device =
            MTLCreateSystemDefaultDevice().ok_or(CustomMetalHostError::DeviceUnavailable)?;
        let queue = device
            .newCommandQueue()
            .ok_or(CustomMetalHostError::ResourceUnavailable)?;
        let layer = CAMetalLayer::layer();
        layer.setDevice(Some(&device));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        // Blit copies are disallowed for framebuffer-only drawable textures.
        layer.setFramebufferOnly(false);
        layer.setAllowsNextDrawableTimeout(true);

        let host = IosCustomRenderHost::new(marker);
        let backing_layer = host.native_view().layer();
        backing_layer.addSublayer(&layer);

        Ok(Self {
            host,
            layer,
            device,
            queue,
            texture: None,
            pending_command: None,
            pending_drawable: None,
            texture_size: None,
            initialized: false,
        })
    }

    /// Attach this host to a caller-owned view using points in the parent coordinate space.
    pub fn attach_to(&mut self, parent: &UIView, frame: CGRect) {
        self.host.attach_to(parent, frame);
        self.initialized = false;
    }

    /// Replace the host frame in parent-view points.
    pub fn set_frame(&mut self, frame: CGRect) {
        self.host.set_frame(frame);
    }

    /// Set the underlying UIKit host's hidden flag and require a full frame when shown again.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.host.set_hidden(hidden);
        if hidden {
            self.initialized = false;
        }
    }

    /// Remove the host from its parent without destroying this handle.
    pub fn detach(&mut self) {
        self.host.detach();
        self.initialized = false;
    }

    /// Mark the retained texture stale after a background, suspend, or content reset.
    pub fn invalidate(&mut self) {
        self.initialized = false;
    }

    /// Borrow the underlying native `UIImageView` for UIKit APIs not wrapped here.
    pub fn native_view(&self) -> &objc2_ui_kit::UIImageView {
        self.host.native_view()
    }

    /// Copy a full or damaged BGRA8 rectangle and asynchronously submit it to Metal.
    pub fn present(&mut self, frame: &Bgra8Frame<'_>) -> Result<(), CustomMetalHostError> {
        let geometry = self.host.geometry();
        if !geometry.visible {
            self.initialized = false;
            return Err(CustomMetalHostError::HostNotVisible);
        }
        if geometry.application_state != HostApplicationState::Active {
            self.initialized = false;
            return Err(CustomMetalHostError::HostInactive);
        }
        self.finish_previous_command()?;

        self.layer.setFrame(CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(geometry.width, geometry.height),
        ));
        self.layer.setContentsScale(geometry.scale);
        self.layer
            .setDrawableSize(CGSize::new(frame.width as f64, frame.height as f64));

        let full = PixelDamageRect {
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height,
        };
        let region = frame.damage.unwrap_or(full);
        let is_full = region == full;
        let size = (frame.width, frame.height);
        if self.texture_size != Some(size) {
            if !is_full {
                return Err(CustomMetalHostError::FullFrameRequired);
            }
            self.texture = Some(self.new_shared_texture(frame.width, frame.height)?);
            self.texture_size = Some(size);
            self.initialized = false;
        }
        if !self.initialized && !is_full {
            return Err(CustomMetalHostError::FullFrameRequired);
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(CustomMetalHostError::ResourceUnavailable)?;
        self.copy_frame_region(texture, frame, region)?;

        let drawable = self
            .layer
            .nextDrawable()
            .ok_or(CustomMetalHostError::DrawableUnavailable)?;
        let destination = drawable.texture();
        if destination.pixelFormat() != MTLPixelFormat::BGRA8Unorm
            || destination.width() != frame.width as usize
            || destination.height() != frame.height as usize
        {
            self.initialized = false;
            return Err(CustomMetalHostError::DrawableMismatch);
        }
        let command = self
            .queue
            .commandBuffer()
            .ok_or(CustomMetalHostError::ResourceUnavailable)?;
        // Metal command buffers created by the normal queue path retain encoded resources by
        // default; `pending_drawable` also explicitly retains the drawable through completion.
        let blit = command
            .blitCommandEncoder()
            .ok_or(CustomMetalHostError::ResourceUnavailable)?;
        // SAFETY: Both texture dimensions and pixel formats are checked above. `texture` is
        // retained in `self`; `drawable` stays retained in `pending_drawable`; and the command
        // buffer retains its encoded resources through completion.
        unsafe { blit.copyFromTexture_toTexture(texture, &destination) };
        blit.endEncoding();
        let drawable_object: &ProtocolObject<dyn CAMetalDrawable> = drawable.as_ref();
        let drawable_protocol: &ProtocolObject<dyn MTLDrawable> = <ProtocolObject<
            dyn CAMetalDrawable,
        > as AsRef<
            ProtocolObject<dyn MTLDrawable>,
        >>::as_ref(
            drawable_object
        );
        command.presentDrawable(drawable_protocol);
        command.commit();
        self.pending_command = Some(command);
        self.pending_drawable = Some(drawable);
        self.initialized = true;
        Ok(())
    }

    /// Wait for the current GPU submission and report asynchronous Metal execution failure.
    pub fn synchronize(&mut self) -> Result<(), CustomMetalHostError> {
        self.finish_previous_command()
    }

    fn new_shared_texture(
        &self,
        width: u32,
        height: u32,
    ) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, CustomMetalHostError> {
        // SAFETY: The shared frame validator limits each nonzero dimension to 16,384 pixels.
        let descriptor = unsafe {
            MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
                MTLPixelFormat::BGRA8Unorm,
                width as usize,
                height as usize,
                false,
            )
        };
        descriptor.setStorageMode(MTLStorageMode::Shared);
        self.device
            .newTextureWithDescriptor(&descriptor)
            .ok_or(CustomMetalHostError::ResourceUnavailable)
    }

    fn copy_frame_region(
        &self,
        texture: &ProtocolObject<dyn MTLTexture>,
        frame: &Bgra8Frame<'_>,
        damage: PixelDamageRect,
    ) -> Result<(), CustomMetalHostError> {
        let offset = (damage.y as usize)
            .checked_mul(frame.bytes_per_row as usize)
            .and_then(|row| row.checked_add((damage.x as usize).checked_mul(4)?))
            .ok_or(CustomMetalHostError::InvalidFrame(
                Rgba8FrameError::SizeOverflow,
            ))?;
        // SAFETY: `validate_frame` checked the full slice extent and damage bounds. The calculated
        // offset is inside the validated slice, and replaceRegion copies its bytes synchronously.
        let pointer = unsafe { frame.pixels.as_ptr().add(offset) as *mut core::ffi::c_void };
        let pointer = NonNull::new(pointer).ok_or(CustomMetalHostError::InvalidFrame(
            Rgba8FrameError::BufferTooShort,
        ))?;
        let region = MTLRegion {
            origin: MTLOrigin {
                x: damage.x as usize,
                y: damage.y as usize,
                z: 0,
            },
            size: MTLSize {
                width: damage.width as usize,
                height: damage.height as usize,
                depth: 1,
            },
        };
        // SAFETY: The region lies within the texture, its backing slice is live for this call,
        // stride is validated as at least four bytes per pixel, and Metal copies synchronously.
        unsafe {
            texture.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
                region,
                0,
                pointer,
                frame.bytes_per_row as usize,
            )
        };
        Ok(())
    }

    fn finish_previous_command(&mut self) -> Result<(), CustomMetalHostError> {
        if let Some(command) = self.pending_command.take() {
            command.waitUntilCompleted();
            self.pending_drawable.take();
            if command.error().is_some() {
                self.initialized = false;
                return Err(CustomMetalHostError::CommandFailed);
            }
        }
        Ok(())
    }
}

impl Drop for IosCustomMetalHost {
    fn drop(&mut self) {
        if let Some(command) = self.pending_command.take() {
            command.waitUntilCompleted();
        }
        self.pending_drawable.take();
        self.layer.removeFromSuperlayer();
    }
}

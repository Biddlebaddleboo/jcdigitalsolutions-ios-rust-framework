#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable image dimensions and source metadata values with a static backend contract."]

use framework_core::{Error, ErrorKind, PlatformErrorCode};

/// Nonzero encoded pixel dimensions for image zero in a source.
///
/// These values describe encoded pixel width and height. They do not apply orientation,
/// display-scale, color-space, or crop transforms.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImageDimensions {
    width_pixels: u32,
    height_pixels: u32,
}

impl ImageDimensions {
    /// Creates dimensions with nonzero encoded pixel width and height.
    pub const fn new(width_pixels: u32, height_pixels: u32) -> Result<Self, ImageMetadataError> {
        if width_pixels == 0 || height_pixels == 0 {
            return Err(ImageMetadataError::InvalidDimensions);
        }
        Ok(Self {
            width_pixels,
            height_pixels,
        })
    }

    /// Returns the encoded pixel width.
    pub const fn width_pixels(self) -> u32 {
        self.width_pixels
    }

    /// Returns the encoded pixel height.
    pub const fn height_pixels(self) -> u32 {
        self.height_pixels
    }

    /// Returns the encoded pixel count as a `u64` product.
    pub const fn pixel_count(self) -> u64 {
        self.width_pixels as u64 * self.height_pixels as u64
    }
}

/// Metadata for one encoded image source.
///
/// `dimensions` describes source image index zero. `image_count` is the total source count
/// reported by the selected backend; it may include animation frames or other source images.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImageMetadata {
    dimensions: ImageDimensions,
    image_count: u32,
}

impl ImageMetadata {
    /// Creates metadata with a nonzero source image count.
    pub const fn new(
        dimensions: ImageDimensions,
        image_count: u32,
    ) -> Result<Self, ImageMetadataError> {
        if image_count == 0 {
            return Err(ImageMetadataError::InvalidImageCount);
        }
        Ok(Self {
            dimensions,
            image_count,
        })
    }

    /// Returns encoded pixel dimensions for source image index zero.
    pub const fn dimensions(self) -> ImageDimensions {
        self.dimensions
    }

    /// Returns the source image/animation-frame count reported by the backend.
    pub const fn image_count(self) -> u32 {
        self.image_count
    }
}

/// A stable portable error for image metadata operations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ImageMetadataError {
    /// The encoded input is empty, malformed, unsupported, or has no usable image-zero metadata.
    InvalidImageData,
    /// A dimension is zero or cannot fit the portable `u32` dimension value.
    InvalidDimensions,
    /// The source image count is zero or cannot fit the portable `u32` count value.
    InvalidImageCount,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl ImageMetadataError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidImageData | Self::InvalidDimensions | Self::InvalidImageCount => {
                ErrorKind::InvalidInput
            }
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns an optional native error code from the selected backend.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidImageData | Self::InvalidDimensions | Self::InvalidImageCount => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A static, synchronous backend for image source metadata.
///
/// The input bytes are borrowed for this call only. A backend must not retain the slice or its
/// pointer after return. The portable contract does not allocate, decode raster pixels, start an
/// executor, or impose a maximum input size. A platform backend may copy the encoded bytes and
/// may allocate internal parsing state.
pub trait ImageMetadataBackend {
    /// Reads dimensions for source image zero and the source image count.
    fn read_metadata(&mut self, encoded_bytes: &[u8]) -> Result<ImageMetadata, ImageMetadataError>;
}

/// A generic facade with statically selected image metadata backend `B`.
pub struct ImageMetadataReader<B> {
    backend: B,
}

impl<B: ImageMetadataBackend> ImageMetadataReader<B> {
    /// Creates a reader that owns the supplied backend value.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reads metadata from caller-owned encoded bytes.
    ///
    /// This call is synchronous. The selected backend controls thread and blocking behavior;
    /// dropping or returning from the call has no async cancellation semantics.
    pub fn read_metadata(
        &mut self,
        encoded_bytes: &[u8],
    ) -> Result<ImageMetadata, ImageMetadataError> {
        self.backend.read_metadata(encoded_bytes)
    }

    /// Returns a shared reference to the selected backend.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Returns a mutable reference to the selected backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_core::{ErrorKind, PlatformErrorCode};

    #[test]
    fn dimensions_reject_zero_and_count_pixels_in_fixed_width() {
        assert_eq!(
            ImageDimensions::new(0, 1),
            Err(ImageMetadataError::InvalidDimensions)
        );
        assert_eq!(
            ImageDimensions::new(1, 0),
            Err(ImageMetadataError::InvalidDimensions)
        );
        let max = ImageDimensions::new(u32::MAX, u32::MAX).unwrap();
        assert_eq!(max.pixel_count(), u64::from(u32::MAX) * u64::from(u32::MAX));
    }

    #[test]
    fn source_count_must_be_nonzero() {
        let dimensions = ImageDimensions::new(640, 480).unwrap();
        assert_eq!(
            ImageMetadata::new(dimensions, 0),
            Err(ImageMetadataError::InvalidImageCount)
        );
        assert_eq!(ImageMetadata::new(dimensions, 1).unwrap().image_count(), 1);
    }

    struct FakeBackend {
        expected_bytes: &'static [u8],
    }

    impl ImageMetadataBackend for FakeBackend {
        fn read_metadata(
            &mut self,
            encoded_bytes: &[u8],
        ) -> Result<ImageMetadata, ImageMetadataError> {
            assert_eq!(encoded_bytes, self.expected_bytes);
            ImageMetadata::new(ImageDimensions::new(640, 480)?, 3)
        }
    }

    #[test]
    fn generic_reader_delegates_borrowed_input_to_static_backend() {
        let bytes = b"caller-owned";
        let backend = FakeBackend {
            expected_bytes: bytes,
        };
        let mut reader = ImageMetadataReader::new(backend);
        let metadata = reader.read_metadata(bytes).unwrap();
        assert_eq!(metadata.dimensions().width_pixels(), 640);
        assert_eq!(metadata.dimensions().height_pixels(), 480);
        assert_eq!(metadata.image_count(), 3);
    }

    #[test]
    fn backend_error_keeps_category_and_native_code() {
        let code = PlatformErrorCode::new(-45).unwrap();
        let error = ImageMetadataError::Backend(
            Error::new(ErrorKind::ResourceExhausted).with_platform_code(code),
        );
        assert_eq!(error.kind(), ErrorKind::ResourceExhausted);
        assert_eq!(error.platform_code().unwrap().get(), -45);
        assert_eq!(
            ImageMetadataError::InvalidImageData.kind(),
            ErrorKind::InvalidInput
        );
    }
}

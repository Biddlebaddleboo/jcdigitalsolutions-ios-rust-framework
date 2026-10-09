use framework_core::{Error, ErrorKind};
use framework_image::{ImageDimensions, ImageMetadata, ImageMetadataBackend, ImageMetadataError};
use objc2_core_foundation::{CFData, CFDictionary, CFIndex, CFNumber, CFString, CFType};
use objc2_image_io::{CGImageSource, kCGImagePropertyPixelHeight, kCGImagePropertyPixelWidth};

/// Zero-sized ImageIO backend for synchronous image metadata reads.
///
/// This backend runs on the caller's thread and has no global state. It copies the full encoded
/// input slice into `CFData`; it does not retain that Rust slice after the method returns.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosImageMetadataBackend;

impl IosImageMetadataBackend {
    /// Creates an ImageIO metadata backend without runtime setup.
    pub const fn new() -> Self {
        Self
    }
}

impl ImageMetadataBackend for IosImageMetadataBackend {
    fn read_metadata(&mut self, encoded_bytes: &[u8]) -> Result<ImageMetadata, ImageMetadataError> {
        let length = CFIndex::try_from(encoded_bytes.len()).map_err(|_| resource_exhausted())?;

        // SAFETY: The slice pointer is valid for `length` bytes for this synchronous call, and
        // `CFDataCreate` copies its input bytes. Passing no allocator selects CoreFoundation's
        // default allocator. No Rust pointer escapes this function.
        let data = unsafe { CFData::new(None, encoded_bytes.as_ptr(), length) }
            .ok_or_else(resource_exhausted)?;

        // SAFETY: The options argument is null, so there are no dictionary type parameters to
        // satisfy. The retained CFData remains live for the full source lifetime below.
        let source = unsafe { CGImageSource::with_data(&data, None) }
            .ok_or(ImageMetadataError::InvalidImageData)?;

        // SAFETY: `source` is a live retained CGImageSource; this generated binding wraps the
        // public CGImageSourceGetCount call. The returned native-sized count is range-checked.
        let count = unsafe { source.count() };
        let image_count =
            u32::try_from(count).map_err(|_| ImageMetadataError::InvalidImageCount)?;
        if image_count == 0 {
            return Err(ImageMetadataError::InvalidImageCount);
        }

        // SAFETY: Index zero is in range because the count was checked above. Null options have
        // no generic type parameters. This requests only the public properties dictionary.
        let properties = unsafe { source.properties_at_index(0, None) }
            .ok_or(ImageMetadataError::InvalidImageData)?;

        // SAFETY: ImageIO returns a properties dictionary with CFString keys and CFType values;
        // values are heterogeneous, so each selected width/height value is downcast separately.
        let properties: &CFDictionary<CFString, CFType> = unsafe { properties.cast_unchecked() };

        // SAFETY: These public ImageIO property keys are exported CFString constants.
        let width_key = unsafe { kCGImagePropertyPixelWidth };
        // SAFETY: These public ImageIO property keys are exported CFString constants.
        let height_key = unsafe { kCGImagePropertyPixelHeight };
        let width = read_dimension(properties, width_key)?;
        let height = read_dimension(properties, height_key)?;
        let dimensions = ImageDimensions::new(width, height)?;
        ImageMetadata::new(dimensions, image_count)
    }
}

fn read_dimension(
    properties: &CFDictionary<CFString, CFType>,
    key: &CFString,
) -> Result<u32, ImageMetadataError> {
    let value = properties
        .get(key)
        .ok_or(ImageMetadataError::InvalidImageData)?;
    let number = value
        .downcast::<CFNumber>()
        .map_err(|_| ImageMetadataError::InvalidImageData)?;
    let value = number
        .as_f64()
        .ok_or(ImageMetadataError::InvalidImageData)?;
    if !value.is_finite() || value <= 0.0 || value.fract() != 0.0 || value > u32::MAX as f64 {
        return Err(ImageMetadataError::InvalidDimensions);
    }
    Ok(value as u32)
}

fn resource_exhausted() -> ImageMetadataError {
    ImageMetadataError::Backend(Error::new(ErrorKind::ResourceExhausted))
}

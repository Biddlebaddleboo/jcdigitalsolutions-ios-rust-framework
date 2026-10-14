//! Host-testable validation for borrowed RGBA8 and packed Mono1 framebuffer submissions.

/// Maximum bytes copied from one framebuffer submission.
pub(crate) const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;
/// Maximum supported width or height in pixels.
pub(crate) const MAX_FRAME_DIMENSION: u32 = 16_384;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameShape {
    pub width: u32,
    pub height: u32,
    pub bytes_per_row: u32,
    pub bytes_available: usize,
    pub damage: Option<DamageShape>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DamageShape {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameShapeError {
    EmptyDimension,
    DimensionTooLarge,
    RowTooShort,
    RowMisaligned,
    SizeOverflow,
    FrameTooLarge,
    BufferTooShort,
    EmptyDamage,
    DamageOutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedFrameShape {
    pub required_bytes: usize,
}

#[derive(Clone, Copy)]
enum FramePixelFormat {
    Rgba8,
    Mono1,
}

pub(crate) fn validate_frame(shape: FrameShape) -> Result<ValidatedFrameShape, FrameShapeError> {
    validate_frame_format(shape, FramePixelFormat::Rgba8)
}

pub(crate) fn validate_mono1_frame(
    shape: FrameShape,
) -> Result<ValidatedFrameShape, FrameShapeError> {
    validate_frame_format(shape, FramePixelFormat::Mono1)
}

fn validate_frame_format(
    shape: FrameShape,
    format: FramePixelFormat,
) -> Result<ValidatedFrameShape, FrameShapeError> {
    if shape.width == 0 || shape.height == 0 {
        return Err(FrameShapeError::EmptyDimension);
    }
    if shape.width > MAX_FRAME_DIMENSION || shape.height > MAX_FRAME_DIMENSION {
        return Err(FrameShapeError::DimensionTooLarge);
    }
    let row_bytes = match format {
        FramePixelFormat::Rgba8 => shape
            .width
            .checked_mul(4)
            .ok_or(FrameShapeError::SizeOverflow)?,
        FramePixelFormat::Mono1 => shape.width / 8 + u32::from(shape.width % 8 != 0),
    };
    if shape.bytes_per_row < row_bytes {
        return Err(FrameShapeError::RowTooShort);
    }
    if matches!(format, FramePixelFormat::Rgba8) && shape.bytes_per_row % 4 != 0 {
        return Err(FrameShapeError::RowMisaligned);
    }
    if let Some(damage) = shape.damage {
        if damage.width == 0 || damage.height == 0 {
            return Err(FrameShapeError::EmptyDamage);
        }
        let right = damage
            .x
            .checked_add(damage.width)
            .ok_or(FrameShapeError::DamageOutOfBounds)?;
        let bottom = damage
            .y
            .checked_add(damage.height)
            .ok_or(FrameShapeError::DamageOutOfBounds)?;
        if right > shape.width || bottom > shape.height {
            return Err(FrameShapeError::DamageOutOfBounds);
        }
    }
    let required_bytes = usize::try_from(shape.bytes_per_row)
        .ok()
        .and_then(|stride| stride.checked_mul(shape.height as usize))
        .ok_or(FrameShapeError::SizeOverflow)?;
    if required_bytes > MAX_FRAME_BYTES {
        return Err(FrameShapeError::FrameTooLarge);
    }
    if shape.bytes_available < required_bytes {
        return Err(FrameShapeError::BufferTooShort);
    }
    Ok(ValidatedFrameShape { required_bytes })
}

#[cfg(test)]
mod tests {
    use super::{
        DamageShape, FrameShape, FrameShapeError, MAX_FRAME_BYTES, MAX_FRAME_DIMENSION,
        validate_frame, validate_mono1_frame,
    };

    fn shape(width: u32, height: u32, bytes_per_row: u32, bytes_available: usize) -> FrameShape {
        FrameShape {
            width,
            height,
            bytes_per_row,
            bytes_available,
            damage: None,
        }
    }

    fn mono1_white_at(pixels: &[u8], bytes_per_row: usize, x: usize, y: usize) -> bool {
        let byte = pixels[y * bytes_per_row + x / 8];
        byte & (0x80 >> (x % 8)) != 0
    }

    #[test]
    fn mono1_frame_accepts_tight_padded_rows_and_msb_first_samples() {
        let tight = validate_mono1_frame(shape(8, 1, 1, 1)).unwrap();
        assert_eq!(tight.required_bytes, 1);

        // The low seven bits of each width-9 row's final pixel byte are outside the image.
        let pixels = [0x81, 0xff, 0xff, 0x40, 0xfe, 0xff];
        let mut padded = shape(9, 2, 3, pixels.len());
        padded.damage = Some(DamageShape {
            x: 1,
            y: 0,
            width: 8,
            height: 2,
        });
        assert_eq!(validate_mono1_frame(padded).unwrap().required_bytes, 6);
        assert!(mono1_white_at(&pixels, 3, 0, 0));
        assert!(mono1_white_at(&pixels, 3, 7, 0));
        assert!(mono1_white_at(&pixels, 3, 8, 0));
        assert!(!mono1_white_at(&pixels, 3, 0, 1));
        assert!(mono1_white_at(&pixels, 3, 1, 1));
        assert!(mono1_white_at(&pixels, 3, 8, 1));
    }

    #[test]
    fn mono1_frame_rejects_invalid_dimensions_stride_and_damage() {
        assert_eq!(
            validate_mono1_frame(shape(0, 1, 0, 0)),
            Err(FrameShapeError::EmptyDimension)
        );
        assert_eq!(
            validate_mono1_frame(shape(1, MAX_FRAME_DIMENSION + 1, 1, usize::MAX)),
            Err(FrameShapeError::DimensionTooLarge)
        );
        assert_eq!(
            validate_mono1_frame(shape(9, 1, 1, 1)),
            Err(FrameShapeError::RowTooShort)
        );
        assert_eq!(
            validate_mono1_frame(shape(9, 2, 2, 3)),
            Err(FrameShapeError::BufferTooShort)
        );
        assert_eq!(
            validate_mono1_frame(shape(
                1,
                2,
                (MAX_FRAME_BYTES / 2 + 1) as u32,
                MAX_FRAME_BYTES + 2,
            )),
            Err(FrameShapeError::FrameTooLarge)
        );

        let mut empty = shape(9, 2, 2, 4);
        empty.damage = Some(DamageShape {
            x: 0,
            y: 0,
            width: 0,
            height: 1,
        });
        assert_eq!(
            validate_mono1_frame(empty),
            Err(FrameShapeError::EmptyDamage)
        );
        let mut outside = shape(9, 2, 2, 4);
        outside.damage = Some(DamageShape {
            x: 8,
            y: 1,
            width: 2,
            height: 1,
        });
        assert_eq!(
            validate_mono1_frame(outside),
            Err(FrameShapeError::DamageOutOfBounds)
        );
        let mut overflow = shape(9, 2, 2, 4);
        overflow.damage = Some(DamageShape {
            x: u32::MAX,
            y: 0,
            width: 1,
            height: 1,
        });
        assert_eq!(
            validate_mono1_frame(overflow),
            Err(FrameShapeError::DamageOutOfBounds)
        );
    }

    #[test]
    fn accepts_tight_and_padded_rows_with_bounded_damage() {
        let tight = validate_frame(shape(2, 3, 8, 24)).unwrap();
        assert_eq!(tight.required_bytes, 24);
        let mut padded = shape(2, 3, 12, 36);
        padded.damage = Some(DamageShape {
            x: 1,
            y: 1,
            width: 1,
            height: 2,
        });
        assert_eq!(validate_frame(padded).unwrap().required_bytes, 36);
    }

    #[test]
    fn rejects_empty_dimensions_and_misaligned_or_short_rows() {
        assert_eq!(
            validate_frame(shape(0, 1, 0, 0)),
            Err(FrameShapeError::EmptyDimension)
        );
        assert_eq!(
            validate_frame(shape(2, 1, 7, 7)),
            Err(FrameShapeError::RowTooShort)
        );
        assert_eq!(
            validate_frame(shape(1, 1, 5, 5)),
            Err(FrameShapeError::RowMisaligned)
        );
        assert_eq!(
            validate_frame(shape(2, 2, 8, 15)),
            Err(FrameShapeError::BufferTooShort)
        );
    }

    #[test]
    fn rejects_invalid_dimensions_sizes_and_damage_without_clipping() {
        assert_eq!(
            validate_frame(shape(MAX_FRAME_DIMENSION + 1, 1, 4, 4)),
            Err(FrameShapeError::DimensionTooLarge)
        );
        assert_eq!(
            validate_frame(shape((u32::MAX / 4) + 1, 1, u32::MAX, usize::MAX)),
            Err(FrameShapeError::DimensionTooLarge)
        );
        assert_eq!(
            validate_frame(shape(4096, 4096, 4096 * 4 + 4, MAX_FRAME_BYTES + 16_384)),
            Err(FrameShapeError::FrameTooLarge)
        );
        let mut empty = shape(2, 2, 8, 16);
        empty.damage = Some(DamageShape {
            x: 0,
            y: 0,
            width: 0,
            height: 1,
        });
        assert_eq!(validate_frame(empty), Err(FrameShapeError::EmptyDamage));
        let mut outside = shape(2, 2, 8, 16);
        outside.damage = Some(DamageShape {
            x: 1,
            y: 1,
            width: 2,
            height: 1,
        });
        assert_eq!(
            validate_frame(outside),
            Err(FrameShapeError::DamageOutOfBounds)
        );
        let mut overflow = shape(2, 2, 8, 16);
        overflow.damage = Some(DamageShape {
            x: u32::MAX,
            y: 0,
            width: 1,
            height: 1,
        });
        assert_eq!(
            validate_frame(overflow),
            Err(FrameShapeError::DamageOutOfBounds)
        );
    }
}

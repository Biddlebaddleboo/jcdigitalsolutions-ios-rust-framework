//! Host-testable validation for borrowed RGBA framebuffer submissions.

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

pub(crate) fn validate_frame(shape: FrameShape) -> Result<ValidatedFrameShape, FrameShapeError> {
    if shape.width == 0 || shape.height == 0 {
        return Err(FrameShapeError::EmptyDimension);
    }
    if shape.width > MAX_FRAME_DIMENSION || shape.height > MAX_FRAME_DIMENSION {
        return Err(FrameShapeError::DimensionTooLarge);
    }
    let row_bytes = shape
        .width
        .checked_mul(4)
        .ok_or(FrameShapeError::SizeOverflow)?;
    if shape.bytes_per_row < row_bytes {
        return Err(FrameShapeError::RowTooShort);
    }
    if shape.bytes_per_row % 4 != 0 {
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
        validate_frame,
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

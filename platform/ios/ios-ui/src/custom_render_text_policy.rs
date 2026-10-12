use core::ops::Range;

/// A selection or marked-text range measured in UTF-16 code units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Utf16Range {
    /// UTF-16 code-unit offset from the start of the native text.
    pub location: u32,
    /// Number of UTF-16 code units in the range.
    pub length: u32,
}

impl Utf16Range {
    /// Create a UTF-16 range. Use [`Self::end`] to detect addition overflow.
    pub const fn new(location: u32, length: u32) -> Self {
        Self { location, length }
    }

    /// Return the exclusive UTF-16 end offset, if representable.
    pub const fn end(self) -> Option<u32> {
        self.location.checked_add(self.length)
    }

    /// Convert UTF-8 byte boundaries in `text` to exact UTF-16 offsets.
    pub fn from_utf8_byte_range(text: &str, range: Range<usize>) -> Result<Self, Utf16RangeError> {
        byte_range_to_utf16(text, range)
    }

    /// Convert this UTF-16 range to UTF-8 byte boundaries in `text`.
    ///
    /// Returns an error if the range is out of bounds or splits a UTF-16 surrogate pair.
    pub fn to_utf8_byte_range(self, text: &str) -> Result<Range<usize>, Utf16RangeError> {
        utf16_range_to_bytes(text, self)
    }
}

/// A failure to translate a UTF-8 byte range to UIKit's UTF-16 offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utf16RangeError {
    /// The requested range extends beyond the text or has reversed byte boundaries.
    OutOfBounds,
    /// A UTF-8 or UTF-16 boundary falls inside one Unicode scalar value.
    SplitScalar,
    /// The range or converted offset cannot fit its public `u32` representation.
    Overflow,
}

pub(crate) fn byte_range_to_utf16(
    text: &str,
    range: Range<usize>,
) -> Result<Utf16Range, Utf16RangeError> {
    if range.start > range.end || range.end > text.len() {
        return Err(Utf16RangeError::OutOfBounds);
    }
    let location = byte_offset_to_utf16(text, range.start)?;
    let end = byte_offset_to_utf16(text, range.end)?;
    let length = end.checked_sub(location).ok_or(Utf16RangeError::Overflow)?;
    Ok(Utf16Range::new(location, length))
}

pub(crate) fn utf16_range_to_bytes(
    text: &str,
    range: Utf16Range,
) -> Result<Range<usize>, Utf16RangeError> {
    let end = range.end().ok_or(Utf16RangeError::Overflow)?;
    let start = utf16_offset_to_byte(text, range.location)?;
    let end = utf16_offset_to_byte(text, end)?;
    Ok(start..end)
}

fn byte_offset_to_utf16(text: &str, offset: usize) -> Result<u32, Utf16RangeError> {
    if !text.is_char_boundary(offset) {
        return Err(Utf16RangeError::SplitScalar);
    }
    text[..offset]
        .encode_utf16()
        .count()
        .try_into()
        .map_err(|_| Utf16RangeError::Overflow)
}

fn utf16_offset_to_byte(text: &str, offset: u32) -> Result<usize, Utf16RangeError> {
    let mut utf16_offset = 0_u32;
    for (byte_offset, character) in text.char_indices() {
        if utf16_offset == offset {
            return Ok(byte_offset);
        }
        utf16_offset = utf16_offset
            .checked_add(character.len_utf16() as u32)
            .ok_or(Utf16RangeError::Overflow)?;
        if utf16_offset > offset {
            return Err(Utf16RangeError::SplitScalar);
        }
        if utf16_offset == offset {
            return Ok(byte_offset + character.len_utf8());
        }
    }
    if utf16_offset == offset {
        Ok(text.len())
    } else {
        Err(Utf16RangeError::OutOfBounds)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CallbackGate {
    active: bool,
}

impl CallbackGate {
    pub(crate) const fn active() -> Self {
        Self { active: true }
    }

    pub(crate) const fn is_active(self) -> bool {
        self.active
    }

    pub(crate) fn deactivate(&mut self) {
        self.active = false;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommittedText {
    text: String,
    selection: Utf16Range,
}

impl CommittedText {
    pub(crate) fn new(text: &str, selection: Utf16Range) -> Self {
        Self {
            text: text.to_owned(),
            selection,
        }
    }

    pub(crate) fn observe(
        &mut self,
        text: &str,
        selection: Utf16Range,
        marked_text: Option<Utf16Range>,
    ) {
        if marked_text.is_none() {
            self.text.clear();
            self.text.push_str(text);
            self.selection = selection;
        }
    }

    pub(crate) fn value(&self) -> (&str, Utf16Range) {
        (&self.text, self.selection)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CallbackGate, CommittedText, Utf16Range, Utf16RangeError, byte_range_to_utf16,
        utf16_range_to_bytes,
    };

    #[test]
    fn maps_utf8_bytes_to_exact_utf16_offsets_with_supplementary_scalars() {
        let text = "a🚗e\u{301}\0z";
        let bytes = "a".len().."a🚗e\u{301}".len();
        assert_eq!(byte_range_to_utf16(text, bytes), Ok(Utf16Range::new(1, 4)));
        assert_eq!(
            utf16_range_to_bytes(text, Utf16Range::new(1, 4)),
            Ok("a".len().."a🚗e\u{301}".len())
        );
    }

    #[test]
    fn rejects_surrogate_splits_invalid_bounds_and_range_overflow() {
        let text = "x🚙y";
        assert_eq!(
            utf16_range_to_bytes(text, Utf16Range::new(2, 0)),
            Err(Utf16RangeError::SplitScalar)
        );
        assert_eq!(
            utf16_range_to_bytes(text, Utf16Range::new(1, 2)),
            Ok(1.."x🚙".len())
        );
        assert_eq!(
            utf16_range_to_bytes(text, Utf16Range::new(5, 0)),
            Err(Utf16RangeError::OutOfBounds)
        );
        assert_eq!(
            utf16_range_to_bytes(text, Utf16Range::new(u32::MAX, 1)),
            Err(Utf16RangeError::Overflow)
        );
    }

    #[test]
    fn active_composition_does_not_replace_the_last_committed_rust_text() {
        let mut committed = CommittedText::new("a🚗", Utf16Range::new(3, 0));
        let composing_text = "a🚗に";
        let marked =
            Utf16Range::from_utf8_byte_range(composing_text, "a🚗".len()..composing_text.len())
                .expect("the marked scalar has valid UTF-8 boundaries");
        assert_eq!(marked, Utf16Range::new(3, 1));
        assert_eq!(
            marked.to_utf8_byte_range(composing_text),
            Ok("a🚗".len()..composing_text.len())
        );
        committed.observe(composing_text, Utf16Range::new(4, 0), Some(marked));
        assert_eq!(committed.value(), ("a🚗", Utf16Range::new(3, 0)));
        committed.observe("a🚗日", Utf16Range::new(4, 0), None);
        assert_eq!(committed.value(), ("a🚗日", Utf16Range::new(4, 0)));
    }

    #[test]
    fn teardown_gate_rejects_stale_callback_delivery() {
        let mut gate = CallbackGate::active();
        assert!(gate.is_active());
        gate.deactivate();
        assert!(!gate.is_active());
        gate.deactivate();
        assert!(!gate.is_active());
    }
}

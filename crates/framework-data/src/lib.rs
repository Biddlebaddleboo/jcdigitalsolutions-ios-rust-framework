#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable borrowed byte and UTF-8 views with explicit ownership conversions."]
//!
//! ```
//! use framework_data::{ByteView, DataError};
//!
//! fn borrowed_text(bytes: &[u8]) -> Result<&str, DataError> {
//!     Ok(ByteView::new(bytes).try_as_utf8()?.as_str())
//! }
//! ```

extern crate alloc;

use alloc::{string::String, vec::Vec};

/// A borrowed view of exact bytes.
///
/// Construction and access do not allocate, copy, interpret, or normalize the bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByteView<'a>(&'a [u8]);

impl<'a> ByteView<'a> {
    /// Creates a view over the exact caller-borrowed byte slice.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Returns the exact borrowed byte slice.
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }

    /// Validates the bytes as UTF-8 and returns a view of the same borrowed storage.
    pub fn try_as_utf8(self) -> Result<Utf8View<'a>, DataError> {
        core::str::from_utf8(self.0)
            .map(Utf8View)
            .map_err(|_| DataError::InvalidUtf8)
    }

    /// Copies these bytes into owned storage, allocating as needed.
    pub fn copy_to_owned(self) -> OwnedBytes {
        OwnedBytes(self.0.to_vec())
    }
}

/// A borrowed view of exact valid UTF-8 text.
///
/// Construction and access do not allocate, copy, transcode, or normalize the text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Utf8View<'a>(&'a str);

impl<'a> Utf8View<'a> {
    /// Creates a view over the exact caller-borrowed UTF-8 text.
    pub const fn new(text: &'a str) -> Self {
        Self(text)
    }

    /// Returns the exact borrowed UTF-8 text.
    pub const fn as_str(self) -> &'a str {
        self.0
    }

    /// Returns the UTF-8 bytes that back this exact text.
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0.as_bytes()
    }

    /// Returns a byte view over the same borrowed storage.
    pub const fn as_byte_view(self) -> ByteView<'a> {
        ByteView(self.0.as_bytes())
    }

    /// Copies this text's UTF-8 bytes into owned storage, allocating as needed.
    pub fn copy_to_owned(self) -> OwnedText {
        OwnedText(String::from(self.0))
    }
}

/// A stable error from byte-to-UTF-8 validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DataError {
    /// The input bytes are not a valid UTF-8 sequence.
    InvalidUtf8,
}

/// Owned bytes whose allocation is moved into and out of this wrapper.
///
/// This type does not implement `Clone`; use [`ByteView::copy_to_owned`] for an explicit copy.
#[derive(Debug, Eq, PartialEq)]
pub struct OwnedBytes(Vec<u8>);

impl OwnedBytes {
    /// Takes ownership of a vector and its allocation without copying its bytes.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrows the owned bytes without copying.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns a borrowed view of these bytes.
    pub fn view(&self) -> ByteView<'_> {
        ByteView(self.0.as_slice())
    }

    /// Attempts to move these bytes into valid owned UTF-8 text without copying on success.
    ///
    /// On invalid UTF-8, the returned error retains the original byte vector and its allocation.
    pub fn try_into_text(self) -> Result<OwnedText, OwnedUtf8Error> {
        match String::from_utf8(self.0) {
            Ok(text) => Ok(OwnedText(text)),
            Err(error) => {
                let valid_up_to = error.utf8_error().valid_up_to();
                Err(OwnedUtf8Error {
                    bytes: Self(error.into_bytes()),
                    valid_up_to,
                })
            }
        }
    }

    /// Returns the original vector and allocation without copying.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

/// An invalid UTF-8 result that retains the original owned bytes.
#[derive(Debug, Eq, PartialEq)]
pub struct OwnedUtf8Error {
    bytes: OwnedBytes,
    valid_up_to: usize,
}

impl OwnedUtf8Error {
    /// Returns the byte index at which the first invalid UTF-8 sequence begins.
    pub const fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// Borrows the original invalid bytes without copying.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_bytes()
    }

    /// Returns the original byte vector and allocation without copying.
    pub fn into_bytes(self) -> OwnedBytes {
        self.bytes
    }
}

/// Owned valid UTF-8 text whose allocation can be transferred without copying.
///
/// This type does not implement `Clone`; use [`Utf8View::copy_to_owned`] for an explicit copy.
#[derive(Debug, Eq, PartialEq)]
pub struct OwnedText(String);

impl OwnedText {
    /// Takes ownership of a string and its allocation without copying its bytes.
    pub fn from_string(text: String) -> Self {
        Self(text)
    }

    /// Borrows the owned UTF-8 text without copying.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Returns a borrowed view of this text.
    pub fn view(&self) -> Utf8View<'_> {
        Utf8View(self.0.as_str())
    }

    /// Returns the UTF-8 bytes backing this text without copying.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Moves the string's backing vector into owned bytes without copying.
    pub fn into_bytes(self) -> OwnedBytes {
        OwnedBytes(self.0.into_bytes())
    }

    /// Returns the original string and allocation without copying.
    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::{ByteView, DataError, OwnedBytes, OwnedText, Utf8View};
    use alloc::{string::String, vec, vec::Vec};

    #[test]
    fn borrowed_views_keep_exact_source_storage() {
        let bytes = [0, 0xff, b'a'];
        let byte_view = ByteView::new(&bytes);
        assert_eq!(byte_view.as_bytes(), &bytes);
        assert_eq!(byte_view.as_bytes().as_ptr(), bytes.as_ptr());

        let text = "café";
        let utf8_view = Utf8View::new(text);
        assert_eq!(utf8_view.as_str(), text);
        assert_eq!(utf8_view.as_str().as_ptr(), text.as_ptr());
        assert_eq!(utf8_view.as_bytes().as_ptr(), text.as_bytes().as_ptr());
        assert_eq!(utf8_view.as_byte_view().as_bytes(), text.as_bytes());

        let valid_bytes = b"hello \xc3\xa9";
        let converted = ByteView::new(valid_bytes).try_as_utf8().unwrap();
        assert_eq!(converted.as_bytes().as_ptr(), valid_bytes.as_ptr());
        assert_eq!(converted.as_str().as_bytes(), valid_bytes);
    }

    #[test]
    fn invalid_utf8_is_rejected_and_owned_error_preserves_bytes() {
        let invalid = [b'a', 0xf0, 0x28, 0x8c, 0x28];
        assert_eq!(
            ByteView::new(&invalid).try_as_utf8(),
            Err(DataError::InvalidUtf8)
        );

        let mut owned = Vec::with_capacity(32);
        owned.extend_from_slice(&invalid);
        let pointer = owned.as_ptr();
        let capacity = owned.capacity();
        let error = OwnedBytes::from_vec(owned).try_into_text().unwrap_err();
        assert_eq!(error.valid_up_to(), 1);
        assert_eq!(error.as_bytes(), &invalid);
        let recovered = error.into_bytes().into_vec();
        assert_eq!(recovered.as_ptr(), pointer);
        assert_eq!(recovered.capacity(), capacity);
    }

    #[test]
    fn explicit_owned_copies_do_not_follow_later_source_mutation() {
        let mut source_bytes = vec![1_u8, 2, 3];
        let copied_bytes = ByteView::new(&source_bytes).copy_to_owned();
        source_bytes[0] = 9;
        assert_eq!(copied_bytes.as_bytes(), &[1, 2, 3]);

        let mut source_text = String::from("first");
        let copied_text = Utf8View::new(&source_text).copy_to_owned();
        source_text.push_str(" second");
        assert_eq!(copied_text.as_str(), "first");
    }

    #[test]
    fn allocation_transfers_preserve_the_backing_buffer() {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(b"owned text");
        let pointer = bytes.as_ptr();
        let capacity = bytes.capacity();

        let owned_bytes = OwnedBytes::from_vec(bytes);
        assert_eq!(owned_bytes.as_bytes().as_ptr(), pointer);
        let text = owned_bytes.try_into_text().expect("valid UTF-8");
        assert_eq!(text.as_bytes().as_ptr(), pointer);
        let bytes = text.into_bytes();
        assert_eq!(bytes.as_bytes().as_ptr(), pointer);
        let bytes = bytes.into_vec();
        assert_eq!(bytes.as_ptr(), pointer);
        assert_eq!(bytes.capacity(), capacity);

        let mut text = String::with_capacity(64);
        text.push_str("owned text");
        let pointer = text.as_ptr();
        let capacity = text.capacity();
        let owned_text = OwnedText::from_string(text);
        assert_eq!(owned_text.as_str().as_ptr(), pointer);
        let text = owned_text.into_string();
        assert_eq!(text.as_ptr(), pointer);
        assert_eq!(text.capacity(), capacity);
    }
}

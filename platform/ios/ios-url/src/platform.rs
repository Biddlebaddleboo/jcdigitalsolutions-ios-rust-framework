use framework_format::Uri;
use objc2::rc::Retained;
use objc2_foundation::{NSString, NSURL};

/// A retained Foundation URL paired with its exact portable source URI
///
/// This value does not normalize or reinterpret the source URI
pub struct NativeUrl<'uri> {
    source: Uri<'uri>,
    native: Retained<NSURL>,
}

impl<'uri> NativeUrl<'uri> {
    /// Builds a strict `NSURL` from an absolute portable URI
    ///
    /// Foundation may reject a syntactically valid `Uri`; input is not auto-encoded
    pub fn new(source: Uri<'uri>) -> Result<Self, NativeUrlError> {
        let text = NSString::from_str(source.as_str());
        let native = NSURL::URLWithString_encodingInvalidCharacters(&text, false)
            .ok_or(NativeUrlError::FoundationRejectedUri)?;
        Ok(Self { source, native })
    }

    /// Returns the exact source URI retained by this owner
    pub const fn source_uri(&self) -> Uri<'uri> {
        self.source
    }

    /// Borrows the retained `NSURL` for this owner's lifetime
    pub fn as_ns_url(&self) -> &NSURL {
        &self.native
    }
}

/// A failure from strict Foundation URI conversion
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NativeUrlError {
    /// Foundation rejected the exact source URI text
    FoundationRejectedUri,
}

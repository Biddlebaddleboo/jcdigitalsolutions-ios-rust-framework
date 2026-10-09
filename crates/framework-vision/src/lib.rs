#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable, framework-owned Vision capability values."]

/// A system snapshot of support for one Vision text-recognition request revision.
///
/// This value means only that `VNRecognizeTextRequest.supportedRevisions` contains the requested
/// revision. It does not mean text recognition has run, a model is ready, or any image will be
/// recognized successfully.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TextRecognitionRevisionSupport {
    revision: u32,
    supported: bool,
}

impl TextRecognitionRevisionSupport {
    /// Creates a support value from the platform's request-revision membership result.
    pub const fn from_system(revision: u32, supported: bool) -> Self {
        Self {
            revision,
            supported,
        }
    }

    /// Returns the Vision text-recognition request revision that was queried.
    pub const fn revision(self) -> u32 {
        self.revision
    }

    /// Returns whether the system lists the queried revision as supported.
    pub const fn is_supported(self) -> bool {
        self.supported
    }
}

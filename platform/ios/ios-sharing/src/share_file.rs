use core::fmt;
use std::error::Error;
use std::path::PathBuf;

pub use crate::document_export_state::DocumentExportFileError as ShareFileStagingError;
use crate::document_export_state::StagedExportFile;

/// The maximum payload accepted by [`IosShareFile::from_bytes`], in bytes.
pub const MAX_IOS_SHARE_FILE_BYTES: u32 = 16 * 1024 * 1024;

/// Why an owned local file for the iOS share sheet could not be staged.
#[derive(Debug)]
#[non_exhaustive]
pub enum ShareFileError {
    /// The payload exceeds [`MAX_IOS_SHARE_FILE_BYTES`].
    TooLarge,
    /// The file name or sandbox staging operation was invalid.
    Staging(ShareFileStagingError),
}

impl fmt::Display for ShareFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => write!(
                formatter,
                "share file exceeds the {MAX_IOS_SHARE_FILE_BYTES}-byte limit"
            ),
            Self::Staging(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl Error for ShareFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TooLarge => None,
            Self::Staging(error) => Some(error),
        }
    }
}

/// An app-sandbox file staged for one `UIActivityViewController` presentation.
///
/// Construction copies the exact payload bytes to a uniquely named file in the app temporary
/// directory and rejects payloads above [`MAX_IOS_SHARE_FILE_BYTES`]. The file name must be one
/// safe path component. A share session consumes this value and owns the file until UIKit reports
/// activity completion/dismissal or releases the activity controller. No security-scoped access
/// is needed for this app-owned temporary file. Arbitrary caller-supplied `file://` URLs remain
/// unsupported by generic share requests.
pub struct IosShareFile {
    staged: StagedExportFile,
}

impl IosShareFile {
    /// Stages bounded caller-owned bytes under `file_name`.
    ///
    /// This synchronous copy may be called on a worker thread. The share session later validates
    /// that the owned path remains a regular file inside the app temporary directory before it
    /// creates the native file URL.
    pub fn from_bytes(file_name: &str, bytes: &[u8]) -> Result<Self, ShareFileError> {
        if !share_file_size_is_allowed(bytes.len()) {
            return Err(ShareFileError::TooLarge);
        }
        let staged =
            StagedExportFile::from_bytes(file_name, bytes).map_err(ShareFileError::Staging)?;
        Ok(Self { staged })
    }

    /// Stages bounded UTF-8 text under `file_name` without changing its encoding or contents.
    pub fn from_utf8(file_name: &str, text: &str) -> Result<Self, ShareFileError> {
        Self::from_bytes(file_name, text.as_bytes())
    }

    pub(crate) fn validated_path(&self) -> Result<PathBuf, ShareFileError> {
        self.staged
            .validated_path()
            .map_err(ShareFileError::Staging)
    }

    pub(crate) fn into_staged(self) -> StagedExportFile {
        self.staged
    }
}

pub(crate) const fn share_file_size_is_allowed(size: usize) -> bool {
    size <= MAX_IOS_SHARE_FILE_BYTES as usize
}

#[cfg(test)]
mod tests {
    use super::{
        IosShareFile, MAX_IOS_SHARE_FILE_BYTES, ShareFileError, share_file_size_is_allowed,
    };
    use std::fs;

    #[test]
    fn utf8_file_stages_exact_named_bytes_under_validated_sandbox_path() {
        let text = "Rideshare Bot – résumé";
        let file = IosShareFile::from_utf8("logs.txt", text).unwrap();
        let path = file.validated_path().unwrap();
        assert_eq!(path.file_name().unwrap(), "logs.txt");
        assert_eq!(fs::read(path).unwrap(), text.as_bytes());
    }

    #[test]
    fn arbitrary_file_names_are_rejected_and_staged_bytes_are_cleaned() {
        assert!(matches!(
            IosShareFile::from_bytes("../escape.txt", b"data"),
            Err(ShareFileError::Staging(
                crate::document_export_state::DocumentExportFileError::InvalidFileName
            ))
        ));
    }

    #[test]
    fn byte_limit_accepts_boundary_and_rejects_the_next_byte() {
        assert!(share_file_size_is_allowed(
            MAX_IOS_SHARE_FILE_BYTES as usize
        ));
        assert!(!share_file_size_is_allowed(
            MAX_IOS_SHARE_FILE_BYTES as usize + 1
        ));
        let oversized = std::vec![0; MAX_IOS_SHARE_FILE_BYTES as usize + 1];
        assert!(matches!(
            IosShareFile::from_bytes("oversized.bin", &oversized),
            Err(ShareFileError::TooLarge)
        ));
    }
}

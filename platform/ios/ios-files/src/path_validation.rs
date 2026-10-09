use std::ffi::CString;
use std::vec::Vec;

use framework_files::FileError;

pub(crate) fn path_parts(text: &str) -> Result<Vec<CString>, FileError> {
    let bytes = text.as_bytes();
    if text.is_empty()
        || text.starts_with('/')
        || text.ends_with('/')
        || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        || bytes.iter().any(|byte| *byte == 0 || *byte == b'\\')
    {
        return Err(FileError::InvalidPath);
    }
    text.split('/')
        .map(|part| {
            if part.is_empty() || part == "." || part == ".." {
                return Err(FileError::InvalidPath);
            }
            CString::new(part).map_err(|_| FileError::InvalidPath)
        })
        .collect()
}

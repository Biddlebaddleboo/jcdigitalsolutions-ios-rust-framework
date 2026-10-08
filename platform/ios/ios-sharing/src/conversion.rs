use alloc::string::String;
use alloc::vec::Vec;
use framework_core::{Error, ErrorKind};
use framework_sharing::ClipboardError;

pub(crate) fn decode_utf8(bytes: Vec<u8>) -> Result<String, ClipboardError> {
    String::from_utf8(bytes)
        .map_err(|_| ClipboardError::Backend(Error::new(ErrorKind::InvalidInput)))
}

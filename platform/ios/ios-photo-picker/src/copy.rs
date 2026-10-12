use std::ffi::OsStr;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

/// A copied image whose app-temporary file is removed when this value is dropped.
#[derive(Debug)]
pub struct IosPickedImage {
    path: PathBuf,
    directory: Option<PathBuf>,
    byte_len: u64,
}

impl IosPickedImage {
    /// The app-owned image path. The path remains valid until this value is dropped or transferred.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The exact number of copied provider bytes.
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }

    /// Transfers the file path to the caller and disables automatic cleanup.
    ///
    /// The caller owns the selected file and its temporary parent directory after this call and
    /// must remove them when they are no longer needed.
    pub fn into_path(mut self) -> PathBuf {
        self.directory.take();
        core::mem::take(&mut self.path)
    }
}

impl Drop for IosPickedImage {
    fn drop(&mut self) {
        if let Some(directory) = self.directory.take() {
            let _ = fs::remove_dir_all(directory);
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CopyError {
    InvalidByteLimit,
    NotRegularFile,
    EmptyFile,
    TooLarge { limit: u64 },
    Cancelled,
    Io(String),
}

pub(crate) fn copy_provider_file(
    source: &Path,
    temporary_root: &Path,
    max_bytes: u64,
    mut is_active: impl FnMut() -> bool,
) -> Result<IosPickedImage, CopyError> {
    if max_bytes == 0 {
        return Err(CopyError::InvalidByteLimit);
    }
    let metadata = fs::symlink_metadata(source).map_err(io_error)?;
    if !metadata.file_type().is_file() {
        return Err(CopyError::NotRegularFile);
    }
    if metadata.len() == 0 {
        return Err(CopyError::EmptyFile);
    }
    if metadata.len() > max_bytes {
        return Err(CopyError::TooLarge { limit: max_bytes });
    }
    if !is_active() {
        return Err(CopyError::Cancelled);
    }

    let directory = create_temp_directory(temporary_root).map_err(io_error)?;
    let extension = source
        .extension()
        .and_then(OsStr::to_str)
        .filter(|extension| {
            !extension.is_empty()
                && extension.len() <= 16
                && extension.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    let destination = directory.join(format!("selected{extension}"));
    let result = copy_into(source, &destination, max_bytes, &mut is_active);
    match result {
        Ok(byte_len) => Ok(IosPickedImage {
            path: destination,
            directory: Some(directory),
            byte_len,
        }),
        Err(error) => {
            let _ = fs::remove_dir_all(directory);
            Err(error)
        }
    }
}

fn copy_into(
    source: &Path,
    destination: &Path,
    max_bytes: u64,
    is_active: &mut impl FnMut() -> bool,
) -> Result<u64, CopyError> {
    let mut input = File::open(source).map_err(io_error)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)
        .map_err(io_error)?;
    let mut buffer = [0_u8; 32 * 1024];
    let mut total = 0_u64;
    loop {
        if !is_active() {
            return Err(CopyError::Cancelled);
        }
        let count = input.read(&mut buffer).map_err(io_error)?;
        if count == 0 {
            break;
        }
        total = checked_total(total, count as u64, max_bytes)?;
        output.write_all(&buffer[..count]).map_err(io_error)?;
    }
    if total == 0 {
        return Err(CopyError::EmptyFile);
    }
    output.flush().map_err(io_error)?;
    Ok(total)
}

fn create_temp_directory(base: &Path) -> std::io::Result<PathBuf> {
    for _ in 0..64 {
        let serial = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = base.join(format!("ios-photo-picker-{}-{serial}", std::process::id()));
        match DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not create a unique photo-picker directory",
    ))
}

fn io_error(error: std::io::Error) -> CopyError {
    CopyError::Io(error.to_string())
}

pub(crate) fn checked_total(total: u64, additional: u64, max_bytes: u64) -> Result<u64, CopyError> {
    total
        .checked_add(additional)
        .filter(|total| *total <= max_bytes)
        .ok_or(CopyError::TooLarge { limit: max_bytes })
}

#[cfg(test)]
mod tests {
    use super::{CopyError, checked_total, copy_provider_file};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    fn fixture_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ios-photo-picker-test-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn copies_provider_bytes_into_owned_app_temp_file_and_cleans_up() {
        let fixture = fixture_dir();
        let source = fixture.join("incoming.heic");
        fs::write(&source, b"image bytes\0preserved").unwrap();
        let storage = fixture.join("storage");
        fs::create_dir(&storage).unwrap();
        let image = copy_provider_file(&source, &storage, 1024, || true).unwrap();
        let selected_path = image.path().to_owned();
        assert_ne!(selected_path, source);
        assert!(selected_path.ends_with("selected.heic"));
        assert_eq!(image.byte_len(), 21);
        assert_eq!(fs::read(&selected_path).unwrap(), b"image bytes\0preserved");
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(selected_path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&selected_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        drop(image);
        assert!(!selected_path.exists());
        assert_eq!(fs::read_dir(&storage).unwrap().count(), 0);
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn transferred_path_survives_value_drop_until_caller_cleanup() {
        let fixture = fixture_dir();
        let source = fixture.join("incoming.png");
        fs::write(&source, b"pixels").unwrap();
        let storage = fixture.join("storage");
        fs::create_dir(&storage).unwrap();
        let image = copy_provider_file(&source, &storage, 100, || true).unwrap();
        let directory = image.path().parent().unwrap().to_owned();
        let path = image.into_path();
        assert!(path.exists());
        drop(path);
        assert!(directory.exists());
        fs::remove_dir_all(directory).unwrap();
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn rejects_zero_and_exceeded_byte_limits() {
        let fixture = fixture_dir();
        let source = fixture.join("image.jpg");
        fs::write(&source, b"123").unwrap();
        let storage = fixture.join("storage");
        fs::create_dir(&storage).unwrap();
        assert!(matches!(
            copy_provider_file(&source, &storage, 0, || true),
            Err(CopyError::InvalidByteLimit)
        ));
        assert!(matches!(
            copy_provider_file(&source, &storage, 2, || true),
            Err(CopyError::TooLarge { limit: 2 })
        ));
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn rejects_non_regular_and_empty_inputs() {
        let fixture = fixture_dir();
        let empty = fixture.join("empty");
        fs::write(&empty, b"").unwrap();
        let storage = fixture.join("storage");
        fs::create_dir(&storage).unwrap();
        assert!(matches!(
            copy_provider_file(&empty, &storage, 10, || true),
            Err(CopyError::EmptyFile)
        ));
        assert!(matches!(
            copy_provider_file(&fixture, &storage, 10, || true),
            Err(CopyError::NotRegularFile)
        ));
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn stops_copy_when_session_is_cancelled() {
        let fixture = fixture_dir();
        let source = fixture.join("image.jpg");
        fs::write(&source, vec![1; 64 * 1024]).unwrap();
        let storage = fixture.join("storage");
        fs::create_dir(&storage).unwrap();
        let calls = std::cell::Cell::new(0);
        let active = || {
            let count = calls.get() + 1;
            calls.set(count);
            count == 1
        };
        assert!(matches!(
            copy_provider_file(&source, &storage, 128 * 1024, active),
            Err(CopyError::Cancelled)
        ));
        assert_eq!(fs::read_dir(&storage).unwrap().count(), 0);
        fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn checked_copy_size_detects_overflow_and_limit_excess() {
        assert_eq!(checked_total(4, 5, 9), Ok(9));
        assert_eq!(
            checked_total(5, 5, 9),
            Err(CopyError::TooLarge { limit: 9 })
        );
        assert_eq!(
            checked_total(u64::MAX, 1, u64::MAX),
            Err(CopyError::TooLarge { limit: u64::MAX })
        );
    }
}

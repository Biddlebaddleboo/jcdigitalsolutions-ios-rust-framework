use framework_photos::PhotoLibraryAddOnlyAuthorizationStatus;
use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::Path;

/// The file kind for one PhotoKit asset-resource save.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhotosAssetFileKind {
    /// One JPEG image resource.
    Jpeg,
    /// One finalized movie resource.
    Movie,
}

/// A local source path could not be prepared for a Photos save.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhotosSourcePathError {
    /// The path is not absolute.
    NotAbsolute,
    /// The path cannot be represented as valid UTF-8 without data loss.
    NonUtf8,
    /// The path contains an embedded NUL byte.
    EmbeddedNul,
    /// The file extension does not match the requested JPEG or movie resource.
    UnsupportedExtension,
    /// The file could not be canonicalized or opened; this stores only the stable OS error kind.
    Io(ErrorKind),
    /// The source does not name a regular file.
    NotRegularFile,
}

/// A Photos save failed before or during the PhotoKit transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhotosSaveError {
    /// The source path failed local file validation.
    SourcePath(PhotosSourcePathError),
    /// Add-only access is not authorized; save operations never request permission.
    AddOnlyNotAuthorized(PhotoLibraryAddOnlyAuthorizationStatus),
    /// PhotoKit rejected the transaction with a copied native error.
    PhotoKit {
        /// The native error domain.
        domain: String,
        /// The native error code.
        code: isize,
    },
    /// PhotoKit reported failure without a native error object.
    PhotoKitFailedWithoutError,
    /// The native change block panicked; unwind was contained at the Objective-C boundary.
    ChangeBlockPanicked,
}

/// Rejects every status except explicit add-only authorization.
pub(crate) fn require_add_only_authorized(
    status: PhotoLibraryAddOnlyAuthorizationStatus,
) -> Result<(), PhotosSaveError> {
    if status == PhotoLibraryAddOnlyAuthorizationStatus::Authorized {
        Ok(())
    } else {
        Err(PhotosSaveError::AddOnlyNotAuthorized(status))
    }
}

/// Validates a caller-owned path and returns its canonical, lossless UTF-8 representation.
pub(crate) fn validated_source_path(
    path: &Path,
    kind: PhotosAssetFileKind,
) -> Result<String, PhotosSourcePathError> {
    if !path.is_absolute() {
        return Err(PhotosSourcePathError::NotAbsolute);
    }
    let original = path.to_str().ok_or(PhotosSourcePathError::NonUtf8)?;
    if original.contains('\0') {
        return Err(PhotosSourcePathError::EmbeddedNul);
    }

    let canonical =
        fs::canonicalize(path).map_err(|error| PhotosSourcePathError::Io(error.kind()))?;
    let canonical_text = canonical.to_str().ok_or(PhotosSourcePathError::NonUtf8)?;
    if canonical_text.contains('\0') {
        return Err(PhotosSourcePathError::EmbeddedNul);
    }
    if !matches_extension(&canonical, kind) {
        return Err(PhotosSourcePathError::UnsupportedExtension);
    }

    let file = File::open(&canonical).map_err(|error| PhotosSourcePathError::Io(error.kind()))?;
    let metadata = file
        .metadata()
        .map_err(|error| PhotosSourcePathError::Io(error.kind()))?;
    if !metadata.is_file() {
        return Err(PhotosSourcePathError::NotRegularFile);
    }
    Ok(canonical_text.to_owned())
}

fn matches_extension(path: &Path, kind: PhotosAssetFileKind) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    match kind {
        PhotosAssetFileKind::Jpeg => {
            extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg")
        }
        PhotosAssetFileKind::Movie => {
            extension.eq_ignore_ascii_case("mov")
                || extension.eq_ignore_ascii_case("mp4")
                || extension.eq_ignore_ascii_case("m4v")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn save_requires_explicit_add_only_authorization() {
        assert_eq!(
            require_add_only_authorized(PhotoLibraryAddOnlyAuthorizationStatus::Authorized),
            Ok(())
        );
        for status in [
            PhotoLibraryAddOnlyAuthorizationStatus::NotDetermined,
            PhotoLibraryAddOnlyAuthorizationStatus::Restricted,
            PhotoLibraryAddOnlyAuthorizationStatus::Denied,
            PhotoLibraryAddOnlyAuthorizationStatus::Unknown,
        ] {
            assert_eq!(
                require_add_only_authorized(status),
                Err(PhotosSaveError::AddOnlyNotAuthorized(status))
            );
        }
    }

    struct TempPath(PathBuf);

    impl TempPath {
        fn file(extension: &str) -> Self {
            let serial = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "ios-photos-save-{}-{serial}.{extension}",
                std::process::id()
            ));
            fs::write(&path, b"fixture").unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempPath {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn accepts_lossless_jpeg_and_movie_paths() {
        let jpeg = TempPath::file("JPEG");
        let movie = TempPath::file("mov");
        assert!(validated_source_path(jpeg.path(), PhotosAssetFileKind::Jpeg).is_ok());
        assert!(validated_source_path(movie.path(), PhotosAssetFileKind::Movie).is_ok());
    }

    #[test]
    fn rejects_relative_and_unsupported_paths() {
        assert_eq!(
            validated_source_path(Path::new("relative.jpg"), PhotosAssetFileKind::Jpeg),
            Err(PhotosSourcePathError::NotAbsolute)
        );
        let png = TempPath::file("png");
        assert_eq!(
            validated_source_path(png.path(), PhotosAssetFileKind::Jpeg),
            Err(PhotosSourcePathError::UnsupportedExtension)
        );
    }

    #[test]
    fn rejects_missing_nonregular_and_nul_paths() {
        let missing = std::env::temp_dir().join(format!(
            "ios-photos-save-missing-{}.jpg",
            std::process::id()
        ));
        assert_eq!(
            validated_source_path(&missing, PhotosAssetFileKind::Jpeg),
            Err(PhotosSourcePathError::Io(ErrorKind::NotFound))
        );

        let directory = std::env::temp_dir().join(format!(
            "ios-photos-save-directory-{}.jpg",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        assert_eq!(
            validated_source_path(&directory, PhotosAssetFileKind::Jpeg),
            Err(PhotosSourcePathError::NotRegularFile)
        );
        fs::remove_dir(directory).unwrap();

        let nul = PathBuf::from("/tmp/invalid\0.jpg");
        assert_eq!(
            validated_source_path(&nul, PhotosAssetFileKind::Jpeg),
            Err(PhotosSourcePathError::EmbeddedNul)
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_non_utf8_paths_without_lossy_conversion() {
        use std::os::unix::ffi::OsStrExt;

        let path = Path::new(std::ffi::OsStr::from_bytes(b"/tmp/non-utf8-\xff.jpg"));
        assert_eq!(
            validated_source_path(path, PhotosAssetFileKind::Jpeg),
            Err(PhotosSourcePathError::NonUtf8)
        );
    }
}

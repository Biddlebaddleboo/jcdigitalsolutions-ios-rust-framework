use objc2_foundation::NSString;
use objc2_model_io::MDLAsset;

/// Ask ModelIO whether it can read asset files with `extension`.
///
/// The wrapper passes the caller's string to `MDLAsset.canImportFileExtension` without
/// normalization. No URL or file data is supplied. A positive result is not proof that a
/// particular file is valid or readable, and does not establish rendering or GPU support.
///
/// This is an iOS-only ModelIO query. It adds no main-thread requirement or thread-safety promise.
pub fn can_import_file_extension(extension: &str) -> bool {
    let extension = NSString::from_str(extension);

    // SAFETY: `extension` is a live immutable NSString with the type required by the generated
    // class-method binding, and it remains alive for the full message send.
    unsafe { MDLAsset::canImportFileExtension(&extension) }
}

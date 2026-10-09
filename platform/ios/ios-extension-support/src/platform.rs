use alloc::string::{String, ToString};

use objc2::runtime::AnyObject;
use objc2_foundation::{NSBundle, NSDictionary, NSString, NSURL};

/// Errors while reading a caller-selected app-extension bundle's point identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionMetadataError {
    /// The path is not absolute, contains NUL, or does not name a `.appex` directory.
    InvalidBundlePath,
    /// Foundation could not create a bundle for the supplied URL.
    BundleUnavailable,
    /// Foundation did not provide the bundle's information dictionary.
    InfoDictionaryUnavailable,
    /// The information dictionary has no `NSExtension` value.
    MissingExtensionDictionary,
    /// The `NSExtension` value is not a dictionary.
    InvalidExtensionDictionaryType,
    /// The `NSExtension` dictionary has no `NSExtensionPointIdentifier` value.
    MissingExtensionPointIdentifier,
    /// The point identifier is not a string.
    InvalidExtensionPointIdentifierType,
    /// The point identifier string is empty.
    EmptyExtensionPointIdentifier,
}

/// Read and copy `NSExtension.NSExtensionPointIdentifier` from one caller-supplied `.appex` path.
///
/// The path must be absolute and end in a non-empty name with the exact `.appex` suffix. The
/// function reads only the documented `Info.plist` key and returns its string as Rust-owned UTF-8.
/// It does not call `NSBundle.load`, execute extension code, search for other bundles, or validate
/// extension-point-specific attributes. A returned identifier is metadata only; it does not prove
/// that the extension is installed, enabled, approved, entitled, launchable, or usable by a host.
pub fn read_extension_point_identifier(
    bundle_path: &str,
) -> Result<String, ExtensionMetadataError> {
    if !is_absolute_appex_path(bundle_path) {
        return Err(ExtensionMetadataError::InvalidBundlePath);
    }

    let path = NSString::from_str(bundle_path);
    let url = NSURL::fileURLWithPath(&path);
    let bundle = NSBundle::bundleWithURL(&url).ok_or(ExtensionMetadataError::BundleUnavailable)?;
    let info = bundle
        .infoDictionary()
        .ok_or(ExtensionMetadataError::InfoDictionaryUnavailable)?;

    let extension_key = NSString::from_str("NSExtension");
    let extension_object = info
        .objectForKey(&extension_key)
        .ok_or(ExtensionMetadataError::MissingExtensionDictionary)?;
    let extension_dictionary: &NSDictionary<AnyObject, AnyObject> = extension_object
        .downcast_ref::<NSDictionary>()
        .ok_or(ExtensionMetadataError::InvalidExtensionDictionaryType)?;

    let point_identifier_key = NSString::from_str("NSExtensionPointIdentifier");
    let point_identifier_object = extension_dictionary
        .objectForKey(&point_identifier_key)
        .ok_or(ExtensionMetadataError::MissingExtensionPointIdentifier)?;
    let point_identifier = point_identifier_object
        .downcast_ref::<NSString>()
        .ok_or(ExtensionMetadataError::InvalidExtensionPointIdentifierType)?;
    if point_identifier.is_empty() {
        return Err(ExtensionMetadataError::EmptyExtensionPointIdentifier);
    }

    Ok(point_identifier.to_string())
}

fn is_absolute_appex_path(path: &str) -> bool {
    if !path.starts_with('/') || path.contains('\0') {
        return false;
    }

    path.rsplit('/')
        .next()
        .and_then(|name| name.strip_suffix(".appex"))
        .is_some_and(|stem| !stem.is_empty())
}

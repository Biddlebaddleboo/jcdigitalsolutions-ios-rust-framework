#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![doc = "Read-only main application bundle resource backend for `framework-resources`."]

use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use framework_resources::{ResourceBackend, ResourceError, ResourcePath};
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::{
    NSBundle, NSCocoaErrorDomain, NSData, NSDataReadingOptions, NSError, NSFileNoSuchFileError,
    NSFileReadNoSuchFileError, NSString,
};

/// A caller-owned backend that reads ordinary files from the main application bundle.
///
/// Reads are synchronous and may block. Foundation loads each resource into `NSData`, then the
/// backend copies those bytes into a caller-owned Rust vector. Lookup uses exact, nonlocalized
/// bundle names; it does not enumerate directories or access asset catalogs.
pub struct IosResources {
    bundle: Retained<NSBundle>,
}

impl IosResources {
    /// Retains Foundation's main application bundle for this backend.
    pub fn new() -> Self {
        Self {
            bundle: NSBundle::mainBundle(),
        }
    }
}

impl Default for IosResources {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceBackend for IosResources {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn read(&mut self, path: ResourcePath<'_>) -> Result<Vec<u8>, ResourceError> {
        autoreleasepool(|_| {
            let relative = path.relative();
            let (subdirectory, filename) = relative
                .rsplit_once('/')
                .map_or((None, relative), |(directory, filename)| {
                    (Some(directory), filename)
                });
            let filename = NSString::from_str(filename);
            let subdirectory = subdirectory.map(NSString::from_str);
            let url = self
                .bundle
                .URLForResource_withExtension_subdirectory_localization(
                    Some(&filename),
                    None,
                    subdirectory.as_deref(),
                    None,
                );
            let Some(url) = url else {
                return Err(ResourceError::Backend(Error::new(ErrorKind::NotFound)));
            };
            match NSData::dataWithContentsOfURL_options_error(&url, NSDataReadingOptions(0)) {
                Ok(data) => Ok(data.to_vec()),
                Err(error) => Err(map_read_error(&error)),
            }
        })
    }
}

fn map_read_error(error: &NSError) -> ResourceError {
    let code = error.code();
    let is_cocoa_error = error.domain().to_string() == unsafe { NSCocoaErrorDomain }.to_string();
    let kind =
        if is_cocoa_error && (code == NSFileNoSuchFileError || code == NSFileReadNoSuchFileError) {
            ErrorKind::NotFound
        } else {
            ErrorKind::Platform
        };
    let mut framework_error = Error::new(kind);
    if let Ok(code) = i32::try_from(code) {
        if let Some(code) = PlatformErrorCode::new(code) {
            framework_error = framework_error.with_platform_code(code);
        }
    }
    ResourceError::Backend(framework_error)
}

use framework_core::ErrorKind;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::{
    NSData, NSOperatingSystemVersion, NSProcessInfo, NSURL, NSURLBookmarkResolutionOptions,
};

use crate::{FileError, backend_error, foundation_error};

/// A retained file URL resolved from plain Foundation bookmark data, with the stale bit kept.
///
/// The resolver does not apply to security-scoped bookmark data. It does not establish access or
/// add the resolved URL to the sandbox `Files` facade.
pub struct IosResolvedBookmark {
    url: Retained<NSURL>,
    stale: bool,
}

impl IosResolvedBookmark {
    /// Resolves caller-supplied, non-security-scoped file bookmark data without UI or mounting.
    ///
    /// This operation needs iOS 14.2 or later because earlier systems lack the
    /// `WithoutImplicitStartAccessing` option. That option blocks implicit start for ephemeral
    /// security-scoped URLs, but Apple declares it inapplicable to security-scoped bookmark data.
    /// This method resolves a location only; it does not make bookmark data, set access rights,
    /// start a security scope, coordinate file calls, or register a file presenter. If Foundation
    /// marks the bookmark stale, the caller decides whether and how to replace it.
    ///
    /// # Safety
    ///
    /// `bookmark_data` must not contain security-scoped bookmark data. Foundation does not expose
    /// a query to verify this before resolution, and its no-implicit-start option does not govern
    /// security-scoped bookmark data. Violating this requirement may start security-scoped access
    /// without returning a guard that can balance it.
    ///
    /// # Errors
    ///
    /// Returns `Unsupported` before iOS 14.2, `InvalidInput` when the resolved URL is not a file
    /// URL, or a Foundation error when resolution fails.
    pub unsafe fn resolve_unscoped(bookmark_data: &NSData) -> Result<Self, FileError> {
        let required_version = NSOperatingSystemVersion {
            majorVersion: 14,
            minorVersion: 2,
            patchVersion: 0,
        };
        if !NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(required_version) {
            return Err(backend_error(ErrorKind::Unsupported, None));
        }

        let options = NSURLBookmarkResolutionOptions::WithoutUI
            | NSURLBookmarkResolutionOptions::WithoutMounting
            | NSURLBookmarkResolutionOptions::WithoutImplicitStartAccessing;
        let mut stale = Bool::NO;
        // SAFETY: `stale` is a live writable Objective-C BOOL for this call. The iOS 14.2 check
        // above ensures the implicit-start option is available; the caller guarantees that the
        // input is not security-scoped bookmark data, for which the option does not apply.
        let url = unsafe {
            NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
                bookmark_data,
                options,
                None,
                &mut stale,
            )
        }
        .map_err(|error| foundation_error(&error))?;
        if !url.isFileURL() {
            return Err(backend_error(ErrorKind::InvalidInput, None));
        }

        Ok(Self {
            url,
            stale: stale.as_bool(),
        })
    }

    /// Borrows the retained resolved file URL.
    pub fn url(&self) -> &NSURL {
        &self.url
    }

    /// Reports whether Foundation marked the bookmark data stale.
    pub const fn is_stale(&self) -> bool {
        self.stale
    }
}

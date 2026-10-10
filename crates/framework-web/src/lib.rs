#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Platform-exclusive HTTPS web-view values and a bounded view/navigation contract."]

use framework_core::{Error, ErrorKind, PlatformErrorCode};
use framework_format::Uri;

/// A borrowed absolute HTTPS URI with a nonempty authority host.
///
/// Construction validates RFC 3986 URI syntax and the HTTPS scheme without copying, normalizing,
/// percent-decoding, resolving, or rewriting caller text. Platform backends may apply additional
/// native URL validation. This value is not a portable browser implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HttpsUrl<'a>(Uri<'a>);

impl<'a> HttpsUrl<'a> {
    /// Validates a borrowed absolute HTTPS URI.
    pub fn new(value: &'a str) -> Result<Self, HttpsUrlError> {
        let uri = Uri::new(value).map_err(|_| HttpsUrlError::InvalidUri)?;
        if !uri.scheme().eq_ignore_ascii_case("https") {
            return Err(HttpsUrlError::HttpsRequired);
        }
        if !uri.authority().is_some_and(authority_has_host) {
            return Err(HttpsUrlError::HostRequired);
        }
        Ok(Self(uri))
    }

    /// Returns the exact caller-borrowed URI text.
    pub fn as_str(self) -> &'a str {
        self.0.as_str()
    }
}

fn authority_has_host(authority: &str) -> bool {
    let host_and_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host_and_port)| host_and_port);
    if let Some(bracketed) = host_and_port.strip_prefix('[') {
        return bracketed.split_once(']').is_some_and(|(host, suffix)| {
            !host.is_empty() && (suffix.is_empty() || suffix.starts_with(':'))
        });
    }
    !host_and_port.split(':').next().is_none_or(str::is_empty)
}

/// A portable reason that an HTTPS URL cannot be used to create a view.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HttpsUrlError {
    /// The text is not a valid absolute RFC 3986 URI.
    InvalidUri,
    /// The URI scheme is not HTTPS.
    HttpsRequired,
    /// The URI authority has no nonempty host.
    HostRequired,
}

impl HttpsUrlError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        ErrorKind::InvalidInput
    }
}

/// A stable web-view construction error that preserves optional backend-native error details.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum WebViewError {
    /// A platform URL constructor rejected an otherwise valid [`HttpsUrl`].
    InvalidUrl,
    /// The selected backend could not create or start the view.
    Backend(Error),
}

impl WebViewError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidUrl => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidUrl => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// Current back and forward capability flags for a view.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct NavigationState {
    can_go_back: bool,
    can_go_forward: bool,
}

impl NavigationState {
    /// Creates a navigation state from the backend's current native flags.
    pub const fn new(can_go_back: bool, can_go_forward: bool) -> Self {
        Self {
            can_go_back,
            can_go_forward,
        }
    }

    /// Reports whether the view currently has an earlier navigation entry.
    pub const fn can_go_back(self) -> bool {
        self.can_go_back
    }

    /// Reports whether the view currently has a later navigation entry.
    pub const fn can_go_forward(self) -> bool {
        self.can_go_forward
    }
}

/// Main-thread-affine view/navigation operations for a platform-native web view.
///
/// This contract deliberately excludes page-load events/results, arbitrary URL loads, JavaScript,
/// script-message bridges, file URLs, and browser UI parity. Navigation-state flags can change as
/// native page work proceeds and are snapshots, not a reservation. A command issued without the
/// corresponding back/forward capability is a no-op. Commands do not report whether a page loaded.
pub trait WebView {
    /// Returns the current back/forward capability snapshot.
    fn navigation_state(&self) -> NavigationState;

    /// Navigates to the preceding history item when one exists.
    fn go_back(&mut self);

    /// Navigates to the following history item when one exists.
    fn go_forward(&mut self);

    /// Requests a reload of the current history item.
    fn reload(&mut self);

    /// Requests cancellation of the current page load, if any.
    fn stop_loading(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_url_preserves_exact_text_and_accepts_ipv6_authority() {
        let original = "HTTPS://user:pass@[2001:db8::1]:443/path?q=1#part";
        let url = HttpsUrl::new(original).unwrap();
        assert_eq!(url.as_str(), original);
        assert_eq!(url.as_str().as_ptr(), original.as_ptr());
        assert_eq!(url.as_str().len(), original.len());
    }

    #[test]
    fn https_url_rejects_non_https_or_hostless_uris() {
        assert_eq!(
            HttpsUrl::new("http://example.com"),
            Err(HttpsUrlError::HttpsRequired)
        );
        assert_eq!(
            HttpsUrl::new("https:/path"),
            Err(HttpsUrlError::HostRequired)
        );
        assert_eq!(
            HttpsUrl::new("https://:443/path"),
            Err(HttpsUrlError::HostRequired)
        );
        assert_eq!(
            HttpsUrl::new("https://@/path"),
            Err(HttpsUrlError::HostRequired)
        );
        assert_eq!(
            HttpsUrl::new("relative/path"),
            Err(HttpsUrlError::InvalidUri)
        );
    }

    #[test]
    fn navigation_state_is_a_copyable_flag_snapshot() {
        let state = NavigationState::new(true, false);
        assert!(state.can_go_back());
        assert!(!state.can_go_forward());
        let copied_state = state;
        assert_eq!(state, copied_state);
    }

    #[test]
    fn web_view_error_preserves_backend_category_and_native_code() {
        let code = PlatformErrorCode::new(-7).unwrap();
        let error =
            WebViewError::Backend(Error::new(ErrorKind::PermissionDenied).with_platform_code(code));
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(error.platform_code(), Some(code));

        let invalid_url = WebViewError::InvalidUrl;
        assert_eq!(invalid_url.kind(), ErrorKind::InvalidInput);
        assert_eq!(invalid_url.platform_code(), None);
    }
}

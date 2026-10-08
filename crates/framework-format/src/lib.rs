#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Borrowed, validated RFC 3986 URI values for portable framework code."]

use framework_core::{Error, ErrorKind};
use iri_string::types::{UriReferenceStr, UriStr};

/// A validated RFC 3986 URI that borrows its exact caller-provided text.
///
/// This is an absolute URI with a required scheme. Parsing validates syntax only; it does not
/// normalize, percent-decode, resolve, or interpret the URI for a particular scheme.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Uri<'a>(&'a UriStr);

impl<'a> Uri<'a> {
    /// Validates an absolute URI without copying or changing its text.
    ///
    /// Invalid URI syntax is reported as [`ErrorKind::InvalidInput`]. Unescaped non-ASCII text is
    /// outside this URI-only contract; use percent-encoded UTF-8 where required by the URI syntax.
    pub fn new(value: &'a str) -> Result<Self, Error> {
        UriStr::new(value)
            .map(Self)
            .map_err(|_| Error::new(ErrorKind::InvalidInput))
    }

    /// Returns the exact original caller-borrowed URI text.
    pub fn as_str(self) -> &'a str {
        let uri: &'a UriStr = self.0;
        uri.as_str()
    }

    /// Returns the scheme without its following colon.
    pub fn scheme(self) -> &'a str {
        let uri: &'a UriStr = self.0;
        uri.scheme_str()
    }

    /// Returns the authority without its leading `//`, or `None` when absent.
    ///
    /// A present but empty authority is `Some("")`.
    pub fn authority(self) -> Option<&'a str> {
        let uri: &'a UriStr = self.0;
        uri.authority_str()
    }

    /// Returns the path exactly as written; it may be empty.
    pub fn path(self) -> &'a str {
        let uri: &'a UriStr = self.0;
        uri.path_str()
    }

    /// Returns the query without its leading `?`, or `None` when absent.
    ///
    /// A present but empty query is `Some("")`.
    pub fn query(self) -> Option<&'a str> {
        let uri: &'a UriStr = self.0;
        uri.query_str()
    }

    /// Returns the fragment without its leading `#`, or `None` when absent.
    ///
    /// A present but empty fragment is `Some("")`.
    pub fn fragment(self) -> Option<&'a str> {
        let uri: &'a UriStr = self.0;
        uri.fragment_str()
    }
}

/// A validated RFC 3986 URI-reference that borrows its exact caller-provided text.
///
/// A URI-reference may be absolute or relative. Parsing validates syntax only; it does not
/// normalize, percent-decode, resolve against a base, or interpret the reference for a scheme.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UriReference<'a>(&'a UriReferenceStr);

impl<'a> UriReference<'a> {
    /// Validates a URI-reference without copying or changing its text.
    ///
    /// Invalid URI-reference syntax is reported as [`ErrorKind::InvalidInput`]. Unescaped
    /// non-ASCII text is outside this URI-only contract; use percent-encoded UTF-8 where required
    /// by the URI syntax.
    pub fn new(value: &'a str) -> Result<Self, Error> {
        UriReferenceStr::new(value)
            .map(Self)
            .map_err(|_| Error::new(ErrorKind::InvalidInput))
    }

    /// Returns the exact original caller-borrowed URI-reference text.
    pub fn as_str(self) -> &'a str {
        let reference: &'a UriReferenceStr = self.0;
        reference.as_str()
    }

    /// Returns the scheme without its following colon, or `None` for a relative reference.
    pub fn scheme(self) -> Option<&'a str> {
        let reference: &'a UriReferenceStr = self.0;
        reference.scheme_str()
    }

    /// Returns the authority without its leading `//`, or `None` when absent.
    ///
    /// A present but empty authority is `Some("")`.
    pub fn authority(self) -> Option<&'a str> {
        let reference: &'a UriReferenceStr = self.0;
        reference.authority_str()
    }

    /// Returns the path exactly as written; it may be empty.
    pub fn path(self) -> &'a str {
        let reference: &'a UriReferenceStr = self.0;
        reference.path_str()
    }

    /// Returns the query without its leading `?`, or `None` when absent.
    ///
    /// A present but empty query is `Some("")`.
    pub fn query(self) -> Option<&'a str> {
        let reference: &'a UriReferenceStr = self.0;
        reference.query_str()
    }

    /// Returns the fragment without its leading `#`, or `None` when absent.
    ///
    /// A present but empty fragment is `Some("")`.
    pub fn fragment(self) -> Option<&'a str> {
        let reference: &'a UriReferenceStr = self.0;
        reference.fragment_str()
    }
}

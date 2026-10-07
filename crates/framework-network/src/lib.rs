#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable foreground HTTP values and a runtime-neutral static backend contract."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A stable foreground HTTP error that preserves portable category and native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NetworkError {
    /// A URL does not use an HTTP scheme or has no authority.
    InvalidUrl,
    /// A method or header does not meet this facade's token/value rules.
    InvalidHeaderOrMethod,
    /// A status code is outside the HTTP range supported by this facade.
    InvalidStatusCode,
    /// A body or header value was not valid UTF-8 when a text view was requested.
    InvalidUtf8,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl NetworkError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidUrl
            | Self::InvalidHeaderOrMethod
            | Self::InvalidStatusCode
            | Self::InvalidUtf8 => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidUrl
            | Self::InvalidHeaderOrMethod
            | Self::InvalidStatusCode
            | Self::InvalidUtf8 => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A validated borrowed HTTP or HTTPS URL.
///
/// The facade checks only the scheme, authority presence, and forbidden ASCII whitespace/control
/// bytes. It does not canonicalize, percent-decode, resolve names, or implement full URL parsing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HttpUrl<'a>(&'a str);

impl<'a> HttpUrl<'a> {
    /// Creates an HTTP(S) URL without copying or normalizing its text.
    pub fn new(value: &'a str) -> Result<Self, NetworkError> {
        let Some((scheme, rest)) = value.split_once("://") else {
            return Err(NetworkError::InvalidUrl);
        };
        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return Err(NetworkError::InvalidUrl);
        }
        let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
        if authority.is_empty() || value.bytes().any(|byte| byte <= 0x20 || byte == 0x7f) {
            return Err(NetworkError::InvalidUrl);
        }
        Ok(Self(value))
    }

    /// Returns the original caller-borrowed URL text.
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// A validated borrowed HTTP method token.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HttpMethod<'a>(&'a str);

impl<'a> HttpMethod<'a> {
    /// Creates a method token without copying or changing its case.
    pub fn new(value: &'a str) -> Result<Self, NetworkError> {
        if valid_token(value) {
            Ok(Self(value))
        } else {
            Err(NetworkError::InvalidHeaderOrMethod)
        }
    }

    /// Returns the original caller-borrowed method token.
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

fn valid_header_value(value: &[u8]) -> bool {
    value
        .iter()
        .all(|byte| *byte == b'\t' || (*byte >= 0x20 && *byte != 0x7f))
}

/// A borrowed request header; duplicate names and caller order are preserved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Header<'a> {
    name: &'a str,
    value: &'a [u8],
}

impl<'a> Header<'a> {
    /// Creates a header after validating its ASCII token name and opaque field-value bytes.
    pub fn new(name: &'a str, value: &'a [u8]) -> Result<Self, NetworkError> {
        if !name.is_ascii() || !valid_token(name) || !valid_header_value(value) {
            return Err(NetworkError::InvalidHeaderOrMethod);
        }
        Ok(Self { name, value })
    }

    /// Returns the original caller-borrowed header name without case normalization.
    pub const fn name(self) -> &'a str {
        self.name
    }

    /// Returns the original caller-borrowed opaque field-value bytes.
    pub const fn value(self) -> &'a [u8] {
        self.value
    }
}

/// A borrowed HTTP request with no hidden body/header allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpRequest<'a> {
    method: HttpMethod<'a>,
    url: HttpUrl<'a>,
    headers: &'a [Header<'a>],
    body: Option<&'a [u8]>,
}

impl<'a> HttpRequest<'a> {
    /// Creates a request that borrows its method, URL, headers, and optional body.
    pub fn new(
        method: HttpMethod<'a>,
        url: HttpUrl<'a>,
        headers: &'a [Header<'a>],
        body: Option<&'a [u8]>,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            body,
        }
    }

    /// Returns the method token.
    pub const fn method(self) -> HttpMethod<'a> {
        self.method
    }

    /// Returns the URL value.
    pub const fn url(self) -> HttpUrl<'a> {
        self.url
    }

    /// Borrows request headers in caller order, including duplicate names.
    pub const fn headers(self) -> &'a [Header<'a>] {
        self.headers
    }

    /// Borrows the optional body bytes.
    pub const fn body(self) -> Option<&'a [u8]> {
        self.body
    }
}

/// A validated fixed-width HTTP status code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct StatusCode(u16);

impl StatusCode {
    /// Creates a status code in the range 100 through 599.
    pub const fn new(value: u16) -> Option<Self> {
        if value >= 100 && value <= 599 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the fixed-width status code.
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// An owned response header that preserves duplicate names and backend order.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ResponseHeader {
    name: String,
    value: Vec<u8>,
}

impl ResponseHeader {
    /// Creates an owned response header from validated name and value storage.
    pub fn new(name: String, value: Vec<u8>) -> Result<Self, NetworkError> {
        if !name.is_ascii() || !valid_token(&name) || !valid_header_value(&value) {
            return Err(NetworkError::InvalidHeaderOrMethod);
        }
        Ok(Self { name, value })
    }

    /// Borrows the owned header name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Borrows the owned opaque field-value bytes.
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Borrows the field value as UTF-8 without a copy.
    pub fn value_utf8(&self) -> Result<&str, NetworkError> {
        core::str::from_utf8(&self.value).map_err(|_| NetworkError::InvalidUtf8)
    }
}

/// An owned HTTP response; the backend transfers its header/body vectors to the caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    status: StatusCode,
    headers: Vec<ResponseHeader>,
    body: Vec<u8>,
}

impl HttpResponse {
    /// Creates a response by taking ownership of its headers and body.
    pub const fn new(status: StatusCode, headers: Vec<ResponseHeader>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// Returns the HTTP status code; non-success status values remain ordinary responses.
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Borrows response headers in backend order, including duplicates.
    pub fn headers(&self) -> &[ResponseHeader] {
        &self.headers
    }

    /// Borrows the owned response body bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Borrows the body as UTF-8 without a copy.
    pub fn body_utf8(&self) -> Result<&str, NetworkError> {
        core::str::from_utf8(&self.body).map_err(|_| NetworkError::InvalidUtf8)
    }

    /// Transfers the body allocation to the caller.
    pub fn into_body(self) -> Vec<u8> {
        self.body
    }

    /// Transfers the body allocation into a UTF-8 string without a second body copy.
    pub fn into_body_string(self) -> Result<String, NetworkError> {
        String::from_utf8(self.body).map_err(|_| NetworkError::InvalidUtf8)
    }
}

/// A statically selected backend for foreground HTTP requests.
///
/// The facade calls this method when its returned future is first polled. The backend owns start,
/// callback/thread, reentrancy, and exactly-once completion details. Dropping the future only drops
/// Rust interest; cancellation versus detachment of a platform request is backend-defined and must
/// be documented. No executor, retry, cookie jar, or background-transfer behavior is supplied.
pub trait HttpBackend {
    /// Reports whether foreground HTTP is usable in the current context.
    fn availability(&self) -> Availability;

    /// The concrete future type returned by this backend, selected without trait-object boxing.
    type SendFuture<'a>: Future<Output = Result<HttpResponse, NetworkError>> + 'a
    where
        Self: 'a;

    /// Starts or attaches to one foreground request when the future is first polled.
    fn send<'a>(&'a mut self, request: HttpRequest<'a>) -> Self::SendFuture<'a>;
}

/// A thin HTTP client over a caller-owned, statically selected backend.
pub struct HttpClient<B> {
    backend: B,
}

impl<B: HttpBackend> HttpClient<B> {
    /// Creates a client around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without performing a global lookup.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Sends one foreground request using the backend's future and executor-neutral semantics.
    pub async fn send<'a>(
        &'a mut self,
        request: HttpRequest<'a>,
    ) -> Result<HttpResponse, NetworkError> {
        self.backend.send(request).await
    }

    /// Borrows the backend for transport options not modeled by this facade.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for transport options not modeled by this facade.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use core::future::{Ready, ready};
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};

    struct Backend;

    impl HttpBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        type SendFuture<'a>
            = Ready<Result<HttpResponse, NetworkError>>
        where
            Self: 'a;

        fn send<'a>(&'a mut self, request: HttpRequest<'a>) -> Self::SendFuture<'a> {
            let _ = request;
            ready(Ok(HttpResponse::new(
                StatusCode::new(200).unwrap(),
                vec![],
                vec![b'o', b'k'],
            )))
        }
    }

    #[test]
    fn request_borrows_validated_values_without_rewriting_them() {
        let method = HttpMethod::new("POST").unwrap();
        let url = HttpUrl::new("https://example.test/v1").unwrap();
        let header = Header::new("X-Mode", b"fast").unwrap();
        let headers = [header];
        let request = HttpRequest::new(method, url, &headers, Some(b"body"));
        assert_eq!(request.method().as_str(), "POST");
        assert_eq!(request.url().as_str(), "https://example.test/v1");
        assert_eq!(request.headers()[0].value(), b"fast");
        assert_eq!(request.body(), Some(&b"body"[..]));
    }

    #[test]
    fn url_method_header_and_status_inputs_are_bounded() {
        assert_eq!(
            HttpUrl::new("file:///tmp/a").err(),
            Some(NetworkError::InvalidUrl)
        );
        assert_eq!(
            HttpUrl::new("https:///missing-host").err(),
            Some(NetworkError::InvalidUrl)
        );
        assert!(HttpUrl::new("HTTPS://example.test/").is_ok());
        assert_eq!(
            HttpMethod::new("BAD METHOD").err(),
            Some(NetworkError::InvalidHeaderOrMethod)
        );
        assert_eq!(
            Header::new("bad name", b"v").err(),
            Some(NetworkError::InvalidHeaderOrMethod)
        );
        assert_eq!(StatusCode::new(99), None);
        assert_eq!(StatusCode::new(600), None);
    }

    #[test]
    fn future_backend_returns_owned_response_without_an_executor() {
        let method = HttpMethod::new("GET").unwrap();
        let url = HttpUrl::new("https://example.test/").unwrap();
        let headers: [Header<'_>; 0] = [];
        let request = HttpRequest::new(method, url, &headers, None);
        let mut client = HttpClient::new(Backend);
        let mut future = pin!(client.send(request));
        let mut context = Context::from_waker(Waker::noop());
        match future.as_mut().poll(&mut context) {
            Poll::Ready(Ok(response)) => {
                assert_eq!(response.status().get(), 200);
                assert_eq!(response.body_utf8(), Ok("ok"));
            }
            _ => panic!("ready test backend did not complete"),
        }
    }
}

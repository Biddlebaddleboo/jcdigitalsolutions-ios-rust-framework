use alloc::string::String;
use alloc::vec::Vec;
use framework_core::{Error, ErrorKind, PlatformErrorCode};
use framework_network::{
    Header, HttpRequest, HttpResponse, NetworkError, ResponseHeader, StatusCode,
};

extern crate alloc;

/// An owned copy of portable request inputs for the escaping native operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnedRequest {
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) headers: Vec<OwnedHeader>,
    pub(crate) body: Option<Vec<u8>>,
}

/// One owned request header, retained in caller order including duplicates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnedHeader {
    pub(crate) name: String,
    pub(crate) value: String,
}

impl OwnedRequest {
    /// Copies borrowed request values and rejects bytes Foundation cannot represent unchanged.
    pub(crate) fn from_request(request: HttpRequest<'_>) -> Result<Self, NetworkError> {
        let mut headers = Vec::with_capacity(request.headers().len());
        for header in request.headers() {
            headers.push(copy_header(*header)?);
        }
        Ok(Self {
            method: request.method().as_str().into(),
            url: request.url().as_str().into(),
            headers,
            body: request.body().map(<[u8]>::to_vec),
        })
    }
}

fn copy_header(header: Header<'_>) -> Result<OwnedHeader, NetworkError> {
    if !header.value().is_ascii() || reserved_header(header.name()) {
        return Err(NetworkError::InvalidHeaderOrMethod);
    }
    let value = core::str::from_utf8(header.value())
        .map_err(|_| NetworkError::InvalidHeaderOrMethod)?
        .into();
    Ok(OwnedHeader {
        name: header.name().into(),
        value,
    })
}

fn reserved_header(name: &str) -> bool {
    [
        "Content-Length",
        "Authorization",
        "Connection",
        "Host",
        "Proxy-Authenticate",
        "Proxy-Authorization",
        "WWW-Authenticate",
    ]
    .iter()
    .any(|reserved| name.eq_ignore_ascii_case(reserved))
}

/// Converts an HTTP status and ordered native header entries into the portable response.
pub(crate) fn response_from_parts(
    status: i64,
    headers: impl IntoIterator<Item = (String, String)>,
    body: Vec<u8>,
) -> Result<HttpResponse, NetworkError> {
    let status = u16::try_from(status)
        .ok()
        .and_then(StatusCode::new)
        .ok_or(NetworkError::InvalidStatusCode)?;
    let headers = headers
        .into_iter()
        .map(|(name, value)| {
            ResponseHeader::new(name, value.into_bytes())
                .map_err(|_| NetworkError::Backend(Error::new(ErrorKind::Platform)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HttpResponse::new(status, headers, body))
}

/// Stable category chosen from the native NSError domain and public URL-loading error code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeErrorClass {
    /// URL loading reported its documented cancellation code.
    Cancelled,
    /// URL loading reported its documented timeout code.
    Timeout,
    /// URL loading reported a connectivity failure.
    Unavailable,
    /// The error domain/code has no more specific portable category.
    Other,
}

/// Maps a native error class and representable signed code to the portable backend error.
pub(crate) fn native_error(class: NativeErrorClass, code: i64) -> NetworkError {
    let kind = match class {
        NativeErrorClass::Cancelled => ErrorKind::Cancelled,
        NativeErrorClass::Timeout => ErrorKind::Timeout,
        NativeErrorClass::Unavailable => ErrorKind::Unavailable,
        NativeErrorClass::Other => ErrorKind::Platform,
    };
    let error = Error::new(kind);
    let Ok(code) = i32::try_from(code) else {
        return NetworkError::Backend(error);
    };
    match PlatformErrorCode::new(code) {
        Some(code) => NetworkError::Backend(error.with_platform_code(code)),
        None => NetworkError::Backend(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use framework_network::{Header, HttpMethod, HttpUrl};

    #[test]
    fn owned_request_copies_bytes_and_preserves_duplicate_header_order() {
        let method = HttpMethod::new("POST").unwrap();
        let url = HttpUrl::new("https://example.test/path").unwrap();
        let headers = [
            Header::new("X-Trace", b"one").unwrap(),
            Header::new("X-Trace", b"two").unwrap(),
        ];
        let request = HttpRequest::new(method, url, &headers, Some(b"\0body\xff"));
        let owned = OwnedRequest::from_request(request).unwrap();
        assert_eq!(owned.method, "POST");
        assert_eq!(owned.url, "https://example.test/path");
        assert_eq!(owned.headers[0].value, "one");
        assert_eq!(owned.headers[1].value, "two");
        assert_eq!(owned.body, Some(b"\0body\xff".to_vec()));
    }

    #[test]
    fn request_rejects_non_ascii_header_values_at_foundation_boundary() {
        let method = HttpMethod::new("GET").unwrap();
        let url = HttpUrl::new("https://example.test/").unwrap();
        let headers = [Header::new("X-Value", &[0x80]).unwrap()];
        let request = HttpRequest::new(method, url, &headers, None);
        assert_eq!(
            OwnedRequest::from_request(request),
            Err(NetworkError::InvalidHeaderOrMethod)
        );
    }

    #[test]
    fn request_rejects_url_loading_system_reserved_headers() {
        let method = HttpMethod::new("GET").unwrap();
        let url = HttpUrl::new("https://example.test/").unwrap();
        for name in ["content-length", "Authorization", "hOsT"] {
            let headers = [Header::new(name, b"value").unwrap()];
            let request = HttpRequest::new(method, url, &headers, None);
            assert_eq!(
                OwnedRequest::from_request(request),
                Err(NetworkError::InvalidHeaderOrMethod)
            );
        }
    }

    #[test]
    fn response_conversion_keeps_status_body_and_supplied_header_order() {
        let response = response_from_parts(
            503,
            vec![
                ("X-Second".into(), "two".into()),
                ("X-First".into(), "one".into()),
                ("X-Second".into(), "again".into()),
            ],
            b"unavailable".to_vec(),
        )
        .unwrap();
        assert_eq!(response.status().get(), 503);
        assert_eq!(response.headers()[0].name(), "X-Second");
        assert_eq!(response.headers()[1].name(), "X-First");
        assert_eq!(response.headers()[2].value(), b"again");
        assert_eq!(response.body(), b"unavailable");
    }

    #[test]
    fn response_conversion_rejects_invalid_status_and_native_header_values() {
        assert_eq!(
            response_from_parts(600, Vec::new(), Vec::new()).err(),
            Some(NetworkError::InvalidStatusCode)
        );
        assert_eq!(
            response_from_parts(200, vec![("bad name".into(), "value".into())], Vec::new()).err(),
            Some(NetworkError::Backend(Error::new(ErrorKind::Platform)))
        );
    }

    #[test]
    fn native_error_mapping_preserves_code_and_classifies_url_errors() {
        let cancelled = native_error(NativeErrorClass::Cancelled, -999);
        assert_eq!(cancelled.kind(), ErrorKind::Cancelled);
        assert_eq!(cancelled.platform_code().unwrap().get(), -999);
        let timeout = native_error(NativeErrorClass::Timeout, -1001);
        assert_eq!(timeout.kind(), ErrorKind::Timeout);
        assert_eq!(timeout.platform_code().unwrap().get(), -1001);
        assert_eq!(
            native_error(NativeErrorClass::Unavailable, -1009).kind(),
            ErrorKind::Unavailable
        );
        assert_eq!(
            native_error(NativeErrorClass::Other, 17).kind(),
            ErrorKind::Platform
        );
    }

    #[test]
    fn unrepresentable_native_error_code_keeps_category_without_truncation() {
        let error = native_error(NativeErrorClass::Other, i64::MAX);
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code(), None);
    }
}

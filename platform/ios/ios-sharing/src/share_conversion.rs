use alloc::string::String;
use alloc::vec::Vec;
use framework_core::{Error, ErrorKind};
use framework_sharing::{ShareError, ShareItem, ShareRequest};

pub(crate) fn map_request<T>(
    request: ShareRequest,
    mut text: impl FnMut(String) -> T,
    mut url: impl FnMut(String) -> Result<T, ShareError>,
) -> Result<Vec<T>, ShareError> {
    let mut native_items = Vec::with_capacity(request.items().len());
    for item in request.into_items() {
        native_items.push(match item {
            ShareItem::Text(value) => text(value),
            ShareItem::Url(value) => url(value)?,
        });
    }
    if native_items.is_empty() {
        return Err(ShareError::Backend(Error::new(ErrorKind::InvalidInput)));
    }
    Ok(native_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use framework_core::ErrorKind;

    #[derive(Debug, Eq, PartialEq)]
    enum FakeNativeItem {
        Text(String),
        Url(String),
    }

    #[test]
    fn conversion_preserves_item_kind_payload_and_order() {
        let mut request = ShareRequest::new(ShareItem::text("hello".to_string()));
        request.push(ShareItem::url("https://example.test/a".to_string()));
        request.push(ShareItem::text("tail".to_string()));
        let items = map_request(
            request,
            |value| FakeNativeItem::Text(value),
            |value| Ok(FakeNativeItem::Url(value)),
        )
        .unwrap();
        assert_eq!(
            items,
            [
                FakeNativeItem::Text("hello".to_string()),
                FakeNativeItem::Url("https://example.test/a".to_string()),
                FakeNativeItem::Text("tail".to_string()),
            ]
        );
    }

    #[test]
    fn conversion_reports_url_parser_rejection_as_invalid_input() {
        let request = ShareRequest::new(ShareItem::url("bad[".to_string()));
        let error = map_request(request, FakeNativeItem::Text, |value| {
            if value.contains('[') {
                Err(ShareError::Backend(Error::new(ErrorKind::InvalidInput)))
            } else {
                Ok(FakeNativeItem::Url(value))
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert_eq!(error.platform_code(), None);
    }
}

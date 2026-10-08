# Portable URI values

`framework-format` provides borrowed, validated RFC 3986 `Uri` and `UriReference` values. `Uri`
requires a scheme; `UriReference` accepts either an absolute URI or a relative reference.

```rust
use framework_format::{Uri, UriReference};

fn main() -> Result<(), framework_core::Error> {
    let uri = Uri::new("https://example.com/a%2Fb?mode=raw#part")?;
    assert_eq!(uri.as_str(), "https://example.com/a%2Fb?mode=raw#part");
    assert_eq!(uri.scheme(), "https");
    assert_eq!(uri.path(), "/a%2Fb");

    let reference = UriReference::new("../image.svg?size=small")?;
    assert_eq!(reference.scheme(), None);
    assert_eq!(reference.path(), "../image.svg");
    Ok(())
}
```

Parsing validates RFC 3986 syntax and reports invalid input as `framework_core::Error` with
`ErrorKind::InvalidInput`. These values borrow the caller's original `&str`; constructing or
reading them does not allocate or alter text. Component access returns borrowed slices that omit
their delimiters: scheme omits `:`, authority omits `//`, query omits `?`, and fragment omits `#`.
Optional components use `None` only when the component is absent; an explicitly empty authority,
query, or fragment is `Some("")`. The path is always present as a slice and may be empty.

The crate is `no_std` and uses `iri-string` 0.7.14 with default features disabled and no optional
features. The dependency's borrowed URI types keep validation and component access allocation-free;
its owned, normalization, resolution, and IRI APIs are not exposed here. The public API does not
leak dependency types and can replace the internal validator without changing the wrappers.

URI syntax alone does not establish scheme-specific validity, web safety, network reachability, or
equivalence after normalization. This URI-only contract excludes unescaped non-ASCII IRI text;
percent-encoded text remains part of the original borrowed URI. These types do not use Foundation
or `NSURL` and do not change `framework-network::HttpUrl`'s HTTP-only semantics.

The portable `framework-format` crate does not dereference URLs, resolve names, or implement network
behavior. B11's separate `ios-browser` backend consumes an absolute HTTPS `Uri` only to issue a
system URL-handler request; it does not guarantee Safari, a page load, or visible browser UI.

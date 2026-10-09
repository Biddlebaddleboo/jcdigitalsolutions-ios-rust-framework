#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_format::Uri;
#[cfg(target_os = "ios")]
use ios_url::NativeUrl;

#[cfg(target_os = "ios")]
fn main() {
    if let Ok(uri) = Uri::new("https://example.com/a%20b?x=1#frag") {
        if let Ok(native) = NativeUrl::new(uri) {
            let _ = core::hint::black_box(native.source_uri().as_str());
            let _ = core::hint::black_box(native.as_ns_url());
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}

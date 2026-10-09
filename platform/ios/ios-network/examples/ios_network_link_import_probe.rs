#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_network::{Header, HttpBackend, HttpMethod, HttpRequest, HttpUrl};
#[cfg(target_os = "ios")]
use ios_network::IosHttpBackend;

#[cfg(target_os = "ios")]
fn main() {
    let mut backend = IosHttpBackend::new();
    let _ = core::hint::black_box(backend.native_session());
    let method = HttpMethod::new("GET").expect("fixed method is valid");
    let url = HttpUrl::new("https://example.invalid/").expect("fixed URL is valid");
    let headers: [Header<'static>; 0] = [];
    let request = HttpRequest::new(method, url, &headers, None);
    let future = backend.send(request);
    // Do not poll: the URLSession task starts on its first poll
    drop(core::hint::black_box(future));
}

#[cfg(not(target_os = "ios"))]
fn main() {}

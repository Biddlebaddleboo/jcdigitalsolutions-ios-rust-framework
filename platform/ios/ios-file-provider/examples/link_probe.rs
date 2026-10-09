#[cfg(target_os = "ios")]
fn main() {
    let _ = std::hint::black_box(ios_file_provider::request_registered_domain_presence());
    let _ = std::hint::black_box(ios_file_provider::request_registered_domain_count());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

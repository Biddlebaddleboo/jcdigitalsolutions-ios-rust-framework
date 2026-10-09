#[cfg(target_os = "ios")]
fn main() {
    let _ = std::hint::black_box(ios_file_provider::request_registered_domain_presence());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

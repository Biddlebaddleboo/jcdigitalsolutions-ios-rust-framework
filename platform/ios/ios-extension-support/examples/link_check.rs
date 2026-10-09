#[cfg(target_os = "ios")]
fn main() {
    let _ = ios_extension_support::read_extension_point_identifier(
        "/tmp/extension-support-link-check.appex",
    );
}

#[cfg(not(target_os = "ios"))]
fn main() {}

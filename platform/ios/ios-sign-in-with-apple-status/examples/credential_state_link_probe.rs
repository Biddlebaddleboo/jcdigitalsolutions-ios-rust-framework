#[cfg(target_os = "ios")]
use ios_sign_in_with_apple_status::get_credential_state;

#[cfg(target_os = "ios")]
fn main() {
    get_credential_state("compile-only-probe-id", |_| {});
}

#[cfg(not(target_os = "ios"))]
fn main() {}

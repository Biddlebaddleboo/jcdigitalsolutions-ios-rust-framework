#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    // SAFETY: this link-only example is not run; a live host must call from its main dispatch queue
    let _ = core::hint::black_box(unsafe {
        ios_family_controls_status::authorization_status_raw_value_on_main_queue()
    });
}

#[cfg(not(target_os = "ios"))]
fn main() {}

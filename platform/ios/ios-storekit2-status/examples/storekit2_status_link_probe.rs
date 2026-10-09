#[cfg(target_os = "ios")]
fn main() {
    std::hint::black_box(ios_storekit2_status::can_make_payments());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

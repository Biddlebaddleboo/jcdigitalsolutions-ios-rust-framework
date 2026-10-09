#[cfg(target_os = "ios")]
fn main() {
    let _ = std::hint::black_box(ios_preferences::IosPreferences::new());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

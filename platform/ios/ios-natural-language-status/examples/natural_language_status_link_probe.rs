#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    core::hint::black_box(ios_natural_language_status::english_contextual_embedding_assets());
}

#[cfg(not(target_os = "ios"))]
fn main() {}

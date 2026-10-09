#[cfg(target_os = "ios")]
fn main() {
    let input = std::hint::black_box(b"CommonCrypto link probe");
    let digest = ios_crypto::sha256(input);
    let _ = std::hint::black_box(digest);
}

#[cfg(not(target_os = "ios"))]
fn main() {}

#[cfg(target_os = "ios")]
fn main() {
    let bytes = [0x04; 65];
    let key = framework_key_support::P256PublicKey::from_x963_uncompressed(&bytes)
        .expect("the probe uses the X9.63 uncompressed marker");
    let result = ios_key_support::IosP256VerificationSupport::new().query(key);
    let _ = core::hint::black_box(result);
}

#[cfg(not(target_os = "ios"))]
fn main() {}

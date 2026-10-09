fn main() {
    println!("cargo:rerun-if-changed=src/storekit2_bridge.c");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        cc::Build::new()
            .file("src/storekit2_bridge.c")
            .warnings(true)
            .extra_warnings(true)
            .flag_if_supported("-Werror")
            .compile("ios_storekit2_status_bridge");
    }
}

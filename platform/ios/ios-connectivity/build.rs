use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/network_shim.c");
    println!("cargo:rerun-if-changed=src/network_shim.h");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("ios") {
        return;
    }

    cc::Build::new()
        .file("src/network_shim.c")
        .include("src")
        .flag("-fblocks")
        .flag_if_supported("-std=c11")
        .compile("ios_connectivity_network_shim");

    println!("cargo:rustc-link-lib=framework=Network");
}

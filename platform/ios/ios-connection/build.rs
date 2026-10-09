use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/network_connection_shim.c");
    println!("cargo:rerun-if-changed=src/network_connection_shim.h");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("ios") {
        return;
    }

    cc::Build::new()
        .file("src/network_connection_shim.c")
        .include("src")
        .flag("-fblocks")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Werror")
        .compile("ios_connection_network_shim");

    println!("cargo:rustc-link-lib=framework=Network");
}

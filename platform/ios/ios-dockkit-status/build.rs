use std::env;
use std::path::PathBuf;
use std::process::Command;

fn run(mut command: Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to run {description}: {error}"));
    assert!(status.success(), "{description} failed with {status}");
}

fn output(mut command: Command, description: &str) -> String {
    let result = command
        .output()
        .unwrap_or_else(|error| panic!("failed to run {description}: {error}"));
    assert!(
        result.status.success(),
        "{description} failed with {}: {}",
        result.status,
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout)
        .unwrap_or_else(|error| panic!("{description} returned non-UTF-8 output: {error}"))
        .trim()
        .to_owned()
}

fn main() {
    println!("cargo:rerun-if-changed=native/dockkit_status.c");
    println!("cargo:rerun-if-env-changed=SDKROOT");
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("ios") {
        return;
    }

    let target = env::var("TARGET").expect("Cargo must set TARGET");
    if target == "aarch64-apple-ios-sim" {
        println!(
            "cargo:warning=DockKit is absent from the installed Simulator SDK; simulator backend is unavailable"
        );
        return;
    }
    if target != "aarch64-apple-ios" {
        panic!("unsupported iOS DockKit target: {target}");
    }

    let sdk = "iphoneos";
    let clang_target = "arm64-apple-ios17.0";
    let sdk_path = {
        let mut command = Command::new("xcrun");
        command.args(["--sdk", sdk, "--show-sdk-path"]);
        output(command, "xcrun SDK lookup")
    };
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    let object = out_dir.join("dockkit_status.o");
    let archive = out_dir.join("libdockkit_status.a");

    let mut clang = Command::new("xcrun");
    clang.args([
        "--sdk",
        sdk,
        "clang",
        "-target",
        clang_target,
        "-isysroot",
        &sdk_path,
        "-std=c11",
        "-Wall",
        "-Wextra",
        "-Werror",
        "-c",
        "native/dockkit_status.c",
        "-o",
    ]);
    clang.arg(&object);
    run(clang, "DockKit C swiftcall bridge compile");

    let mut ar = Command::new("xcrun");
    ar.args(["--sdk", sdk, "ar", "crs"]);
    ar.arg(&archive).arg(&object);
    run(ar, "DockKit static archive creation");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dockkit_status");
    println!("cargo:rustc-link-lib=framework=DockKit");
}

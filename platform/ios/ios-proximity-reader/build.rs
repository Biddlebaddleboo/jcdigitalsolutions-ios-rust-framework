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
    println!("cargo:rerun-if-changed=native/proximity_reader_support.c");
    println!("cargo:rerun-if-env-changed=SDKROOT");
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("ios") {
        return;
    }

    let target = env::var("TARGET").expect("Cargo must set TARGET");
    let (sdk, clang_target) = match target.as_str() {
        "aarch64-apple-ios" => ("iphoneos", "arm64-apple-ios15.4"),
        "aarch64-apple-ios-sim" => ("iphonesimulator", "arm64-apple-ios15.4-simulator"),
        _ => panic!("unsupported iOS ProximityReader target: {target}"),
    };
    let sdk_path = {
        let mut command = Command::new("xcrun");
        command.args(["--sdk", sdk, "--show-sdk-path"]);
        output(command, "xcrun SDK lookup")
    };
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    let object = out_dir.join("proximity_reader_support.o");
    let archive = out_dir.join("libproximity_reader_support.a");

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
        "native/proximity_reader_support.c",
        "-o",
    ]);
    clang.arg(&object);
    run(clang, "ProximityReader C swiftcall thunk compile");

    let mut ar = Command::new("xcrun");
    ar.args(["--sdk", sdk, "ar", "crs"]);
    ar.arg(&archive).arg(&object);
    run(ar, "ProximityReader static archive creation");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=proximity_reader_support");
    println!("cargo:rustc-link-lib=framework=ProximityReader");
}

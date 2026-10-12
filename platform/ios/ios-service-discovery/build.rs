use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const EXPECTED_TOOL_VERSION: &str = "0.1.0";

fn main() {
    if let Err(error) = run() {
        panic!("ios-rust-build bridge failed: {error}");
    }
}

fn run() -> Result<(), String> {
    println!("cargo:rerun-if-changed=build-spec.json");
    println!("cargo:rerun-if-env-changed=PATH");
    let manifest = PathBuf::from(required_env("CARGO_MANIFEST_DIR")?);
    let host = required_env("HOST")?;
    let root = manifest
        .join("../../..")
        .canonicalize()
        .map_err(|error| format!("cannot locate workspace root: {error}"))?;
    let spec = manifest.join("build-spec.json");
    let version = Command::new("ios-rust-build")
        .args(["--version", "--format", "json"])
        .output()
        .map_err(|error| {
            format!(
                "cannot run ios-rust-build from PATH ({error}); install the pinned build tools with tools/install-tools.sh"
            )
        })?;
    if !version.status.success() {
        return Err(format!(
            "ios-rust-build --version failed: {}",
            String::from_utf8_lossy(&version.stderr).trim()
        ));
    }
    let version: Value = serde_json::from_slice(&version.stdout)
        .map_err(|error| format!("ios-rust-build returned invalid version JSON: {error}"))?;
    if version.get("tool").and_then(Value::as_str) != Some("ios-rust-build")
        || version.get("version").and_then(Value::as_str) != Some(EXPECTED_TOOL_VERSION)
        || version.get("schema_major").and_then(Value::as_u64) != Some(1)
        || version.get("host").and_then(Value::as_str) != Some(host.as_str())
    {
        return Err(format!("incompatible ios-rust-build protocol: {version}"));
    }
    let target = required_env("TARGET")?;
    let target_os = required_env("CARGO_CFG_TARGET_OS")?;
    let out_dir = required_env("OUT_DIR")?;
    let output = Command::new("ios-rust-build")
        .arg("build")
        .arg("--workspace-root")
        .arg(root)
        .arg("--spec")
        .arg(spec)
        .arg("--target")
        .arg(target)
        .arg("--target-os")
        .arg(target_os)
        .arg("--out-dir")
        .arg(out_dir)
        .arg("--manifest-dir")
        .arg(manifest)
        .args(["--format", "json"])
        .output()
        .map_err(|error| format!("cannot execute ios-rust-build: {error}"))?;
    if !output.status.success() {
        let message = serde_json::from_slice::<Value>(&output.stdout)
            .ok()
            .and_then(|value| {
                value
                    .get("message")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| String::from_utf8_lossy(&output.stderr).trim().to_owned());
        return Err(format!("ios-rust-build failed: {message}"));
    }
    let result: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("ios-rust-build returned invalid build JSON: {error}"))?;
    if result.get("schema_version").and_then(Value::as_u64) != Some(1)
        || result.get("status").and_then(Value::as_str) != Some("ok")
    {
        return Err(format!("incompatible ios-rust-build result: {result}"));
    }
    let directives = result
        .get("directives")
        .and_then(Value::as_array)
        .ok_or_else(|| "ios-rust-build result has no directives array".to_owned())?;
    for directive in directives {
        let directive = directive
            .as_str()
            .ok_or_else(|| "ios-rust-build emitted a non-string Cargo directive".to_owned())?;
        if directive.contains(['\n', '\r', '\0'])
            || ![
                "cargo:rerun-if-changed=",
                "cargo:rerun-if-env-changed=",
                "cargo:rustc-link-search=native=",
                "cargo:rustc-link-lib=static=",
                "cargo:rustc-link-lib=framework=",
            ]
            .iter()
            .any(|prefix| directive.starts_with(prefix))
        {
            return Err(format!("rejected unsafe Cargo directive: {directive:?}"));
        }
        println!("{directive}");
    }
    Ok(())
}

fn required_env(name: &str) -> Result<String, String> {
    env::var(name)
        .map_err(|error| format!("Cargo environment variable {name} is unavailable: {error}"))
}

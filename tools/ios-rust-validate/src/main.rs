#[path = "../../xtask/src/validation.rs"]
mod validation;

#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Err(error) = verify_runtime_host() {
        eprintln!(
            "{}",
            serde_json::json!({
                "schema_version": 1,
                "status": "error-blocked",
                "code": "host_mismatch",
                "message": error
            })
        );
        std::process::exit(1);
    }
    if args == ["--help"] || args == ["-h"] {
        print!("{}", HELP);
        return;
    }
    if args == ["--version"] {
        println!("ios-rust-validate {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args == ["--version", "--format", "json"] {
        let source_sha = option_env!("TOOL_SOURCE_SHA").unwrap_or("working-tree");
        println!(
            "{}",
            serde_json::json!({
                "tool": "ios-rust-validate",
                "version": env!("CARGO_PKG_VERSION"),
                "schema_major": 1,
                "result_schema": 1,
                "spec_schema": 1,
                "release_id": format!("ios-rust-tools-{}+{}", env!("CARGO_PKG_VERSION"), source_sha),
                "source_sha": source_sha,
                "rustc_version": option_env!("TOOL_RUSTC_VERSION").unwrap_or("unknown"),
                "host": option_env!("HOST").unwrap_or("unknown")
            })
        );
        return;
    }
    if let Err(error) = run(&args) {
        eprintln!(
            "{}",
            serde_json::json!({
                "schema_version": 1,
                "status": "error-blocked",
                "code": "validation_error",
                "message": error
            })
        );
        std::process::exit(1);
    }
}

fn verify_runtime_host() -> Result<(), String> {
    let Some(expected) = option_env!("HOST") else {
        return Ok(());
    };
    let actual = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        _ => "unsupported",
    };
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "ios-rust-validate host mismatch: binary={expected}, runtime={actual}"
        ))
    }
}

const HELP: &str = "ios-rust-validate <options>\n\nOptions:\n  --workspace-root PATH\n  --spec PATH\n  --list | --explain ID | --capability ID | --changed EXPLICIT_BASE | --all\n  --format human|json\n  --help | --version [--format json]\n";

fn run(args: &[String]) -> Result<(), String> {
    let mut workspace_root = None;
    let mut spec = None;
    let mut validation_args = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--workspace-root" | "--spec" => {
                let name = args[index].as_str();
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("`{name}` requires a value"))?;
                let slot = if name == "--workspace-root" {
                    &mut workspace_root
                } else {
                    &mut spec
                };
                if slot.replace(PathBuf::from(value)).is_some() {
                    return Err(format!("duplicate option `{name}`"));
                }
                index += 2;
            }
            _ => {
                validation_args.push(args[index].clone());
                index += 1;
            }
        }
    }
    let workspace_root =
        workspace_root.ok_or_else(|| "missing required option `--workspace-root`".to_owned())?;
    let spec = spec.ok_or_else(|| "missing required option `--spec`".to_owned())?;
    validation::run_at(&validation_args, &workspace_root, &spec)
}

#[cfg(test)]
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("ios-rust-validate must live under tools")
        .to_path_buf()
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("JSON string serialization is infallible")
}

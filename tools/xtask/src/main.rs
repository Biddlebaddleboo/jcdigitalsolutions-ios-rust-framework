mod audits;
mod codegen;
mod no_std_link;
mod sdk_inventory;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PORTABLE_CRATES: &[&str] = &[
    "framework-core",
    "framework-alloc",
    "framework-async",
    "framework-abi",
    "framework-platform",
    "framework-background",
    "framework-background-execution",
    "framework-accessory",
    "framework-contacts",
    "framework-calendar",
    "framework-bluetooth",
    "framework-health-authorization",
    "framework-device-integrity",
    "framework-cloud",
    "framework-game",
    "framework-maps",
    "framework-vision",
    "framework-roomplan",
    "framework-web",
    "framework-nfc",
    "framework-data",
    "framework-app",
    "framework-files",
    "framework-preferences",
    "framework-resources",
    "framework-format",
    "framework-key-support",
    "framework-spritekit",
    "framework-image",
    "framework-network",
    "framework-connection",
    "framework-connectivity",
    "framework-transfer",
    "framework-secure-storage",
    "framework-auth",
    "framework-notifications",
    "framework-location",
    "framework-motion",
    "framework-photos",
    "framework-media",
    "framework-media-authorization",
    "framework-nearby",
    "framework-metal",
    "framework-sharing",
    "framework-audio",
    "framework-ui",
    "framework-watch-connectivity",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };
    let args = args.collect::<Vec<_>>();
    match command.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "toolchain-manifest" => toolchain_manifest(&args),
        "no-std-check" => no_std_check(),
        "no-std-link-probe" => no_std_link::run(&args),
        "sdk-inventory" => sdk_inventory::run(&args),
        "dependency-audit" => audits::dependency_audit(&args),
        "abi-audit" => audits::abi_audit(&args),
        "codegen-audit" => codegen::run(&args),
        "linkage-audit" => audits::linkage_audit(&args),
        "zero-swift-source" => zero_swift_source(),
        "docs-check" => docs_check(),
        "ios-build" => ios_build(&args),
        "archive-smoke" => run_example_script("archive.sh", &args, "archive smoke"),
        "parity" => Err("parity is unavailable until a real Apple reference adapter and Rust candidate suite exist".into()),
        _ => Err(format!("unknown command `{command}`; run `cargo xtask help`")),
    }
}

fn print_help() {
    print!("{}", help_text());
}

fn help_text() -> &'static str {
    "cargo xtask <command>\n\
\n\
Commands:\n\
  help | --help | -h\n\
  toolchain-manifest [--output PATH]       Record host, Xcode, SDK, Swift, Clang, and Rust versions\n\
  ios-build (--simulator | --device) --release  Build the minimal iOS example\n\
  no-std-check                            Check portable crates with default and no default features\n\
  no-std-link-probe [--output PATH]        Link and audit the host and Apple-target no_std probes\n\
  sdk-inventory [--sdk NAME] [--output PATH]  Inventory installed public Apple SDK files\n\
  dependency-audit [--output-dir PATH]     Record Cargo graphs and duplicate versions\n\
  abi-audit [--output PATH]                Inventory current C ABI source declarations\n\
  codegen-audit [--output PATH]             Inspect optimized OperationId LLVM IR for host and installed iOS targets\n\
  linkage-audit --binary PATH              Inspect a Mach-O binary with otool\n\
  zero-swift-source                       Reject .swift files in the checkout except .git and target\n\
  docs-check                              Check shared docs index and zero-Swift-source policy\n\
  archive-smoke                            Build and inspect an unsigned Xcode archive\n\
  parity                                  Unavailable until a real Apple reference adapter and Rust candidate suite exist\n\
\n\
Xcode 27.x is the plan baseline. A toolchain manifest warns when the host does not meet it\n"
}

pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask must live under tools/xtask")
        .to_path_buf()
}

pub(crate) fn output_path(args: &[String], option: &str) -> Result<Option<PathBuf>, String> {
    let mut path = None;
    let mut index = 0;
    while index < args.len() {
        if args[index] == option {
            if path.is_some() {
                return Err(format!("`{option}` may be passed only once"));
            }
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("`{option}` requires a path"))?;
            path = Some(PathBuf::from(value));
        } else {
            return Err(format!("unexpected argument `{}`", args[index]));
        }
        index += 1;
    }
    Ok(path)
}

pub(crate) fn write_report(path: Option<PathBuf>, body: &str) -> Result<(), String> {
    if let Some(path) = path {
        let path = if path.is_absolute() {
            path
        } else {
            root().join(path)
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        fs::write(&path, body).map_err(|error| format!("write {}: {error}", path.display()))?;
        println!("wrote {}", path.display());
    } else {
        print!("{body}");
    }
    Ok(())
}

pub(crate) fn json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

pub(crate) fn json_string_array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn run_capture(program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .current_dir(root())
        .output()
        .map_err(|error| format!("run {program}: {error}"))
}

pub(crate) fn output_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    }
}

fn observation(program: &str, args: &[&str]) -> String {
    let command = format!("{} {}", program, args.join(" "));
    match run_capture(program, args) {
        Ok(output) => {
            let status = output
                .status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".into());
            format!(
                "{{\"command\":{},\"exit_code\":{},\"output\":{}}}",
                json_string(&command),
                status,
                json_string(&output_text(&output))
            )
        }
        Err(error) => format!(
            "{{\"command\":{},\"exit_code\":null,\"error\":{}}}",
            json_string(&command),
            json_string(&error)
        ),
    }
}

fn toolchain_manifest(args: &[String]) -> Result<(), String> {
    let output = output_path(args, "--output")?;
    let created_at = run_capture("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .ok()
        .filter(|output| output.status.success())
        .map(|output| output_text(&output));
    let xcode = run_capture("xcodebuild", &["-version"])
        .ok()
        .filter(|output| output.status.success())
        .map(|output| output_text(&output));
    let xcode_version = xcode.as_deref().and_then(|value| {
        value
            .lines()
            .next()?
            .strip_prefix("Xcode ")
            .map(str::to_owned)
    });
    let plan_xcode_compliance = match xcode_version.as_deref() {
        Some(version) if version.starts_with("27.") => "met",
        Some(_) => "not_met",
        None => "unknown",
    };
    let deployment_target = env::var("IPHONEOS_DEPLOYMENT_TARGET").ok();
    let sdks = ["iphoneos", "iphonesimulator"]
        .iter()
        .map(|sdk| {
            format!(
                "{{\"name\":{},\"path\":{},\"version\":{}}}",
                json_string(sdk),
                observation("xcrun", &["--sdk", sdk, "--show-sdk-path"]),
                observation("xcrun", &["--sdk", sdk, "--show-sdk-version"])
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let installed_targets = observation("rustup", &["target", "list", "--installed"]);
    let macos_version = observation("sw_vers", &["-productVersion"]);
    let macos_build = observation("sw_vers", &["-buildVersion"]);
    let ios_targets = ["aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios"]
        .iter()
        .map(|triple| {
            format!(
                "{{\"triple\":{},\"deployment_target_env\":\"IPHONEOS_DEPLOYMENT_TARGET\",\"deployment_target\":{}}}",
                json_string(triple),
                deployment_target
                    .as_deref()
                    .map(json_string)
                    .unwrap_or_else(|| "null".into())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let manifest = format!(
        "{{\n  \"schema_version\": 1,\n  \"captured_at_utc\": {},\n  \"host\": {{\"os\": {}, \"architecture\": {}, \"macos_version\": {}, \"macos_build\": {}}},\n  \"plan_xcode_requirement\": \"27.x\",\n  \"plan_xcode_compliance\": {},\n  \"xcode_select_path\": {},\n  \"xcodebuild_version\": {},\n  \"swift_compiler\": {},\n  \"clang_llvm\": {},\n  \"rustc_verbose\": {},\n  \"rustup_installed_targets\": {},\n  \"ios_targets\": [{}],\n  \"apple_sdks\": [{}]\n}}\n",
        created_at
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        json_string(env::consts::OS),
        json_string(env::consts::ARCH),
        macos_version,
        macos_build,
        json_string(plan_xcode_compliance),
        observation("xcode-select", &["-p"]),
        xcode
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        observation("swift", &["--version"]),
        observation("clang", &["--version"]),
        observation("rustc", &["-Vv"]),
        installed_targets,
        ios_targets,
        sdks
    );
    write_report(output, &manifest)?;
    if plan_xcode_compliance != "met" {
        let message = format!(
            "Xcode plan requirement 27.x is {plan_xcode_compliance}; observed {}",
            xcode_version.as_deref().unwrap_or("no Xcode version")
        );
        if env::var_os("GITHUB_ACTIONS").is_some() {
            eprintln!("::warning::{message}");
        } else {
            eprintln!("warning: {message}");
        }
    }
    Ok(())
}

fn no_std_check() -> Result<(), String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    for package in PORTABLE_CRATES {
        check_portable_normal_dependencies(&cargo, package)?;
        let source = root().join("crates").join(package).join("src/lib.rs");
        let contents = fs::read_to_string(&source)
            .map_err(|error| format!("read {}: {error}", source.display()))?;
        if !contents.lines().any(|line| line.trim() == "#![no_std]") {
            return Err(format!(
                "{package} does not declare `#![no_std]` in src/lib.rs"
            ));
        }
        for features in ["default features", "--no-default-features"] {
            let mut command = Command::new(&cargo);
            command
                .current_dir(root())
                .args(["check", "--locked", "-p", package]);
            if features == "--no-default-features" {
                command.arg("--no-default-features");
            }
            println!("cargo check -p {package} {features}");
            let status = command
                .status()
                .map_err(|error| format!("run cargo check for {package}: {error}"))?;
            if !status.success() {
                return Err(format!("no_std check failed for {package} ({features})"));
            }
        }
    }
    Ok(())
}

fn check_portable_normal_dependencies(cargo: &str, package: &str) -> Result<(), String> {
    let platform_root = root()
        .join("platform")
        .canonicalize()
        .map_err(|error| format!("resolve platform workspace path: {error}"))?;
    let normal_tree = portable_dependency_tree(cargo, package, "normal")?;
    let dev_tree = portable_dependency_tree(cargo, package, "dev")?;
    let build_tree = portable_dependency_tree(cargo, package, "build")?;
    let platform_packages = normal_tree
        .iter()
        .filter_map(|(_, package_id)| {
            let (_, source) = package_id.split_once(" (")?;
            let source = source.strip_suffix(')')?;
            if Path::new(source).starts_with(&platform_root) {
                package_id.split_whitespace().next().map(str::to_owned)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if !platform_packages.is_empty() {
        return Err(format!(
            "{package} normal dependency tree includes internal platform package(s): {}",
            platform_packages.join(", ")
        ));
    }
    let direct_dependencies = |tree: &[(usize, String)]| {
        tree.iter()
            .filter(|(depth, _)| *depth == 1)
            .map(|(_, package_id)| package_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let normal_direct = direct_dependencies(&normal_tree);
    let dev_direct = direct_dependencies(&dev_tree);
    let build_direct = direct_dependencies(&build_tree);
    let duplicated_kinds = normal_direct
        .intersection(&dev_direct)
        .chain(normal_direct.intersection(&build_direct))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    if !duplicated_kinds.is_empty() {
        return Err(format!(
            "{package} portable dependencies occur as both normal and dev/build edges: {}",
            duplicated_kinds.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    println!(
        "cargo tree --locked --all-features --target all -p {package}: normal={}, dev={}, build={}; no normal/dev/build edge overlap or internal platform package in the normal tree",
        normal_direct.len(),
        dev_direct.len(),
        build_direct.len()
    );
    Ok(())
}

fn portable_dependency_tree(
    cargo: &str,
    package: &str,
    edge: &str,
) -> Result<Vec<(usize, String)>, String> {
    let args = [
        "tree",
        "--locked",
        "--all-features",
        "--target",
        "all",
        "--edges",
        edge,
        "--no-dedupe",
        "--prefix",
        "depth",
        "--format",
        "{p}",
        "-p",
        package,
    ];
    let output = Command::new(cargo)
        .current_dir(root())
        .args(args)
        .output()
        .map_err(|error| format!("run cargo tree for {package} ({edge} edges): {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo tree for {package} ({edge} edges) failed: {}",
            output_text(&output)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let depth_length = line.bytes().take_while(u8::is_ascii_digit).count();
        if depth_length == 0 {
            return Err(format!(
                "unrecognized cargo tree ({edge} edges) output for {package}: {line}"
            ));
        }
        let depth = line[..depth_length]
            .parse::<usize>()
            .map_err(|error| format!("parse cargo tree depth for {package}: {error}"))?;
        let package_id = line[depth_length..].trim();
        if package_id.is_empty() {
            return Err(format!(
                "missing package ID in cargo tree ({edge} edges) output for {package}"
            ));
        }
        entries.push((depth, package_id.to_owned()));
    }
    if !entries.first().is_some_and(|(depth, package_id)| {
        *depth == 0 && package_id.split_whitespace().next() == Some(package)
    }) {
        return Err(format!(
            "cargo tree ({edge} edges) output did not start with package {package}"
        ));
    }
    Ok(entries)
}

fn zero_swift_source() -> Result<(), String> {
    let mut swift_files = Vec::new();
    collect_swift_files(&root(), &root(), &mut swift_files)?;
    if swift_files.is_empty() {
        println!(
            "zero-Swift-source check passed; oracle input must remain transient and outside the checkout"
        );
        return Ok(());
    }
    Err(format!(
        ".swift source found in the checkout; oracle input must remain transient and outside the checkout: {}",
        swift_files.join(", ")
    ))
}

fn collect_swift_files(root: &Path, path: &Path, files: &mut Vec<String>) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(|error| format!("read {}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("read directory entry: {error}"))?;
        let entry_path = entry.path();
        let relative = entry_path
            .strip_prefix(root)
            .map_err(|error| format!("relative path {}: {error}", entry_path.display()))?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            let relative_text = relative.to_string_lossy();
            if relative_text == ".git" || relative_text == "target" {
                continue;
            }
            collect_swift_files(root, &entry_path, files)?;
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "swift")
        {
            files.push(relative.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn ios_build(args: &[String]) -> Result<(), String> {
    let target = parse_ios_build_args(args)?;
    run_example_script("build.sh", &[target.to_owned()], "iOS build")
}

fn parse_ios_build_args(args: &[String]) -> Result<&'static str, String> {
    const USAGE: &str = "usage: cargo xtask ios-build (--simulator | --device) --release";
    let mut target = None;
    let mut release = false;
    for arg in args {
        match arg.as_str() {
            "--simulator" if target.is_none() => target = Some("simulator"),
            "--device" if target.is_none() => target = Some("device"),
            "--release" if !release => release = true,
            "--simulator" | "--device" => return Err(USAGE.into()),
            "--release" => return Err("`--release` may be passed only once".into()),
            other => return Err(format!("unknown iOS build argument `{other}`; {USAGE}")),
        }
    }
    let target = target.ok_or(USAGE)?;
    if !release {
        return Err(format!("`--release` is required; {USAGE}"));
    }
    Ok(target)
}

fn run_example_script(script_name: &str, args: &[String], label: &str) -> Result<(), String> {
    let script = root().join("examples/ios-minimal").join(script_name);
    if !script.is_file() {
        return Err(format!(
            "{label} is not available: examples/ios-minimal/{script_name} does not exist; no command was attempted"
        ));
    }
    let status = Command::new("sh")
        .arg(&script)
        .args(args)
        .current_dir(root())
        .status()
        .map_err(|error| format!("run {}: {error}", script.display()))?;
    if !status.success() {
        return Err(format!("{label} script failed with status {status}"));
    }
    Ok(())
}

fn docs_check() -> Result<(), String> {
    let required = [
        "docs/VALIDATION.md",
        "docs/DOCUMENTATION.md",
        "docs/IOS_BUILD.md",
    ];
    for relative in required {
        if !root().join(relative).is_file() {
            return Err(format!("required shared document is missing: {relative}"));
        }
    }
    let index = fs::read_to_string(root().join("docs/DOCUMENTATION.md"))
        .map_err(|error| format!("read docs/DOCUMENTATION.md: {error}"))?;
    if !index.contains("VALIDATION.md") {
        return Err("docs/DOCUMENTATION.md does not index VALIDATION.md".into());
    }
    zero_swift_source()?;
    println!("shared documentation index check passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{collect_swift_files, help_text, json_string, parse_ios_build_args};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("xtask-test-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn json_escape_handles_quotes_slashes_and_control_chars() {
        assert_eq!(
            json_string("quote \" slash \\ line\n tab\t"),
            "\"quote \\\" slash \\\\ line\\n tab\\t\""
        );
    }

    #[test]
    fn swift_scan_includes_oracle_and_skips_build_metadata() {
        let root = temp_root();
        for file in [
            "Sources/Auth.swift",
            "tools/swift-oracle/Input.swift",
            "target/Generated.swift",
            ".git/Hook.swift",
        ] {
            let path = root.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "// input").unwrap();
        }
        let mut found = Vec::new();
        collect_swift_files(&root, &root, &mut found).unwrap();
        found.sort();
        assert_eq!(
            found,
            ["Sources/Auth.swift", "tools/swift-oracle/Input.swift"].map(str::to_owned)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ios_build_flags_map_to_the_example_script_target() {
        assert_eq!(
            parse_ios_build_args(&["--simulator".into(), "--release".into()]).unwrap(),
            "simulator"
        );
        assert_eq!(
            parse_ios_build_args(&["--device".into(), "--release".into()]).unwrap(),
            "device"
        );
    }

    #[test]
    fn ios_build_rejects_missing_both_duplicate_and_unknown_flags() {
        assert!(parse_ios_build_args(&[]).is_err());
        assert!(parse_ios_build_args(&["--release".into()]).is_err());
        assert!(
            parse_ios_build_args(&["--simulator".into(), "--device".into(), "--release".into()])
                .is_err()
        );
        assert!(
            parse_ios_build_args(&["--simulator".into(), "--release".into(), "--debug".into()])
                .is_err()
        );
        assert!(parse_ios_build_args(&["--simulator".into()]).is_err());
    }

    #[test]
    fn help_lists_each_supported_command() {
        let help = help_text();
        for command in [
            "help | --help | -h",
            "toolchain-manifest",
            "ios-build (--simulator | --device) --release",
            "no-std-check",
            "no-std-link-probe [--output PATH]",
            "sdk-inventory",
            "dependency-audit",
            "abi-audit",
            "linkage-audit",
            "zero-swift-source",
            "docs-check",
            "archive-smoke",
            "parity",
        ] {
            assert!(help.contains(command), "help is missing {command}");
        }
        assert!(help.contains("Build and inspect an unsigned Xcode archive"));
        assert!(help.contains(
            "Unavailable until a real Apple reference adapter and Rust candidate suite exist"
        ));
    }
}

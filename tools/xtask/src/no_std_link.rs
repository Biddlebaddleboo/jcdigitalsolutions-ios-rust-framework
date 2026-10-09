use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::{json_string, output_path, root, write_report};

const PROBE_CRATES: &[&str] = super::PORTABLE_CRATES;

const LINKED_OBJECT_CRATES: &[&str] = &[
    "framework-abi",
    "framework-async",
    "framework-auth",
    "framework-core",
    "framework-data",
    "framework-files",
    "framework-format",
    "framework-location",
    "framework-network",
    "framework-notifications",
    "framework-preferences",
    "framework-resources",
    "framework-secure-storage",
];

const API_CRATE_CHECKS: &[(&str, &str)] = &[
    ("framework-core", "check_core_and_platform"),
    ("framework-platform", "check_core_and_platform"),
    ("framework-alloc", "check_alloc_apis"),
    ("framework-async", "check_async_api"),
    ("framework-abi", "check_abi_apis"),
    ("framework-background", "check_framework_background"),
    (
        "framework-background-execution",
        "check_framework_background_execution",
    ),
    ("framework-accessory", "check_framework_accessory"),
    ("framework-contacts", "check_framework_contacts"),
    ("framework-calendar", "check_framework_calendar"),
    ("framework-bluetooth", "check_framework_bluetooth"),
    ("framework-cloud", "check_framework_cloud"),
    ("framework-game", "check_framework_game"),
    ("framework-maps", "check_framework_maps"),
    ("framework-vision", "check_framework_vision"),
    ("framework-roomplan", "check_framework_roomplan"),
    ("framework-web", "check_framework_web"),
    ("framework-nfc", "check_framework_nfc"),
    (
        "framework-media-authorization",
        "check_framework_media_authorization",
    ),
    ("framework-nearby", "check_framework_nearby"),
    ("framework-metal", "check_framework_metal"),
    (
        "framework-watch-connectivity",
        "check_framework_watch_connectivity",
    ),
    (
        "framework-health-authorization",
        "check_framework_health_authorization",
    ),
    (
        "framework-device-integrity",
        "check_framework_device_integrity",
    ),
    ("framework-data", "check_framework_data"),
    ("framework-app", "check_framework_app"),
    ("framework-files", "check_framework_files"),
    ("framework-preferences", "check_framework_preferences"),
    ("framework-resources", "check_framework_resources"),
    ("framework-format", "check_framework_format"),
    ("framework-key-support", "check_framework_key_support"),
    ("framework-spritekit", "check_framework_spritekit"),
    ("framework-image", "check_framework_image"),
    ("framework-network", "check_framework_network"),
    ("framework-connection", "check_framework_connection"),
    ("framework-connectivity", "check_framework_connectivity"),
    ("framework-transfer", "check_framework_transfer"),
    ("framework-secure-storage", "check_framework_secure_storage"),
    ("framework-auth", "check_framework_auth"),
    ("framework-notifications", "check_framework_notifications"),
    ("framework-location", "check_framework_location"),
    ("framework-motion", "check_framework_motion"),
    ("framework-photos", "check_framework_photos"),
    ("framework-sharing", "check_framework_sharing"),
    ("framework-audio", "check_framework_audio"),
    ("framework-ui", "check_framework_ui"),
    ("framework-media", "check_framework_media"),
];

const API_CRATES_WITHOUT_SEPARATE_OBJECT: &[&str] = &[
    "framework-alloc",
    "framework-app",
    "framework-transfer",
    "framework-motion",
    "framework-sharing",
    "framework-audio",
    "framework-ui",
    "framework-connectivity",
    "framework-connection",
    "framework-media",
    "framework-image",
    "framework-photos",
    "framework-background",
    "framework-background-execution",
    "framework-contacts",
    "framework-calendar",
    "framework-bluetooth",
    "framework-game",
    "framework-maps",
    "framework-vision",
    "framework-roomplan",
    "framework-health-authorization",
    "framework-cloud",
    "framework-web",
    "framework-nfc",
    "framework-media-authorization",
    "framework-nearby",
    "framework-metal",
    "framework-device-integrity",
    "framework-watch-connectivity",
    "framework-accessory",
    "framework-key-support",
    "framework-spritekit",
];

const COMPILE_TIME_API_CRATES: &[&str] = &["framework-platform"];

const APPLE_TARGETS: &[AppleTarget] = &[
    AppleTarget {
        rust_target: "aarch64-apple-ios",
        sdk: "iphoneos",
        clang_target: "arm64-apple-ios12.0",
        platform: "iOS device",
        platform_id: 2,
        minimum_os: "12.0",
        artifact_name: "no-std-link-probe-ios-device.dylib",
        map_name: "no-std-link-probe-ios-device.map",
    },
    AppleTarget {
        rust_target: "aarch64-apple-ios-sim",
        sdk: "iphonesimulator",
        clang_target: "arm64-apple-ios14.0-simulator",
        platform: "iOS Simulator",
        platform_id: 7,
        minimum_os: "14.0",
        artifact_name: "no-std-link-probe-ios-sim.dylib",
        map_name: "no-std-link-probe-ios-sim.map",
    },
];

const PERSONALITY_ABORT_SHIM: &str = r#"#include <stdint.h>
#include <stdlib.h>

int32_t rust_eh_personality(int32_t version, ...) {
    (void)version;
    abort();
}
"#;

const TEMPLATE_MANIFEST: &str = include_str!("../fixtures/no-std-link-probe/Cargo.toml.in");
const PROBE_RUST: &str = include_str!("../fixtures/no-std-link-probe/src/lib.rs");
const PROBE_C: &str = include_str!("../fixtures/no-std-link-probe/src/main.c");

struct AppleTarget {
    rust_target: &'static str,
    sdk: &'static str,
    clang_target: &'static str,
    platform: &'static str,
    platform_id: u8,
    minimum_os: &'static str,
    artifact_name: &'static str,
    map_name: &'static str,
}

struct AppleProbeResult {
    rust_target: String,
    sdk: String,
    sdk_version: String,
    clang_target: String,
    platform: String,
    minimum_os: String,
    archive: String,
    artifact: String,
    architecture: String,
    imports: Vec<String>,
    undefined_symbols: Vec<String>,
    linked_object_crates: Vec<String>,
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let output = output_path(args, "--output")?;
    if env::consts::OS != "macos" {
        return Err("no-std-link-probe requires a macOS host with clang, otool, nm, and ar".into());
    }

    let root = root();
    let host = rustc_host(&root)?;
    if !host.ends_with("-apple-darwin") {
        return Err(format!(
            "no-std-link-probe requires an Apple Darwin Rust host, found `{host}`"
        ));
    }

    let report_dir = root.join("target/xtask/no-std-link-probe");
    let project = report_dir.join("project");
    let build_dir = report_dir.join("cargo-target");
    if project.exists() {
        fs::remove_dir_all(&project)
            .map_err(|error| format!("remove {}: {error}", project.display()))?;
    }
    fs::create_dir_all(project.join("src"))
        .map_err(|error| format!("create {}: {error}", project.display()))?;

    let root_text = root
        .canonicalize()
        .map_err(|error| format!("resolve repository root: {error}"))?
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let manifest = TEMPLATE_MANIFEST.replace("__ROOT__", &root_text);
    fs::write(project.join("Cargo.toml"), manifest)
        .map_err(|error| format!("write probe manifest: {error}"))?;
    fs::write(project.join("src/lib.rs"), PROBE_RUST)
        .map_err(|error| format!("write Rust probe: {error}"))?;
    fs::write(project.join("src/main.c"), PROBE_C)
        .map_err(|error| format!("write C probe: {error}"))?;
    fs::write(project.join("src/personality.c"), PERSONALITY_ABORT_SHIM)
        .map_err(|error| format!("write target panic-abort fallback: {error}"))?;

    if LINKED_OBJECT_CRATES.len()
        + API_CRATES_WITHOUT_SEPARATE_OBJECT.len()
        + COMPILE_TIME_API_CRATES.len()
        != PROBE_CRATES.len()
        || API_CRATE_CHECKS.len() != PROBE_CRATES.len()
    {
        return Err("probe crate classifications do not cover PORTABLE_CRATES exactly".into());
    }
    for package in PROBE_CRATES {
        let dependency = format!("{package} = {{ path = \"__ROOT__/crates/{package}\"");
        if !TEMPLATE_MANIFEST.contains(&dependency) {
            return Err(format!("probe manifest omits direct dependency {package}"));
        }
        let crate_name = package.replace('-', "_");
        let Some((_, check)) = API_CRATE_CHECKS
            .iter()
            .find(|(api_package, _)| api_package == package)
        else {
            return Err(format!("probe has no public API check for {package}"));
        };
        if !PROBE_RUST.contains(&crate_name)
            || !PROBE_RUST.contains(&format!("fn {check}("))
            || !PROBE_RUST.contains(&format!("if !{check}()"))
        {
            return Err(format!(
                "Rust probe omits an invoked public API check for {package}"
            ));
        }
        if !LINKED_OBJECT_CRATES.contains(package)
            && !API_CRATES_WITHOUT_SEPARATE_OBJECT.contains(package)
            && !COMPILE_TIME_API_CRATES.contains(package)
        {
            return Err(format!("probe has no link-evidence class for {package}"));
        }
        let source = root.join("crates").join(package).join("src/lib.rs");
        let contents = fs::read_to_string(&source)
            .map_err(|error| format!("read {}: {error}", source.display()))?;
        if !contents.lines().any(|line| line.trim() == "#![no_std]") {
            return Err(format!(
                "{package} does not declare `#![no_std]` in src/lib.rs"
            ));
        }
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let manifest_path = project.join("Cargo.toml");
    let manifest_text = manifest_path.to_string_lossy().into_owned();
    let build_dir_text = build_dir.to_string_lossy().into_owned();
    let cargo_args = [
        "build",
        "--offline",
        "--quiet",
        "--manifest-path",
        manifest_text.as_str(),
        "--release",
        "--target",
        host.as_str(),
    ];
    println!("{cargo} {}", cargo_args.join(" "));
    let cargo_output = Command::new(&cargo)
        .current_dir(&root)
        .env("CARGO_TARGET_DIR", &build_dir_text)
        .args(cargo_args)
        .output()
        .map_err(|error| format!("run Cargo no_std link-probe build: {error}"))?;
    require_success("Cargo no_std link-probe build", &cargo_output)?;

    let archive = build_dir.join(&host).join("release/libno_std_link_probe.a");
    if !archive.is_file() {
        return Err(format!(
            "probe static library is missing: {}",
            archive.display()
        ));
    }
    let c_main = project.join("src/main.c");
    let personality = project.join("src/personality.c");
    let executable = report_dir.join("no-std-link-probe");
    let map = report_dir.join("no-std-link-probe.map");
    let c_main_text = c_main.to_string_lossy().into_owned();
    let personality_text = personality.to_string_lossy().into_owned();
    let archive_text = archive.to_string_lossy().into_owned();
    let executable_text = executable.to_string_lossy().into_owned();
    let map_text = map.to_string_lossy().into_owned();
    let map_argument = format!("-Wl,-map,{map_text}");
    let clang_args = [
        c_main_text.as_str(),
        personality_text.as_str(),
        archive_text.as_str(),
        "-Wl,-dead_strip",
        map_argument.as_str(),
        "-o",
        executable_text.as_str(),
    ];
    println!("clang {}", clang_args.join(" "));
    let clang_output = Command::new("clang")
        .current_dir(&root)
        .args(clang_args)
        .output()
        .map_err(|error| format!("run clang link probe: {error}"))?;
    require_success("clang link probe", &clang_output)?;

    let map_bytes =
        fs::read(&map).map_err(|error| format!("read linker map {}: {error}", map.display()))?;
    let map_contents = String::from_utf8_lossy(&map_bytes);
    let object_files = linker_map_objects(&map_contents)
        .ok_or_else(|| "Apple linker map has no `Object files` section".to_owned())?;
    let archive_members = command_text("ar", &["-t", &archive_text], &root)?;
    if contains_rust_std_member(&archive_members) {
        return Err("probe static library contains a Rust std archive member".into());
    }
    if contains_rust_std_member(&map_contents) {
        return Err("probe linker map contains a Rust std object member".into());
    }
    for crate_name in LINKED_OBJECT_CRATES {
        let normalized = crate_name.replace('-', "_");
        if !object_files.contains(&normalized) && !object_files.contains(crate_name) {
            return Err(format!(
                "probe linker map has no linked object evidence for {crate_name}"
            ));
        }
    }

    let symbols = command_text("nm", &["-g", &executable_text], &root)?;
    for symbol in [
        "_framework_no_std_link_probe",
        "_framework_owned_buffer_destroy",
    ] {
        if !symbols.contains(symbol) {
            return Err(format!("linked probe is missing required symbol {symbol}"));
        }
    }

    let imports = dynamic_imports(&executable, &root)?;
    if imports.len() != 1 || imports[0] != "libSystem.B.dylib" {
        return Err(format!(
            "probe import set differs from the exact allowlist [libSystem.B.dylib]: {imports:?}"
        ));
    }

    println!("run {executable_text}");
    let run_output = Command::new(&executable)
        .current_dir(&root)
        .output()
        .map_err(|error| format!("run linked no_std probe: {error}"))?;
    require_success("linked no_std probe", &run_output)?;
    if run_output.status.code() != Some(0) {
        return Err(format!(
            "linked no_std probe returned {:?}",
            run_output.status.code()
        ));
    }

    let apple_probes = APPLE_TARGETS
        .iter()
        .map(|target| run_apple_target_probe(target, &root, &project, &manifest_path, &build_dir))
        .collect::<Result<Vec<_>, _>>()?;

    let probed_crates = json_array(PROBE_CRATES);
    let linked_object_crates = json_array(LINKED_OBJECT_CRATES);
    let crates_without_separate_object = json_array(API_CRATES_WITHOUT_SEPARATE_OBJECT);
    let no_separate_object_count = API_CRATES_WITHOUT_SEPARATE_OBJECT.len();
    let apple_probe_reports = apple_probes
        .iter()
        .map(apple_probe_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let report = format!(
        "{{\n  \"schema_version\": 2,\n  \"scope\": \"bounded no_std probe links for host macOS, arm64 iOS device, and arm64 iOS Simulator\",\n  \"host_target\": {},\n  \"probe_crates\": {},\n  \"remaining_portable_crates_without_link_probe\": [],\n  \"no_std_source_roots_checked\": {},\n  \"linked_object_evidence\": {},\n  \"api_crates_without_separate_object_evidence\": {},\n  \"compile_time_api_crates\": [\"framework-platform\"],\n  \"explicit_allocator\": \"C malloc/free global allocator with checked alignment; alloc::alloc request/free\",\n  \"collection_allocation_scope\": \"framework-alloc public collection calls use an empty slab; nonempty collection allocation is not exercised\",\n  \"host_rust_std_archive_member\": false,\n  \"host_rust_std_link_map_member\": false,\n  \"required_symbols\": [\"_framework_no_std_link_probe\", \"_framework_owned_buffer_destroy\"],\n  \"host_dynamic_imports\": [\"libSystem.B.dylib\"],\n  \"host_probe_exit_code\": 0,\n  \"apple_target_link_probes\": [\n    {}\n  ],\n  \"device_or_simulator_linkage_checked\": true,\n  \"all_portable_crates_host_no_std_linkage_proven\": true,\n  \"all_portable_crates_apple_target_selected_api_linkage_audited\": true,\n  \"all_portable_crates_no_std_linkage_proven\": false,\n  \"limits\": [\"Apple target artifacts are link-only and were not executed on a simulator or device\", \"link-probe-local rust_eh_personality shim aborts if called; it is a fail-fast panic=abort fallback, not an unwinding personality implementation\", \"link evidence covers these bounded probe artifacts and selected API calls, not every framework code path or runtime behavior\", \"{} portable APIs have no separate object entry in the linker map; their invoked checks reside in the probe object or are compile-time APIs\"]\n}}\n",
        json_string(&host),
        probed_crates,
        probed_crates,
        linked_object_crates,
        crates_without_separate_object,
        apple_probe_reports,
        no_separate_object_count
    );
    let output =
        output.or_else(|| Some(PathBuf::from("target/xtask/no-std-link-probe/report.json")));
    write_report(output, &report)
}

fn rustc_host(root: &Path) -> Result<String, String> {
    let output = Command::new("rustc")
        .args(["-vV"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("run rustc -vV: {error}"))?;
    require_success("rustc -vV", &output)?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
        .ok_or_else(|| "rustc -vV did not report a host target".into())
}

fn run_apple_target_probe(
    target: &AppleTarget,
    root: &Path,
    project: &Path,
    manifest_path: &Path,
    build_dir: &Path,
) -> Result<AppleProbeResult, String> {
    let sdk_path = command_text("xcrun", &["--sdk", target.sdk, "--show-sdk-path"], root)?
        .trim()
        .to_owned();
    let sdk_version = command_text("xcrun", &["--sdk", target.sdk, "--show-sdk-version"], root)?
        .trim()
        .to_owned();
    if !Path::new(&sdk_path).is_dir() {
        return Err(format!("{} SDK path is missing: {sdk_path}", target.sdk));
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let manifest_text = manifest_path.to_string_lossy().into_owned();
    let build_dir_text = build_dir.to_string_lossy().into_owned();
    let cargo_args = [
        "build",
        "--offline",
        "--quiet",
        "--manifest-path",
        manifest_text.as_str(),
        "--release",
        "--target",
        target.rust_target,
    ];
    println!("{cargo} {}", cargo_args.join(" "));
    let cargo_output = Command::new(&cargo)
        .current_dir(root)
        .env("CARGO_TARGET_DIR", &build_dir_text)
        .args(cargo_args)
        .output()
        .map_err(|error| {
            format!(
                "run Cargo {} no_std link-probe build: {error}",
                target.rust_target
            )
        })?;
    require_success(
        &format!("Cargo {} no_std link-probe build", target.rust_target),
        &cargo_output,
    )?;

    let archive = build_dir
        .join(target.rust_target)
        .join("release/libno_std_link_probe.a");
    if !archive.is_file() {
        return Err(format!(
            "{} probe static library is missing: {}",
            target.rust_target,
            archive.display()
        ));
    }

    let c_main = project.join("src/main.c");
    let personality = project.join("src/personality.c");
    let artifact = root
        .join("target/xtask/no-std-link-probe")
        .join(target.artifact_name);
    let map = root
        .join("target/xtask/no-std-link-probe")
        .join(target.map_name);
    let c_main_text = c_main.to_string_lossy().into_owned();
    let personality_text = personality.to_string_lossy().into_owned();
    let archive_text = archive.to_string_lossy().into_owned();
    let artifact_text = artifact.to_string_lossy().into_owned();
    let map_text = map.to_string_lossy().into_owned();
    let map_argument = format!("-Wl,-map,{map_text}");
    let clang_args = vec![
        "--sdk".to_owned(),
        target.sdk.to_owned(),
        "clang".to_owned(),
        "-target".to_owned(),
        target.clang_target.to_owned(),
        "-arch".to_owned(),
        "arm64".to_owned(),
        "-isysroot".to_owned(),
        sdk_path,
        "-dynamiclib".to_owned(),
        "-Wl,-undefined,error".to_owned(),
        "-Wl,-dead_strip".to_owned(),
        "-Wl,-u,_rust_eh_personality".to_owned(),
        map_argument,
        c_main_text,
        archive_text.clone(),
        personality_text,
        "-o".to_owned(),
        artifact_text.clone(),
    ];
    println!("xcrun {}", clang_args.join(" "));
    let clang_output = Command::new("xcrun")
        .current_dir(root)
        .args(&clang_args)
        .output()
        .map_err(|error| {
            format!(
                "run {} no_std target link probe: {error}",
                target.rust_target
            )
        })?;
    require_success(
        &format!("{} no_std target link probe", target.rust_target),
        &clang_output,
    )?;

    let map_bytes =
        fs::read(&map).map_err(|error| format!("read linker map {}: {error}", map.display()))?;
    let map_contents = String::from_utf8_lossy(&map_bytes);
    let object_files = linker_map_objects(&map_contents).ok_or_else(|| {
        format!(
            "{} linker map has no `Object files` section",
            target.rust_target
        )
    })?;
    let archive_members = command_text("ar", &["-t", &archive_text], root)?;
    if contains_rust_std_member(&archive_members) {
        return Err(format!(
            "{} probe static library contains a Rust std archive member",
            target.rust_target
        ));
    }
    if contains_rust_std_member(&map_contents) {
        return Err(format!(
            "{} probe linker map contains a Rust std object member",
            target.rust_target
        ));
    }
    let mut linked_object_crates = Vec::new();
    for crate_name in LINKED_OBJECT_CRATES {
        let normalized = crate_name.replace('-', "_");
        if !object_files.contains(&normalized) && !object_files.contains(crate_name) {
            return Err(format!(
                "{} linker map has no linked object evidence for {crate_name}",
                target.rust_target
            ));
        }
        linked_object_crates.push((*crate_name).to_owned());
    }

    let artifact_symbols = command_text("nm", &["-g", &artifact_text], root)?;
    let detailed_symbols = command_text("nm", &["-m", &artifact_text], root)?;
    for symbol in [
        "_framework_no_std_link_probe",
        "_framework_owned_buffer_destroy",
        "_rust_eh_personality",
    ] {
        if !artifact_symbols.contains(symbol) {
            return Err(format!(
                "{} linked probe is missing required symbol {symbol}",
                target.rust_target
            ));
        }
    }
    if !detailed_symbols.contains("(__TEXT,__text) external _rust_eh_personality") {
        return Err(format!(
            "{} linked probe does not define its fail-fast rust_eh_personality shim",
            target.rust_target
        ));
    }
    let undefined_symbols = undefined_symbols(&artifact, root)?;
    let unresolved_rust_symbols = undefined_symbols
        .iter()
        .filter(|symbol| is_unresolved_rust_symbol(symbol))
        .collect::<Vec<_>>();
    if !unresolved_rust_symbols.is_empty() {
        return Err(format!(
            "{} linked probe has unresolved Rust symbols: {unresolved_rust_symbols:?}",
            target.rust_target
        ));
    }
    if undefined_symbols
        .iter()
        .any(|symbol| symbol.trim_start_matches('_') == "rust_eh_personality")
    {
        return Err(format!(
            "{} linked probe leaves rust_eh_personality unresolved",
            target.rust_target
        ));
    }

    let imports = dynamic_imports(&artifact, root)?;
    if imports.len() != 1 || imports[0] != "libSystem.B.dylib" {
        return Err(format!(
            "{} probe import set differs from the exact allowlist [libSystem.B.dylib]: {imports:?}",
            target.rust_target
        ));
    }
    let architecture = command_text("lipo", &["-info", &artifact_text], root)?;
    if !architecture.contains("architecture: arm64") && !architecture.contains("are: arm64") {
        return Err(format!(
            "{} linked probe is not arm64: {}",
            target.rust_target,
            architecture.trim()
        ));
    }
    let load_commands = command_text("otool", &["-l", &artifact_text], root)?;
    if !load_commands.contains(&format!("platform {}", target.platform_id))
        || !load_commands.contains(&format!("minos {}", target.minimum_os))
        || !load_commands.contains(&format!("sdk {sdk_version}"))
    {
        return Err(format!(
            "{} linked probe has unexpected LC_BUILD_VERSION data",
            target.rust_target
        ));
    }

    Ok(AppleProbeResult {
        rust_target: target.rust_target.to_owned(),
        sdk: target.sdk.to_owned(),
        sdk_version,
        clang_target: target.clang_target.to_owned(),
        platform: target.platform.to_owned(),
        minimum_os: target.minimum_os.to_owned(),
        archive: relative_path(&archive, root),
        artifact: relative_path(&artifact, root),
        architecture: "arm64".to_owned(),
        imports,
        undefined_symbols,
        linked_object_crates,
    })
}

fn apple_probe_json(result: &AppleProbeResult) -> String {
    let linked_object_crates = result
        .linked_object_crates
        .iter()
        .map(|crate_name| json_string(crate_name))
        .collect::<Vec<_>>()
        .join(", ");
    let imports = result
        .imports
        .iter()
        .map(|import| json_string(import))
        .collect::<Vec<_>>()
        .join(", ");
    let undefined_symbols = result
        .undefined_symbols
        .iter()
        .map(|symbol| json_string(symbol))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{{\"rust_target\":{},\"sdk\":{},\"sdk_version\":{},\"clang_target\":{},\"platform\":{},\"minimum_os\":{},\"architecture\":{},\"archive\":{},\"linked_artifact\":{},\"linked_object_evidence\":[{}],\"rust_std_archive_member\":false,\"rust_std_link_map_member\":false,\"dynamic_imports\":[{}],\"undefined_symbols_resolved_from_imports\":[{}],\"unresolved_rust_symbols\":[],\"rust_eh_personality_resolved_by_abort_shim\":true,\"probe_executed\":false,\"runtime_verified\":false}}",
        json_string(&result.rust_target),
        json_string(&result.sdk),
        json_string(&result.sdk_version),
        json_string(&result.clang_target),
        json_string(&result.platform),
        json_string(&result.minimum_os),
        json_string(&result.architecture),
        json_string(&result.archive),
        json_string(&result.artifact),
        linked_object_crates,
        imports,
        undefined_symbols
    )
}

fn relative_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn undefined_symbols(artifact: &Path, root: &Path) -> Result<Vec<String>, String> {
    let artifact_text = artifact.to_string_lossy();
    let output = command_text("nm", &["-u", artifact_text.as_ref()], root)?;
    Ok(output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.ends_with(": no symbols"))
        .map(|line| line.split_whitespace().last().unwrap_or(line).to_owned())
        .collect())
}

fn is_unresolved_rust_symbol(symbol: &str) -> bool {
    let symbol = symbol.trim_start_matches('_');
    symbol.starts_with("ZN")
        || (symbol.starts_with('R') && symbol.as_bytes().get(1) == Some(&b'N'))
        || symbol.starts_with("rust_")
        || symbol == "rust_eh_personality"
}

fn command_text(program: &str, args: &[&str], root: &Path) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("run {program}: {error}"))?;
    require_success(program, &output)?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn require_success(label: &str, output: &Output) -> Result<(), String> {
    if output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(format!(
        "{label} failed with {}\n{stdout}\n{stderr}",
        output.status
    ))
}

fn dynamic_imports(executable: &Path, root: &Path) -> Result<Vec<String>, String> {
    let executable_text = executable.to_string_lossy();
    let output = command_text("otool", &["-L", executable_text.as_ref()], root)?;
    let executable_name = executable.file_name().and_then(|name| name.to_str());
    let imports = output
        .lines()
        .skip(1)
        .filter_map(|line| line.trim().split(" (").next())
        .filter(|path| !path.is_empty())
        .filter(|path| {
            Path::new(path).file_name().and_then(|name| name.to_str()) != executable_name
        })
        .map(|path| {
            Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(path)
                .to_owned()
        })
        .collect();
    Ok(imports)
}

fn contains_rust_std_member(text: &str) -> bool {
    text.split(|character: char| {
        !character.is_ascii_alphanumeric() && character != '_' && character != '-'
    })
    .any(|token| {
        let token = token.strip_prefix("lib").unwrap_or(token);
        let Some(hash) = token.strip_prefix("std-") else {
            return false;
        };
        let hash_length = hash
            .chars()
            .take_while(|character| character.is_ascii_hexdigit())
            .count();
        hash_length >= 8
    })
}

fn linker_map_objects(text: &str) -> Option<&str> {
    text.split_once("# Object files:")?
        .1
        .split_once("# Sections:")
        .map(|(objects, _)| objects)
}

fn json_array(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

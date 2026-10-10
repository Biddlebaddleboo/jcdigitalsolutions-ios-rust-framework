//! PATH-installed native build engine for capability-scoped iOS C bridges.
//! The engine has no shipping runtime dependency and does not define native ABI or capability behavior.

use serde::Deserialize;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};

const RERUN_ENV: [&str; 2] = ["SDKROOT", "DEVELOPER_DIR"];
const MAX_BUILD_SPEC_BYTES: usize = 65_536;
const MAX_BUILD_RESULT_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A native CPU architecture supported by the iOS build config.
pub enum Architecture {
    /// 64-bit ARM, named `arm64` by Clang.
    Arm64,
    /// 64-bit x86, supported here for iOS Simulator only.
    X86_64,
}

impl Architecture {
    fn clang_name(self) -> &'static str {
        match self {
            Self::Arm64 => "arm64",
            Self::X86_64 => "x86_64",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// An iOS deployment floor for the native C compilation target.
pub struct DeploymentTarget {
    /// The major OS version.
    pub major: u16,
    /// The minor OS version.
    pub minor: u8,
}

impl DeploymentTarget {
    fn as_string(self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// A Cargo iOS target with a fixed SDK and Apple environment mapping.
pub enum IosTarget {
    /// `aarch64-apple-ios`, using the iPhoneOS SDK.
    DeviceArm64,
    /// `aarch64-apple-ios-sim`, using the iPhone Simulator SDK.
    SimulatorArm64,
    /// `x86_64-apple-ios`, using the iPhone Simulator SDK.
    SimulatorX86_64,
}

impl IosTarget {
    fn from_cargo_target(target: &str) -> Option<Self> {
        match target {
            "aarch64-apple-ios" => Some(Self::DeviceArm64),
            "aarch64-apple-ios-sim" => Some(Self::SimulatorArm64),
            "x86_64-apple-ios" => Some(Self::SimulatorX86_64),
            _ => None,
        }
    }

    fn cargo_target(self) -> &'static str {
        match self {
            Self::DeviceArm64 => "aarch64-apple-ios",
            Self::SimulatorArm64 => "aarch64-apple-ios-sim",
            Self::SimulatorX86_64 => "x86_64-apple-ios",
        }
    }

    fn architecture(self) -> Architecture {
        match self {
            Self::DeviceArm64 | Self::SimulatorArm64 => Architecture::Arm64,
            Self::SimulatorX86_64 => Architecture::X86_64,
        }
    }

    fn sdk(self) -> &'static str {
        match self {
            Self::DeviceArm64 => "iphoneos",
            Self::SimulatorArm64 | Self::SimulatorX86_64 => "iphonesimulator",
        }
    }

    fn clang_target(self, minimum_os: DeploymentTarget) -> String {
        let simulator = match self {
            Self::DeviceArm64 => "",
            Self::SimulatorArm64 | Self::SimulatorX86_64 => "-simulator",
        };
        format!(
            "{}-apple-ios{}{}",
            self.architecture().clang_name(),
            minimum_os.as_string(),
            simulator
        )
    }
}

/// Declarative native build inputs kept local to one capability crate.
pub struct IosNativeBuildConfig {
    /// Capability label used in unsupported-target errors.
    pub capability: &'static str,
    /// Static archive name without the `lib` prefix or `.a` suffix.
    pub library: &'static str,
    /// Manifest-relative C source files, with unique file stems.
    pub sources: &'static [&'static str],
    /// Manifest-relative C headers used by the sources and tracked by Cargo.
    pub headers: &'static [&'static str],
    /// Public Apple frameworks linked by this capability crate.
    pub frameworks: &'static [&'static str],
    /// Deployment floor used in each Clang target triple.
    pub minimum_os: DeploymentTarget,
    /// Architectures allowed by this capability.
    pub supported_architectures: &'static [Architecture],
    /// Exact Cargo target triples allowed by this capability.
    pub targets: &'static [IosTarget],
    /// Additional Clang options, before `-c`, source, and output arguments.
    pub compiler_options: &'static [&'static str],
    /// Phase label used for C compilation failures.
    pub compile_description: &'static str,
    /// Phase label used for archive creation failures.
    pub archive_description: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BuildEnvironment {
    target_os: Option<String>,
    target: Option<String>,
    out_dir: Option<PathBuf>,
    manifest_dir: Option<PathBuf>,
}

impl BuildEnvironment {
    fn from_process() -> Self {
        Self {
            target_os: std::env::var("CARGO_CFG_TARGET_OS").ok(),
            target: std::env::var("TARGET").ok(),
            out_dir: std::env::var_os("OUT_DIR").map(PathBuf::from),
            manifest_dir: std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Capture,
    Inherit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandResult {
    success: bool,
    status: String,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl CommandResult {
    fn from_output(output: Output) -> Self {
        Self {
            success: output.status.success(),
            status: output.status.to_string(),
            stdout: output.stdout,
            stderr: output.stderr,
        }
    }

    fn from_status(status: std::process::ExitStatus) -> Self {
        Self {
            success: status.success(),
            status: status.to_string(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

/// Injectable command boundary used by build scripts and deterministic tests.
trait CommandRunner {
    fn run(
        &mut self,
        program: &OsStr,
        args: &[OsString],
        output_mode: OutputMode,
    ) -> io::Result<CommandResult>;
}

struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run(
        &mut self,
        program: &OsStr,
        args: &[OsString],
        output_mode: OutputMode,
    ) -> io::Result<CommandResult> {
        let mut command = Command::new(program);
        command.args(args);
        match output_mode {
            OutputMode::Capture => command.output().map(CommandResult::from_output),
            OutputMode::Inherit => command
                .stdout(Stdio::null())
                .status()
                .map(CommandResult::from_status),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BuildError(String);

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for BuildError {}

/// Run the native build with process environment and real `xcrun` commands.
pub fn build(config: &IosNativeBuildConfig) {
    let mut runner = ProcessRunner;
    let mut cargo_output = io::stdout().lock();
    if let Err(error) = build_with(
        config,
        &BuildEnvironment::from_process(),
        &mut runner,
        &mut cargo_output,
    ) {
        panic!("{error}");
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeBuildSpec {
    schema_version: u32,
    capability: String,
    library: String,
    sources: Vec<String>,
    headers: Vec<String>,
    frameworks: Vec<String>,
    minimum_os: String,
    targets: Vec<NativeBuildTargetSpec>,
    feature_guards: Vec<NativeBuildFeatureGuard>,
    compiler_options: Vec<String>,
    compile_description: String,
    archive_description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeBuildTargetSpec {
    triple: String,
    sdk: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeBuildFeatureGuard {
    name: String,
    enabled: bool,
}

/// Run the installed build tool protocol and emit one JSON result on stdout.
pub fn run_cli(args: &[String]) -> Result<(), String> {
    if args == ["--help"] || args == ["-h"] {
        print!("{}", build_help());
        return Ok(());
    }
    if args == ["--version"] {
        println!("ios-rust-build {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args == ["--version", "--format", "json"] {
        let source_sha = option_env!("TOOL_SOURCE_SHA").unwrap_or("working-tree");
        println!(
            "{}",
            serde_json::json!({
                "tool": "ios-rust-build",
                "version": env!("CARGO_PKG_VERSION"),
                "schema_major": 1,
                "build_spec_schema": 1,
                "result_schema": 1,
                "release_id": format!("ios-rust-tools-{}+{}", env!("CARGO_PKG_VERSION"), source_sha),
                "source_sha": source_sha,
                "rustc_version": option_env!("TOOL_RUSTC_VERSION").unwrap_or("unknown"),
                "host": option_env!("HOST").unwrap_or("unknown")
            })
        );
        return Ok(());
    }

    let options = parse_cli_options(args)?;
    let workspace_root = required_option(&options, "--workspace-root")?;
    let spec_path = required_option(&options, "--spec")?;
    let target = required_option(&options, "--target")?;
    let target_os = required_option(&options, "--target-os")?;
    let out_dir = required_option(&options, "--out-dir")?;
    let manifest_dir = required_option(&options, "--manifest-dir")?;
    if required_option(&options, "--format")? != "json" {
        return Err("`--format` must be `json`".into());
    }
    let compiled_host = option_env!("HOST").ok_or_else(|| {
        "ios-rust-build has no pinned host provenance; install the verified PATH tool package"
            .to_owned()
    })?;
    if runtime_host_triple() != Some(compiled_host) {
        return Err(format!(
            "ios-rust-build host mismatch: binary={compiled_host}, runtime={}",
            runtime_host_triple().unwrap_or("unsupported")
        ));
    }

    let workspace_root = PathBuf::from(workspace_root)
        .canonicalize()
        .map_err(|error| format!("invalid workspace root: {error}"))?;
    let manifest_dir = PathBuf::from(manifest_dir)
        .canonicalize()
        .map_err(|error| format!("invalid manifest directory: {error}"))?;
    if !manifest_dir.starts_with(&workspace_root) {
        return Err("manifest directory is outside the workspace root".into());
    }
    let spec_path = resolve_workspace_path(&workspace_root, spec_path, "build spec")?;
    let spec_bytes = std::fs::read(&spec_path)
        .map_err(|error| format!("cannot read build spec {}: {error}", spec_path.display()))?;
    if spec_bytes.len() > MAX_BUILD_SPEC_BYTES {
        return Err(format!(
            "build spec exceeds the {MAX_BUILD_SPEC_BYTES} byte limit"
        ));
    }
    let spec: NativeBuildSpec = serde_json::from_slice(&spec_bytes)
        .map_err(|error| format!("invalid build spec {}: {error}", spec_path.display()))?;
    if spec.schema_version != 1 {
        return Err(format!(
            "unsupported build spec schema major {}; this tool supports 1",
            spec.schema_version
        ));
    }
    for guard in &spec.feature_guards {
        if guard.name.is_empty()
            || !guard
                .name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(format!("invalid feature guard `{}`", guard.name));
        }
        let env_name = format!("CARGO_FEATURE_{}", guard.name);
        let enabled = std::env::var_os(&env_name).is_some();
        if enabled != guard.enabled {
            return Err(format!(
                "feature guard {env_name} expected {}, got {}",
                if guard.enabled { "enabled" } else { "disabled" },
                if enabled { "enabled" } else { "disabled" }
            ));
        }
    }
    for source in &spec.sources {
        resolve_package_path(&workspace_root, &manifest_dir, source, "native source")?;
    }
    for header in &spec.headers {
        resolve_package_path(&workspace_root, &manifest_dir, header, "native header")?;
    }
    let minimum_os = parse_deployment_target(&spec.minimum_os)?;
    let targets = spec
        .targets
        .iter()
        .map(|target_spec| {
            let target = IosTarget::from_cargo_target(&target_spec.triple).ok_or_else(|| {
                format!("unsupported configured iOS target `{}`", target_spec.triple)
            })?;
            if target.sdk() != target_spec.sdk {
                return Err(format!(
                    "SDK `{}` does not match target `{}`",
                    target_spec.sdk, target_spec.triple
                ));
            }
            Ok(target)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let architectures = targets
        .iter()
        .map(|target| target.architecture())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let config = IosNativeBuildConfig {
        capability: leak_string(spec.capability),
        library: leak_string(spec.library),
        sources: leak_strings(spec.sources),
        headers: leak_strings(spec.headers),
        frameworks: leak_strings(spec.frameworks),
        minimum_os,
        supported_architectures: leak_values(architectures),
        targets: leak_values(targets),
        compiler_options: leak_strings(spec.compiler_options),
        compile_description: leak_string(spec.compile_description),
        archive_description: leak_string(spec.archive_description),
    };
    let environment = BuildEnvironment {
        target_os: Some(target_os.to_owned()),
        target: Some(target.to_owned()),
        out_dir: Some(PathBuf::from(out_dir)),
        manifest_dir: Some(manifest_dir.clone()),
    };
    if let Some(cargo_target) = std::env::var("TARGET").ok()
        && cargo_target != target
    {
        return Err(format!(
            "explicit target `{target}` does not match Cargo TARGET `{cargo_target}`"
        ));
    }
    if let Some(cargo_os) = std::env::var("CARGO_CFG_TARGET_OS").ok()
        && cargo_os != target_os
    {
        return Err(format!(
            "explicit target OS `{target_os}` does not match CARGO_CFG_TARGET_OS `{cargo_os}`"
        ));
    }
    if target_os == "ios" && std::env::consts::OS != "macos" {
        return Err("iOS native builds require a macOS host with Xcode command line tools".into());
    }
    std::env::set_current_dir(&manifest_dir)
        .map_err(|error| format!("cannot enter manifest directory: {error}"))?;
    let mut runner = ProcessRunner;
    let mut cargo_output = Vec::new();
    build_with(&config, &environment, &mut runner, &mut cargo_output)
        .map_err(|error| error.to_string())?;
    let directives = String::from_utf8(cargo_output)
        .map_err(|error| format!("Cargo directives are not UTF-8: {error}"))?
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let output = serde_json::json!({
        "schema_version":1,
        "status":"ok",
        "directives":directives
    })
    .to_string();
    if output.len() > MAX_BUILD_RESULT_BYTES {
        return Err(format!(
            "build result exceeds the {MAX_BUILD_RESULT_BYTES} byte output limit"
        ));
    }
    println!("{output}");
    Ok(())
}

fn parse_cli_options(args: &[String]) -> Result<std::collections::BTreeMap<&str, &str>, String> {
    if args.first().map(String::as_str) != Some("build") {
        return Err("usage: ios-rust-build build --workspace-root PATH --spec PATH --target TRIPLE --target-os OS --out-dir PATH --manifest-dir PATH --format json".into());
    }
    let mut options = std::collections::BTreeMap::new();
    let mut index = 1;
    while index < args.len() {
        let name = args[index].as_str();
        if ![
            "--workspace-root",
            "--spec",
            "--target",
            "--target-os",
            "--out-dir",
            "--manifest-dir",
            "--format",
        ]
        .contains(&name)
        {
            return Err(format!("unknown build option `{name}`"));
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("`{name}` requires a value"))?;
        if value.starts_with('-') && name != "--spec" {
            return Err(format!("invalid value for `{name}`"));
        }
        if options.insert(name, value.as_str()).is_some() {
            return Err(format!("duplicate build option `{name}`"));
        }
        index += 2;
    }
    Ok(options)
}

fn required_option<'a>(
    options: &'a std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<&'a str, String> {
    options
        .get(name)
        .copied()
        .ok_or_else(|| format!("missing required option `{name}`"))
}

fn resolve_workspace_path(root: &Path, path: &str, kind: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    let resolved = if path.is_absolute() {
        path.canonicalize()
    } else {
        root.join(path).canonicalize()
    }
    .map_err(|error| format!("invalid {kind} path: {error}"))?;
    if !resolved.starts_with(root) {
        return Err(format!("{kind} is outside the workspace root"));
    }
    Ok(resolved)
}

fn resolve_package_path(
    root: &Path,
    manifest_dir: &Path,
    path: &str,
    kind: &str,
) -> Result<PathBuf, String> {
    if path.is_empty() || path.contains('\n') || path.contains('\r') {
        return Err(format!("invalid {kind} path"));
    }
    let resolved = manifest_dir
        .join(path)
        .canonicalize()
        .map_err(|error| format!("invalid {kind} `{path}`: {error}"))?;
    if !resolved.starts_with(root) {
        return Err(format!("{kind} `{path}` is outside the workspace root"));
    }
    Ok(resolved)
}

fn parse_deployment_target(version: &str) -> Result<DeploymentTarget, String> {
    let mut parts = version.split('.');
    let major = parts
        .next()
        .and_then(|part| part.parse::<u16>().ok())
        .ok_or_else(|| format!("invalid iOS deployment floor `{version}`"))?;
    let minor = parts
        .next()
        .and_then(|part| part.parse::<u8>().ok())
        .ok_or_else(|| format!("invalid iOS deployment floor `{version}`"))?;
    if parts.next().is_some() {
        return Err(format!("invalid iOS deployment floor `{version}`"));
    }
    Ok(DeploymentTarget { major, minor })
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn leak_strings(values: Vec<String>) -> &'static [&'static str] {
    let values = values.into_iter().map(leak_string).collect::<Vec<_>>();
    Box::leak(values.into_boxed_slice())
}

fn leak_values<T: 'static>(values: Vec<T>) -> &'static [T] {
    Box::leak(values.into_boxed_slice())
}

fn build_help() -> &'static str {
    "ios-rust-build <command>\n\nCommands:\n  --help, -h\n  --version [--format json]\n  build --workspace-root PATH --spec PATH --target TRIPLE --target-os OS --out-dir PATH --manifest-dir PATH --format json\n"
}

fn runtime_host_triple() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        _ => None,
    }
}

/// Run a native build with injected environment, command runner, and Cargo output.
fn build_with(
    config: &IosNativeBuildConfig,
    environment: &BuildEnvironment,
    runner: &mut impl CommandRunner,
    cargo_output: &mut impl io::Write,
) -> Result<(), BuildError> {
    validate_config(config)?;
    for source in config.sources {
        writeln!(cargo_output, "cargo:rerun-if-changed={source}")
            .map_err(|error| BuildError(format!("failed to write Cargo build output: {error}")))?;
    }
    for header in config.headers {
        let manifest_dir = environment.manifest_dir.as_deref().ok_or_else(|| {
            BuildError(
                "Cargo must set CARGO_MANIFEST_DIR when native headers are configured".to_owned(),
            )
        })?;
        writeln!(
            cargo_output,
            "cargo:rerun-if-changed={}",
            manifest_dir.join(header).display()
        )
        .map_err(|error| BuildError(format!("failed to write Cargo build output: {error}")))?;
    }
    for name in RERUN_ENV {
        writeln!(cargo_output, "cargo:rerun-if-env-changed={name}")
            .map_err(|error| BuildError(format!("failed to write Cargo build output: {error}")))?;
    }

    if environment.target_os.as_deref() != Some("ios") {
        return Ok(());
    }

    let cargo_target = environment
        .target
        .as_deref()
        .ok_or_else(|| BuildError("Cargo must set TARGET".to_owned()))?;
    let target = IosTarget::from_cargo_target(cargo_target).ok_or_else(|| {
        BuildError(format!(
            "unsupported iOS {} target: {cargo_target}",
            config.capability
        ))
    })?;
    if !config.targets.contains(&target)
        || !config
            .supported_architectures
            .contains(&target.architecture())
    {
        return Err(BuildError(format!(
            "unsupported iOS {} target: {cargo_target}",
            config.capability
        )));
    }

    let sdk_path_result = run_command(
        runner,
        "xcrun",
        &[
            "--sdk".into(),
            target.sdk().into(),
            "--show-sdk-path".into(),
        ],
        OutputMode::Capture,
        "xcrun SDK lookup",
    )?;
    let sdk_path = String::from_utf8(sdk_path_result.stdout).map_err(|error| {
        BuildError(format!(
            "xcrun SDK lookup returned non-UTF-8 output: {error}"
        ))
    })?;
    let sdk_path = sdk_path.trim();
    if sdk_path.is_empty() {
        return Err(BuildError(
            "xcrun SDK lookup returned an empty SDK path".to_owned(),
        ));
    }

    let out_dir = environment
        .out_dir
        .as_deref()
        .ok_or_else(|| BuildError("Cargo must set OUT_DIR".to_owned()))?;
    let archive = out_dir.join(format!("lib{}.a", config.library));
    let clang_target = target.clang_target(config.minimum_os);
    let include_dirs = native_include_dirs(config, environment)?;

    let mut objects = Vec::with_capacity(config.sources.len());
    for source in config.sources {
        let stem = Path::new(source)
            .file_stem()
            .ok_or_else(|| BuildError(format!("invalid native source path: {source}")))?;
        let object = out_dir.join(stem).with_extension("o");
        if objects.contains(&object) {
            return Err(BuildError(format!(
                "native sources map to a duplicate object path: {}",
                object.display()
            )));
        }
        objects.push(object.clone());
        let mut clang_args = vec![
            "--sdk".into(),
            target.sdk().into(),
            "clang".into(),
            "-target".into(),
            clang_target.clone().into(),
            "-isysroot".into(),
            sdk_path.into(),
        ];
        for include_dir in &include_dirs {
            clang_args.push("-I".into());
            clang_args.push(include_dir.as_os_str().to_owned());
        }
        clang_args.extend(config.compiler_options.iter().map(OsString::from));
        clang_args.push("-c".into());
        clang_args.push((*source).into());
        clang_args.push("-o".into());
        clang_args.push(object.into_os_string());
        run_command(
            runner,
            "xcrun",
            &clang_args,
            OutputMode::Inherit,
            config.compile_description,
        )?;
    }

    let mut archive_args = vec![
        "--sdk".into(),
        target.sdk().into(),
        "ar".into(),
        "crs".into(),
        archive.as_os_str().to_owned(),
    ];
    archive_args.extend(objects.iter().map(|object| object.as_os_str().to_owned()));
    run_command(
        runner,
        "xcrun",
        &archive_args,
        OutputMode::Inherit,
        config.archive_description,
    )?;

    writeln!(
        cargo_output,
        "cargo:rustc-link-search=native={}",
        out_dir.display()
    )
    .map_err(|error| BuildError(format!("failed to write Cargo build output: {error}")))?;
    writeln!(
        cargo_output,
        "cargo:rustc-link-lib=static={}",
        config.library
    )
    .map_err(|error| BuildError(format!("failed to write Cargo build output: {error}")))?;
    for framework in config.frameworks {
        writeln!(cargo_output, "cargo:rustc-link-lib=framework={framework}")
            .map_err(|error| BuildError(format!("failed to write Cargo build output: {error}")))?;
    }
    Ok(())
}

fn native_include_dirs(
    config: &IosNativeBuildConfig,
    environment: &BuildEnvironment,
) -> Result<Vec<PathBuf>, BuildError> {
    if config.headers.is_empty() {
        return Ok(Vec::new());
    }
    let manifest_dir = environment.manifest_dir.as_deref().ok_or_else(|| {
        BuildError(
            "Cargo must set CARGO_MANIFEST_DIR when native headers are configured".to_owned(),
        )
    })?;
    let mut directories = Vec::new();
    for header in config.headers {
        let parent = Path::new(header).parent().ok_or_else(|| {
            BuildError(format!("native header has no parent directory: {header}"))
        })?;
        let directory = manifest_dir.join(parent);
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }
    Ok(directories)
}

fn run_command(
    runner: &mut impl CommandRunner,
    program: &str,
    args: &[OsString],
    output_mode: OutputMode,
    description: &str,
) -> Result<CommandResult, BuildError> {
    let result = runner
        .run(OsStr::new(program), args, output_mode)
        .map_err(|error| BuildError(format!("failed to run {description}: {error}")))?;
    if !result.success {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let detail = if stderr.is_empty() {
            String::new()
        } else {
            format!(": {stderr}")
        };
        return Err(BuildError(format!(
            "{description} failed with {}{detail}",
            result.status
        )));
    }
    Ok(result)
}

fn validate_config(config: &IosNativeBuildConfig) -> Result<(), BuildError> {
    if config.capability.is_empty()
        || config.capability.contains('\n')
        || config.capability.contains('\r')
    {
        return Err(BuildError("capability name must not be empty".to_owned()));
    }
    validate_link_name("static library", config.library)?;
    if config.sources.is_empty() {
        return Err(BuildError("at least one C source is required".to_owned()));
    }
    let mut source_paths = HashSet::new();
    let mut object_stems = HashSet::new();
    for source in config.sources {
        let path = Path::new(source);
        if source.is_empty()
            || path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
            || source.contains('\n')
            || source.contains('\r')
        {
            return Err(BuildError(format!("invalid native source path: {source}")));
        }
        if path.extension() != Some(OsStr::new("c")) || path.file_stem().is_none() {
            return Err(BuildError(format!(
                "native source must be a .c file: {source}"
            )));
        }
        if !source_paths.insert(*source) {
            return Err(BuildError(format!("duplicate native source: {source}")));
        }
        if !object_stems.insert(path.file_stem().expect("checked source stem")) {
            return Err(BuildError(format!(
                "native sources have duplicate file stems: {source}"
            )));
        }
    }
    let mut header_paths = HashSet::new();
    for header in config.headers {
        let path = Path::new(header);
        if header.is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, Component::RootDir | Component::Prefix(_)))
            || header.contains('\n')
            || header.contains('\r')
        {
            return Err(BuildError(format!("invalid native header path: {header}")));
        }
        if path.extension() != Some(OsStr::new("h")) || path.file_stem().is_none() {
            return Err(BuildError(format!(
                "native header must be a .h file: {header}"
            )));
        }
        if !header_paths.insert(*header) {
            return Err(BuildError(format!("duplicate native header: {header}")));
        }
    }
    if config.frameworks.is_empty() {
        return Err(BuildError(
            "at least one public framework dependency is required".to_owned(),
        ));
    }
    let mut frameworks = HashSet::new();
    for framework in config.frameworks {
        validate_link_name("framework", framework)?;
        if !frameworks.insert(*framework) {
            return Err(BuildError(format!(
                "duplicate framework dependency: {framework}"
            )));
        }
    }
    if config.minimum_os.major == 0 || config.minimum_os.minor > 99 {
        return Err(BuildError(
            "invalid iOS minimum deployment version".to_owned(),
        ));
    }
    if config.targets.is_empty() || config.supported_architectures.is_empty() {
        return Err(BuildError(
            "at least one iOS target and architecture are required".to_owned(),
        ));
    }
    let mut architectures = HashSet::new();
    for architecture in config.supported_architectures {
        if !architectures.insert(*architecture) {
            return Err(BuildError("duplicate supported architecture".to_owned()));
        }
    }
    let mut targets = HashSet::new();
    for target in config.targets {
        if !targets.insert(*target) {
            return Err(BuildError("duplicate iOS target".to_owned()));
        }
        if !architectures.contains(&target.architecture()) {
            return Err(BuildError(format!(
                "target {} uses an unsupported architecture",
                target.cargo_target()
            )));
        }
    }
    for architecture in &architectures {
        if !targets
            .iter()
            .any(|target| target.architecture() == *architecture)
        {
            return Err(BuildError(
                "supported architecture has no Cargo target".to_owned(),
            ));
        }
    }
    for option in config.compiler_options {
        if option.is_empty()
            || option.starts_with("-target")
            || option.starts_with("-isysroot")
            || *option == "-c"
            || *option == "-o"
            || option.contains('\n')
            || option.contains('\r')
        {
            return Err(BuildError(format!(
                "invalid explicit compiler option: {option}"
            )));
        }
    }
    Ok(())
}

fn validate_link_name(kind: &str, name: &str) -> Result<(), BuildError> {
    let mut chars = name.chars();
    let valid_first = chars
        .next()
        .map(|character| character.is_ascii_alphabetic() || character == '_')
        .unwrap_or(false);
    if !valid_first || !chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(BuildError(format!("invalid {kind} name: {name}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    const ARM64: &[Architecture] = &[Architecture::Arm64];
    const ARM64_AND_X86_64: &[Architecture] = &[Architecture::Arm64, Architecture::X86_64];
    const DEVICE_AND_ARM64_SIM: &[IosTarget] = &[IosTarget::DeviceArm64, IosTarget::SimulatorArm64];
    const ALL_PHOTOGRAMMETRY_TARGETS: &[IosTarget] = &[
        IosTarget::DeviceArm64,
        IosTarget::SimulatorArm64,
        IosTarget::SimulatorX86_64,
    ];
    const OPTIONS: &[&str] = &["-std=c11", "-Wall", "-Wextra", "-Werror"];

    const ACTIVITYKIT: IosNativeBuildConfig = IosNativeBuildConfig {
        capability: "ActivityKit",
        library: "activitykit_status",
        sources: &["native/activitykit_status.c"],
        headers: &["../../../interop/swift-abi-core/include/swift_abi_runtime.h"],
        frameworks: &["ActivityKit"],
        minimum_os: DeploymentTarget {
            major: 16,
            minor: 1,
        },
        supported_architectures: ARM64,
        targets: DEVICE_AND_ARM64_SIM,
        compiler_options: OPTIONS,
        compile_description: "ActivityKit C swiftcall bridge compile",
        archive_description: "ActivityKit static archive creation",
    };

    const ALARMKIT: IosNativeBuildConfig = IosNativeBuildConfig {
        capability: "AlarmKit",
        library: "alarmkit_status",
        sources: &["native/alarmkit_status.c"],
        headers: &["../../../interop/swift-abi-core/include/swift_abi_runtime.h"],
        frameworks: &["AlarmKit"],
        minimum_os: DeploymentTarget {
            major: 26,
            minor: 0,
        },
        supported_architectures: ARM64,
        targets: DEVICE_AND_ARM64_SIM,
        compiler_options: OPTIONS,
        compile_description: "AlarmKit C swiftcall and enum-witness bridge compile",
        archive_description: "AlarmKit static archive creation",
    };

    const PHOTOGRAMMETRY: IosNativeBuildConfig = IosNativeBuildConfig {
        capability: "photogrammetry",
        library: "photogrammetry_status",
        sources: &["native/photogrammetry_status.c"],
        headers: &["../../../interop/swift-abi-core/include/swift_abi_runtime.h"],
        frameworks: &["RealityFoundation"],
        minimum_os: DeploymentTarget {
            major: 17,
            minor: 0,
        },
        supported_architectures: ARM64_AND_X86_64,
        targets: ALL_PHOTOGRAMMETRY_TARGETS,
        compiler_options: OPTIONS,
        compile_description: "RealityFoundation C swiftcall thunk compile",
        archive_description: "RealityFoundation static archive creation",
    };

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct RecordedCommand {
        program: OsString,
        args: Vec<OsString>,
        output_mode: OutputMode,
    }

    #[derive(Default)]
    struct FakeRunner {
        commands: Vec<RecordedCommand>,
        results: VecDeque<CommandResult>,
    }

    impl FakeRunner {
        fn with_results(results: impl IntoIterator<Item = CommandResult>) -> Self {
            Self {
                commands: Vec::new(),
                results: results.into_iter().collect(),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(
            &mut self,
            program: &OsStr,
            args: &[OsString],
            output_mode: OutputMode,
        ) -> io::Result<CommandResult> {
            self.commands.push(RecordedCommand {
                program: program.to_owned(),
                args: args.to_vec(),
                output_mode,
            });
            Ok(self.results.pop_front().unwrap_or_else(ok_result))
        }
    }

    fn ok_result() -> CommandResult {
        CommandResult {
            success: true,
            status: "exit status: 0".to_owned(),
            stdout: b"/Xcode SDKs/iPhoneOS.sdk\n".to_vec(),
            stderr: Vec::new(),
        }
    }

    fn command_result(success: bool, stdout: &[u8], stderr: &[u8]) -> CommandResult {
        CommandResult {
            success,
            status: if success {
                "exit status: 0"
            } else {
                "exit status: 1"
            }
            .to_owned(),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }

    fn environment(target: &str, out_dir: &str) -> BuildEnvironment {
        BuildEnvironment {
            target_os: Some("ios".to_owned()),
            target: Some(target.to_owned()),
            out_dir: Some(PathBuf::from(out_dir)),
            manifest_dir: Some(PathBuf::from("/repo/platform/ios/ios-activitykit-status")),
        }
    }

    fn run_fake(
        config: &IosNativeBuildConfig,
        target: &str,
        runner: &mut FakeRunner,
    ) -> (Result<(), BuildError>, String) {
        let mut output = Vec::new();
        let result = build_with(
            config,
            &environment(target, "/tmp/build/out"),
            runner,
            &mut output,
        );
        (
            result,
            String::from_utf8(output).expect("Cargo output is UTF-8"),
        )
    }

    fn args(command: &RecordedCommand) -> Vec<String> {
        command
            .args
            .iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn maps_each_pilot_target_to_its_baseline_sdk_and_clang_target() {
        let cases = [
            (
                &ACTIVITYKIT,
                "aarch64-apple-ios",
                "iphoneos",
                "arm64-apple-ios16.1",
            ),
            (
                &ACTIVITYKIT,
                "aarch64-apple-ios-sim",
                "iphonesimulator",
                "arm64-apple-ios16.1-simulator",
            ),
            (
                &ALARMKIT,
                "aarch64-apple-ios",
                "iphoneos",
                "arm64-apple-ios26.0",
            ),
            (
                &ALARMKIT,
                "aarch64-apple-ios-sim",
                "iphonesimulator",
                "arm64-apple-ios26.0-simulator",
            ),
            (
                &PHOTOGRAMMETRY,
                "aarch64-apple-ios",
                "iphoneos",
                "arm64-apple-ios17.0",
            ),
            (
                &PHOTOGRAMMETRY,
                "aarch64-apple-ios-sim",
                "iphonesimulator",
                "arm64-apple-ios17.0-simulator",
            ),
            (
                &PHOTOGRAMMETRY,
                "x86_64-apple-ios",
                "iphonesimulator",
                "x86_64-apple-ios17.0-simulator",
            ),
        ];
        for (config, cargo_target, sdk, clang_target) in cases {
            let mut runner = FakeRunner::with_results([
                command_result(true, b"/SDK path/with spaces\n", b""),
                command_result(true, b"", b""),
                command_result(true, b"", b""),
            ]);
            let (result, output) = run_fake(config, cargo_target, &mut runner);
            result.expect("pilot target must build");
            assert_eq!(runner.commands.len(), 3);
            assert_eq!(args(&runner.commands[0]), ["--sdk", sdk, "--show-sdk-path"]);
            assert_eq!(
                args(&runner.commands[1])[0..5],
                ["--sdk", sdk, "clang", "-target", clang_target]
            );
            assert_eq!(runner.commands[0].output_mode, OutputMode::Capture);
            assert_eq!(runner.commands[1].output_mode, OutputMode::Inherit);
            assert_eq!(runner.commands[2].output_mode, OutputMode::Inherit);
            assert!(output.contains(&format!("cargo:rustc-link-lib=static={}\n", config.library)));
            for framework in config.frameworks {
                assert!(output.contains(&format!("cargo:rustc-link-lib=framework={framework}\n")));
            }
        }
    }

    #[test]
    fn emits_deterministic_rerun_and_link_directives() {
        let mut runner = FakeRunner::with_results([
            command_result(true, b"/SDK/iPhoneOS.sdk\n", b""),
            command_result(true, b"", b""),
            command_result(true, b"", b""),
        ]);
        let (result, output) = run_fake(&ACTIVITYKIT, "aarch64-apple-ios", &mut runner);
        result.expect("fake build must succeed");
        assert_eq!(
            output,
            concat!(
                "cargo:rerun-if-changed=native/activitykit_status.c\n",
                "cargo:rerun-if-changed=/repo/platform/ios/ios-activitykit-status/../../../interop/swift-abi-core/include/swift_abi_runtime.h\n",
                "cargo:rerun-if-env-changed=SDKROOT\n",
                "cargo:rerun-if-env-changed=DEVELOPER_DIR\n",
                "cargo:rustc-link-search=native=/tmp/build/out\n",
                "cargo:rustc-link-lib=static=activitykit_status\n",
                "cargo:rustc-link-lib=framework=ActivityKit\n",
            )
        );
    }

    #[test]
    fn missing_manifest_dir_rejects_configured_headers_before_running_tools() {
        let mut env = environment("aarch64-apple-ios", "/tmp/build/out");
        env.manifest_dir = None;
        let mut runner = FakeRunner::default();
        let mut output = Vec::new();
        let error = build_with(&ACTIVITYKIT, &env, &mut runner, &mut output)
            .unwrap_err()
            .to_string();
        assert!(error.contains("CARGO_MANIFEST_DIR"));
        assert!(runner.commands.is_empty());
    }

    #[test]
    fn non_ios_targets_emit_rerun_directives_without_native_commands() {
        let mut runner = FakeRunner::default();
        let mut output = Vec::new();
        build_with(
            &ACTIVITYKIT,
            &BuildEnvironment {
                target_os: Some("macos".to_owned()),
                target: None,
                out_dir: None,
                manifest_dir: Some(PathBuf::from("/repo/platform/ios/ios-activitykit-status")),
            },
            &mut runner,
            &mut output,
        )
        .expect("non-iOS build scripts are no-ops");
        assert!(runner.commands.is_empty());
        assert_eq!(String::from_utf8(output).unwrap().lines().count(), 4);
    }

    #[test]
    fn reports_missing_sdk_and_rejects_empty_sdk_path() {
        let mut failed_lookup =
            FakeRunner::with_results([command_result(false, b"", b"unable to find SDK")]);
        let (result, _) = run_fake(&ACTIVITYKIT, "aarch64-apple-ios", &mut failed_lookup);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("xcrun SDK lookup failed")
        );

        let mut empty_lookup = FakeRunner::with_results([command_result(true, b"\n", b"")]);
        let (result, _) = run_fake(&ACTIVITYKIT, "aarch64-apple-ios", &mut empty_lookup);
        assert!(result.unwrap_err().to_string().contains("empty SDK path"));
        assert_eq!(empty_lookup.commands.len(), 1);
    }

    #[test]
    fn reports_clang_and_archive_failures_with_phase_context() {
        let mut clang_failure = FakeRunner::with_results([
            command_result(true, b"/SDK\n", b""),
            command_result(false, b"", b"clang rejected source"),
        ]);
        let (result, _) = run_fake(&ALARMKIT, "aarch64-apple-ios", &mut clang_failure);
        let error = result.unwrap_err().to_string();
        assert!(error.contains("AlarmKit C swiftcall and enum-witness bridge compile failed"));
        assert!(error.contains("clang rejected source"));

        let mut archive_failure = FakeRunner::with_results([
            command_result(true, b"/SDK\n", b""),
            command_result(true, b"", b""),
            command_result(false, b"", b"ar rejected object"),
        ]);
        let (result, _) = run_fake(&ALARMKIT, "aarch64-apple-ios", &mut archive_failure);
        let error = result.unwrap_err().to_string();
        assert!(error.contains("AlarmKit static archive creation failed"));
        assert!(error.contains("ar rejected object"));
    }

    #[test]
    fn rejects_unsupported_targets_and_invalid_link_names() {
        let mut runner = FakeRunner::default();
        let (result, _) = run_fake(&ACTIVITYKIT, "aarch64-apple-ios-macabi", &mut runner);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("unsupported iOS ActivityKit target")
        );
        assert!(runner.commands.is_empty());

        let invalid_library = IosNativeBuildConfig {
            library: "../escape",
            ..ACTIVITYKIT
        };
        let (result, _) = run_fake(&invalid_library, "aarch64-apple-ios", &mut runner);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid static library name")
        );

        let invalid_framework = IosNativeBuildConfig {
            frameworks: &["ActivityKit\ncargo:rustc-link-lib=framework=UIKit"],
            ..ACTIVITYKIT
        };
        let (result, _) = run_fake(&invalid_framework, "aarch64-apple-ios", &mut runner);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid framework name")
        );

        let invalid_header = IosNativeBuildConfig {
            headers: &["native/not-a-header.txt"],
            ..ACTIVITYKIT
        };
        let (result, _) = run_fake(&invalid_header, "aarch64-apple-ios", &mut runner);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("native header must be a .h file")
        );
    }

    #[test]
    fn preserves_sdk_source_and_output_paths_with_spaces_as_single_arguments() {
        let config = IosNativeBuildConfig {
            sources: &["native files/activitykit status.c"],
            ..ACTIVITYKIT
        };
        let mut runner = FakeRunner::with_results([
            command_result(true, b"/Xcode SDKs/iPhoneOS.sdk\n", b""),
            command_result(true, b"", b""),
            command_result(true, b"", b""),
        ]);
        let mut cargo_output = Vec::new();
        let mut environment = environment("aarch64-apple-ios", "/tmp/build directory/out files");
        environment.manifest_dir = Some(PathBuf::from(
            "/repo directory/platform/ios/ios-activitykit-status",
        ));
        build_with(&config, &environment, &mut runner, &mut cargo_output)
            .expect("spaces in filesystem paths must not split command arguments");
        let clang_args = args(&runner.commands[1]);
        assert_eq!(clang_args[6], "/Xcode SDKs/iPhoneOS.sdk");
        assert_eq!(
            clang_args[8],
            "/repo directory/platform/ios/ios-activitykit-status/../../../interop/swift-abi-core/include"
        );
        assert_eq!(clang_args[14], "native files/activitykit status.c");
        assert_eq!(
            clang_args[16],
            "/tmp/build directory/out files/activitykit status.o"
        );
        assert_eq!(
            String::from_utf8(cargo_output)
                .unwrap()
                .lines()
                .rev()
                .nth(2),
            Some("cargo:rustc-link-search=native=/tmp/build directory/out files")
        );
        let archive_args = args(&runner.commands[2]);
        assert_eq!(
            archive_args[4],
            "/tmp/build directory/out files/libactivitykit_status.a"
        );
        assert_eq!(
            archive_args[5],
            "/tmp/build directory/out files/activitykit status.o"
        );
    }
}

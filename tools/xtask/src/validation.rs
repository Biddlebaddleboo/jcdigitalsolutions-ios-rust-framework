use crate::json_string;
#[cfg(test)]
use crate::root;
use serde::Deserialize;
use std::collections::{BTreeSet, VecDeque};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_VALIDATION_SPEC_BYTES: usize = 262_144;
const MAX_RESULT_JSON_BYTES: usize = 1_048_576;
const MAX_RESULT_COUNT: usize = 2_048;
const MAX_GATE_DETAIL_BYTES: usize = 4_096;

#[derive(Clone, Copy, Debug)]
struct Target {
    triple: &'static str,
    sdk: &'static str,
    minimum_os: &'static str,
}

#[derive(Clone, Copy)]
struct Guard {
    path: &'static str,
    text: &'static str,
    must_exist: bool,
    message: &'static str,
}

#[derive(Clone, Copy)]
struct Pilot {
    id: &'static str,
    package: &'static str,
    manifest: &'static str,
    check_script: &'static str,
    targets: &'static [Target],
    abi_targets: &'static [Target],
    expected_imports: &'static [&'static str],
    abi_scripts: &'static [&'static str],
    shell_scripts: &'static [&'static str],
    docs: &'static [&'static str],
    scoped_files: &'static [&'static str],
    source_dirs: &'static [&'static str],
    guards: &'static [Guard],
    adapters: &'static [PythonAdapter],
    offline: bool,
    fmt_all: bool,
    clippy_all_targets: bool,
    clippy_lib: bool,
    rustdoc_warnings: bool,
    cargo_tree: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationFileSpec {
    schema_version: u32,
    dependency_edges: Vec<[String; 2]>,
    path_rules: Vec<PathRuleSpec>,
    pilots: Vec<PilotSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PathRuleSpec {
    path: String,
    node: Option<String>,
    global: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PilotSpec {
    id: String,
    package: String,
    manifest: String,
    check_script: String,
    targets: Vec<TargetSpec>,
    abi_targets: Vec<TargetSpec>,
    expected_imports: Vec<String>,
    abi_scripts: Vec<String>,
    shell_scripts: Vec<String>,
    docs: Vec<String>,
    scoped_files: Vec<String>,
    source_dirs: Vec<String>,
    guards: Vec<GuardSpec>,
    #[serde(default)]
    adapters: Vec<PythonAdapterSpec>,
    offline: bool,
    fmt_all: bool,
    clippy_all_targets: bool,
    clippy_lib: bool,
    rustdoc_warnings: bool,
    cargo_tree: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetSpec {
    triple: String,
    sdk: String,
    minimum_os: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GuardSpec {
    path: String,
    text: String,
    must_exist: bool,
    message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PythonAdapterSpec {
    id: String,
    interpreter: String,
    minimum_version: String,
    script: String,
    input_paths: Vec<String>,
    required: bool,
    evidence_type: String,
    deadline_seconds: u64,
    stdout_limit_bytes: usize,
    stderr_limit_bytes: usize,
    temporary_space_bytes: u64,
}

#[derive(Clone, Copy)]
struct PythonAdapter {
    id: &'static str,
    interpreter: &'static str,
    minimum_version: &'static str,
    script: &'static str,
    input_paths: &'static [&'static str],
    required: bool,
    evidence_type: &'static str,
    deadline_seconds: u64,
    stdout_limit_bytes: usize,
    stderr_limit_bytes: usize,
    temporary_space_bytes: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PythonAdapterResponse {
    schema_version: u32,
    status: String,
    detail: String,
}

struct ValidationConfig {
    pilots: Vec<Pilot>,
    graph: DependencyGraph,
    path_rules: Vec<PathRuleSpec>,
}

impl PilotSpec {
    fn into_pilot(self) -> Pilot {
        Pilot {
            id: leak_string(self.id),
            package: leak_string(self.package),
            manifest: leak_string(self.manifest),
            check_script: leak_string(self.check_script),
            targets: leak_targets(self.targets),
            abi_targets: leak_targets(self.abi_targets),
            expected_imports: leak_strings(self.expected_imports),
            abi_scripts: leak_strings(self.abi_scripts),
            shell_scripts: leak_strings(self.shell_scripts),
            docs: leak_strings(self.docs),
            scoped_files: leak_strings(self.scoped_files),
            source_dirs: leak_strings(self.source_dirs),
            guards: leak_guards(self.guards),
            adapters: leak_adapters(self.adapters),
            offline: self.offline,
            fmt_all: self.fmt_all,
            clippy_all_targets: self.clippy_all_targets,
            clippy_lib: self.clippy_lib,
            rustdoc_warnings: self.rustdoc_warnings,
            cargo_tree: self.cargo_tree,
        }
    }
}

fn load_validation_config(root: &Path, spec_path: &Path) -> Result<ValidationConfig, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("invalid workspace root: {error}"))?;
    let spec_path = if spec_path.is_absolute() {
        spec_path.to_path_buf()
    } else {
        root.join(spec_path)
    };
    let spec_path = spec_path
        .canonicalize()
        .map_err(|error| format!("cannot read validation spec: {error}"))?;
    if !spec_path.starts_with(&root) {
        return Err("validation spec is outside the workspace root".into());
    }
    let bytes = std::fs::read(&spec_path).map_err(|error| {
        format!(
            "cannot read validation spec {}: {error}",
            spec_path.display()
        )
    })?;
    if bytes.len() > MAX_VALIDATION_SPEC_BYTES {
        return Err(format!(
            "validation spec exceeds the {} byte limit",
            MAX_VALIDATION_SPEC_BYTES
        ));
    }
    let file: ValidationFileSpec = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid validation spec {}: {error}", spec_path.display()))?;
    if file.schema_version != 1 {
        return Err(format!(
            "unsupported validation spec schema major {}; this tool supports 1",
            file.schema_version
        ));
    }
    if file.pilots.is_empty() {
        return Err("validation spec must define at least one pilot".into());
    }
    for rule in &file.path_rules {
        validate_relative_path(&rule.path)?;
        if rule.path.contains('\n') || rule.path.contains('\r') {
            return Err("invalid changed-path rule".into());
        }
        if let Some(node) = &rule.node {
            if !valid_id(node) {
                return Err(format!("invalid changed-path dependency node `{node}`"));
            }
        }
        if rule.global && rule.node.is_some() {
            return Err(format!(
                "global changed-path rule `{}` cannot name a node",
                rule.path
            ));
        }
        if !rule.global && rule.node.is_none() {
            return Err(format!(
                "non-global changed-path rule `{}` must name a node",
                rule.path
            ));
        }
    }
    let mut ids = BTreeSet::new();
    for pilot in &file.pilots {
        if !valid_id(&pilot.id) || !valid_id(&pilot.package) {
            return Err(format!("invalid pilot or package ID `{}`", pilot.id));
        }
        if !ids.insert(pilot.id.as_str()) {
            return Err(format!("duplicate pilot ID `{}`", pilot.id));
        }
        if pilot.targets.is_empty() || pilot.abi_targets.is_empty() {
            return Err(format!(
                "pilot `{}` must declare compile and ABI targets",
                pilot.id
            ));
        }
        for path in std::iter::once(&pilot.manifest)
            .chain(std::iter::once(&pilot.check_script))
            .chain(pilot.abi_scripts.iter())
            .chain(pilot.shell_scripts.iter())
            .chain(pilot.docs.iter())
            .chain(pilot.scoped_files.iter())
            .chain(pilot.source_dirs.iter())
            .chain(pilot.guards.iter().map(|guard| &guard.path))
        {
            validate_relative_path(path)?;
        }
        for target in pilot.targets.iter().chain(pilot.abi_targets.iter()) {
            if !matches!(
                target.triple.as_str(),
                "aarch64-apple-ios" | "aarch64-apple-ios-sim" | "x86_64-apple-ios"
            ) {
                return Err(format!(
                    "pilot `{}` has unsupported target `{}`",
                    pilot.id, target.triple
                ));
            }
            let expected_sdk = if target.triple == "aarch64-apple-ios" {
                "iphoneos"
            } else {
                "iphonesimulator"
            };
            if target.sdk != expected_sdk {
                return Err(format!(
                    "target `{}` requires SDK `{expected_sdk}`",
                    target.triple
                ));
            }
            if !target
                .minimum_os
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
            {
                return Err(format!("invalid deployment floor `{}`", target.minimum_os));
            }
        }
        for guard in &pilot.guards {
            if guard.path.is_empty()
                || guard.message.is_empty()
                || guard.text.contains('\n')
                || guard.text.contains('\r')
            {
                return Err(format!("invalid source guard in pilot `{}`", pilot.id));
            }
        }
        let mut adapter_ids = BTreeSet::new();
        for adapter in &pilot.adapters {
            if !valid_id(&adapter.id) || !adapter_ids.insert(adapter.id.as_str()) {
                return Err(format!("invalid or duplicate adapter ID `{}`", adapter.id));
            }
            if adapter.interpreter != "python3"
                || !valid_version(&adapter.minimum_version)
                || !valid_id(&adapter.evidence_type)
                || adapter.input_paths.is_empty()
                || !(1..=300).contains(&adapter.deadline_seconds)
                || !(1..=1_048_576).contains(&adapter.stdout_limit_bytes)
                || !(1..=1_048_576).contains(&adapter.stderr_limit_bytes)
                || !(1..=536_870_912).contains(&adapter.temporary_space_bytes)
            {
                return Err(format!("invalid adapter contract `{}`", adapter.id));
            }
            validate_relative_path(&adapter.script)?;
            for path in &adapter.input_paths {
                validate_relative_path(path)?;
            }
        }
    }
    let mut edges = Vec::with_capacity(file.dependency_edges.len());
    for [consumer, dependency] in file.dependency_edges {
        if !valid_id(&consumer) || !valid_id(&dependency) {
            return Err("invalid dependency graph node ID".into());
        }
        edges.push((consumer, dependency));
    }
    Ok(ValidationConfig {
        pilots: file.pilots.into_iter().map(PilotSpec::into_pilot).collect(),
        graph: DependencyGraph { edges },
        path_rules: file.path_rules,
    })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() >= 2
        && parts.len() <= 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn validate_relative_path(value: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\n')
        || value.contains('\r')
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!("unsafe workspace-relative path `{value}`"));
    }
    Ok(())
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn leak_strings(values: Vec<String>) -> &'static [&'static str] {
    let values = values.into_iter().map(leak_string).collect::<Vec<_>>();
    Box::leak(values.into_boxed_slice())
}

fn leak_targets(values: Vec<TargetSpec>) -> &'static [Target] {
    let values = values
        .into_iter()
        .map(|value| Target {
            triple: leak_string(value.triple),
            sdk: leak_string(value.sdk),
            minimum_os: leak_string(value.minimum_os),
        })
        .collect::<Vec<_>>();
    Box::leak(values.into_boxed_slice())
}

fn leak_guards(values: Vec<GuardSpec>) -> &'static [Guard] {
    let values = values
        .into_iter()
        .map(|value| Guard {
            path: leak_string(value.path),
            text: leak_string(value.text),
            must_exist: value.must_exist,
            message: leak_string(value.message),
        })
        .collect::<Vec<_>>();
    Box::leak(values.into_boxed_slice())
}

fn leak_adapters(values: Vec<PythonAdapterSpec>) -> &'static [PythonAdapter] {
    let values = values
        .into_iter()
        .map(|value| PythonAdapter {
            id: leak_string(value.id),
            interpreter: leak_string(value.interpreter),
            minimum_version: leak_string(value.minimum_version),
            script: leak_string(value.script),
            input_paths: leak_strings(value.input_paths),
            required: value.required,
            evidence_type: leak_string(value.evidence_type),
            deadline_seconds: value.deadline_seconds,
            stdout_limit_bytes: value.stdout_limit_bytes,
            stderr_limit_bytes: value.stderr_limit_bytes,
            temporary_space_bytes: value.temporary_space_bytes,
        })
        .collect::<Vec<_>>();
    Box::leak(values.into_boxed_slice())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandResult {
    success: bool,
    stdout: String,
    stderr: String,
}

trait CommandRunner {
    fn run(
        &mut self,
        program: &str,
        args: &[String],
        env: &[(String, String)],
    ) -> io::Result<CommandResult>;
}

struct ProcessRunner {
    workspace_root: PathBuf,
}

impl CommandRunner for ProcessRunner {
    fn run(
        &mut self,
        program: &str,
        args: &[String],
        env: &[(String, String)],
    ) -> io::Result<CommandResult> {
        let mut command = Command::new(program);
        command.args(args).current_dir(&self.workspace_root);
        for (name, value) in env {
            command.env(name, value);
        }
        let output = command.output()?;
        Ok(CommandResult::from_output(output))
    }
}

impl CommandResult {
    fn from_output(output: Output) -> Self {
        Self {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

fn result_text(output: &CommandResult) -> String {
    let stdout = output.stdout.trim();
    let stderr = output.stderr.trim();
    let text = match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout.to_owned(),
        (true, false) => stderr.to_owned(),
        (true, true) => String::new(),
    };
    bounded_text(&text, MAX_GATE_DETAIL_BYTES)
}

fn bounded_text(value: &str, maximum: usize) -> String {
    if value.len() <= maximum {
        return value.to_owned();
    }
    let mut end = maximum;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… [truncated]", &value[..end])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    Passed,
    Failed,
    OptionalFailed,
    Skipped,
    Blocked,
    OptionalBlocked,
}

#[derive(Clone, Debug)]
struct GateResult {
    capability: String,
    gate: String,
    phase: String,
    outcome: Outcome,
    detail: String,
}

#[derive(Clone, Debug)]
struct Step {
    name: String,
    phase: String,
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    target: Option<Target>,
}

fn cargo_step(name: &str, phase: &str, args: &[&str], target: Option<Target>) -> Step {
    let mut all = vec!["+1.94.1".to_owned()];
    all.extend(args.iter().map(|arg| (*arg).to_owned()));
    Step {
        name: name.to_owned(),
        phase: phase.to_owned(),
        program: "cargo".to_owned(),
        args: all,
        env: Vec::new(),
        target,
    }
}

fn build_steps(pilot: &Pilot) -> Vec<Step> {
    let mut steps = Vec::new();
    if pilot.fmt_all {
        steps.push(cargo_step(
            "format",
            "format",
            &["fmt", "--all", "--", "--check"],
            None,
        ));
    } else {
        steps.push(cargo_step(
            "format",
            "format",
            &["fmt", "--manifest-path", pilot.manifest, "--", "--check"],
            None,
        ));
    }
    let lock = ["--locked"];
    let offline = if pilot.offline {
        vec!["--offline"]
    } else {
        Vec::new()
    };
    let mut common = vec!["check"];
    common.extend(lock);
    common.extend(offline.iter().copied());
    common.extend(["-p", pilot.package]);
    steps.push(cargo_step("host check", "compile", &common, None));

    let mut clippy = vec!["clippy"];
    clippy.extend(lock);
    clippy.extend(offline.iter().copied());
    if pilot.clippy_lib {
        clippy.push("--lib");
    }
    if pilot.clippy_all_targets {
        clippy.push("--all-targets");
    }
    clippy.extend(["-p", pilot.package, "--", "-D", "warnings"]);
    steps.push(cargo_step("host clippy", "clippy", &clippy, None));

    let mut doc = vec!["doc"];
    doc.extend(lock);
    doc.extend(offline.iter().copied());
    doc.extend(["--no-deps", "-p", pilot.package]);
    let mut host_doc = cargo_step("host rustdoc", "rustdoc", &doc, None);
    if pilot.rustdoc_warnings {
        host_doc
            .env
            .push(("RUSTDOCFLAGS".into(), "-D warnings".into()));
    }
    steps.push(host_doc);

    for target in pilot.targets {
        let mut check = vec!["check"];
        check.extend(lock);
        check.extend(offline.iter().copied());
        check.extend(["-p", pilot.package, "--target", target.triple]);
        let target_phase = if target.sdk == "iphoneos" {
            "device-compile"
        } else {
            "simulator-compile"
        };
        steps.push(cargo_step(
            &format!("check {}", target.triple),
            target_phase,
            &check,
            Some(*target),
        ));

        let mut target_clippy = vec!["clippy"];
        target_clippy.extend(lock);
        target_clippy.extend(offline.iter().copied());
        if pilot.clippy_lib {
            target_clippy.push("--lib");
        }
        if pilot.clippy_all_targets {
            target_clippy.push("--all-targets");
        }
        target_clippy.extend([
            "-p",
            pilot.package,
            "--target",
            target.triple,
            "--",
            "-D",
            "warnings",
        ]);
        let clippy_phase = if target.sdk == "iphoneos" {
            "device-clippy"
        } else {
            "simulator-clippy"
        };
        steps.push(cargo_step(
            &format!("clippy {}", target.triple),
            clippy_phase,
            &target_clippy,
            Some(*target),
        ));

        if target.sdk == "iphoneos" {
            let mut target_doc = vec!["doc"];
            target_doc.extend(lock);
            target_doc.extend(offline.iter().copied());
            target_doc.extend(["--no-deps", "-p", pilot.package, "--target", target.triple]);
            let mut target_doc = cargo_step(
                &format!("rustdoc {}", target.triple),
                "device-rustdoc",
                &target_doc,
                Some(*target),
            );
            if pilot.rustdoc_warnings {
                target_doc
                    .env
                    .push(("RUSTDOCFLAGS".into(), "-D warnings".into()));
            }
            steps.push(target_doc);
        }
    }

    for path in pilot.shell_scripts {
        steps.push(Step {
            name: format!("shell syntax {path}"),
            phase: "shell".into(),
            program: "sh".into(),
            args: vec!["-n".into(), (*path).into()],
            env: Vec::new(),
            target: None,
        });
    }
    for path in pilot.abi_scripts {
        let phase = if path.contains("check-swiftcall") {
            "abi-oracle"
        } else {
            "link-import-scan"
        };
        steps.push(Step {
            name: format!("{phase} {path}"),
            phase: phase.into(),
            program: "sh".into(),
            args: vec![(*path).into()],
            env: Vec::new(),
            target: pilot.abi_targets.first().copied(),
        });
    }
    if pilot.cargo_tree {
        steps.push(cargo_step(
            "feature dependency graph",
            "dependencies",
            &[
                "tree",
                "--locked",
                "-p",
                pilot.package,
                "--target",
                "aarch64-apple-ios",
                "-e",
                "features",
            ],
            Some(pilot.targets[0]),
        ));
    }
    steps.push(cargo_step(
        "documentation index",
        "documentation",
        &["xtask", "docs-check"],
        None,
    ));
    steps.push(cargo_step(
        "zero Swift source",
        "policy",
        &["xtask", "zero-swift-source"],
        None,
    ));
    let mut diff = vec!["diff", "--check", "--"];
    diff.extend(pilot.scoped_files.iter().copied());
    steps.push(Step {
        name: "scoped diff whitespace".into(),
        phase: "policy".into(),
        program: "git".into(),
        args: diff.iter().map(|arg| (*arg).to_owned()).collect(),
        env: Vec::new(),
        target: None,
    });
    steps
}

fn check_target_available(runner: &mut impl CommandRunner, target: Target) -> Result<(), String> {
    let sdk = runner
        .run(
            "xcrun",
            &["--sdk".into(), target.sdk.into(), "--show-sdk-path".into()],
            &[],
        )
        .map_err(|error| format!("xcrun unavailable for {}: {error}", target.sdk))?;
    if !sdk.success || sdk.stdout.trim().is_empty() {
        return Err(format!(
            "{} SDK unavailable: {}",
            target.sdk,
            result_text(&sdk)
        ));
    }
    let installed = runner
        .run(
            "rustup",
            &[
                "+1.94.1".into(),
                "target".into(),
                "list".into(),
                "--installed".into(),
            ],
            &[],
        )
        .map_err(|error| format!("rustup target inventory unavailable: {error}"))?;
    if !installed.success {
        return Err(format!(
            "Rust target inventory unavailable: {}",
            result_text(&installed)
        ));
    }
    if !installed
        .stdout
        .lines()
        .any(|line| line.trim() == target.triple)
    {
        return Err(format!("Rust target {} is not installed", target.triple));
    }
    Ok(())
}

fn required_tools(step: &Step) -> &'static [&'static str] {
    match step.phase.as_str() {
        "abi-oracle" => &["xcrun", "swiftc", "clang", "grep", "rg"],
        "link-import-scan" if step.name.contains("activitykit") => {
            &["cargo", "otool", "awk", "sort", "diff", "grep", "nm"]
        }
        "link-import-scan" => &["xcrun", "cargo", "otool", "grep", "nm", "vtool"],
        _ => &[],
    }
}

fn check_required_tools(runner: &mut impl CommandRunner, step: &Step) -> Result<(), String> {
    for tool in required_tools(step) {
        let check = runner
            .run(
                "sh",
                &["-c".into(), format!("command -v {tool} >/dev/null 2>&1")],
                &[],
            )
            .map_err(|error| format!("cannot check required tool `{tool}`: {error}"))?;
        if !check.success {
            return Err(format!("required tool `{tool}` is unavailable"));
        }
    }
    Ok(())
}

fn run_step(pilot: &Pilot, step: &Step, runner: &mut impl CommandRunner) -> GateResult {
    if let Some(target) = step.target {
        let required = if matches!(step.phase.as_str(), "abi-oracle" | "link-import-scan") {
            pilot.abi_targets
        } else {
            std::slice::from_ref(&target)
        };
        for target in required {
            if let Err(reason) = check_target_available(runner, *target) {
                return GateResult {
                    capability: pilot.id.into(),
                    gate: step.name.clone(),
                    phase: step.phase.clone(),
                    outcome: Outcome::Skipped,
                    detail: reason,
                };
            }
        }
    }
    if let Err(reason) = check_required_tools(runner, step) {
        return GateResult {
            capability: pilot.id.into(),
            gate: step.name.clone(),
            phase: step.phase.clone(),
            outcome: Outcome::Skipped,
            detail: reason,
        };
    }
    match runner.run(&step.program, &step.args, &step.env) {
        Ok(output) if output.success => GateResult {
            capability: pilot.id.into(),
            gate: step.name.clone(),
            phase: step.phase.clone(),
            outcome: Outcome::Passed,
            detail: String::new(),
        },
        Ok(output) => GateResult {
            capability: pilot.id.into(),
            gate: step.name.clone(),
            phase: step.phase.clone(),
            outcome: Outcome::Failed,
            detail: result_text(&output),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => GateResult {
            capability: pilot.id.into(),
            gate: step.name.clone(),
            phase: step.phase.clone(),
            outcome: Outcome::Skipped,
            detail: format!("required tool {} not found: {error}", step.program),
        },
        Err(error) => GateResult {
            capability: pilot.id.into(),
            gate: step.name.clone(),
            phase: step.phase.clone(),
            outcome: Outcome::Failed,
            detail: format!("run {}: {error}", step.program),
        },
    }
}

fn source_guards(pilot: &Pilot, workspace_root: &Path) -> Vec<GateResult> {
    pilot
        .guards
        .iter()
        .map(|guard| {
            let content = std::fs::read_to_string(workspace_root.join(guard.path));
            let (ok, detail) = match content {
                Ok(content) => {
                    let found = content.contains(guard.text);
                    let ok = found == guard.must_exist;
                    (
                        ok,
                        if ok {
                            String::new()
                        } else {
                            guard.message.to_owned()
                        },
                    )
                }
                Err(error) => (false, format!("read {}: {error}", guard.path)),
            };
            GateResult {
                capability: pilot.id.into(),
                gate: format!("source guard {}", guard.path),
                phase: "source-policy".into(),
                outcome: if ok { Outcome::Passed } else { Outcome::Failed },
                detail,
            }
        })
        .collect()
}

static ADAPTER_CANCELLED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HANDLER: OnceLock<Result<(), String>> = OnceLock::new();

#[cfg(unix)]
extern "C" fn mark_adapter_cancelled(_: libc::c_int) {
    ADAPTER_CANCELLED.store(true, Ordering::SeqCst);
}

fn install_adapter_signal_handlers() -> Result<(), String> {
    SIGNAL_HANDLER
        .get_or_init(|| {
            #[cfg(unix)]
            unsafe {
                if libc::signal(
                    libc::SIGINT,
                    mark_adapter_cancelled as *const () as libc::sighandler_t,
                ) == libc::SIG_ERR
                    || libc::signal(
                        libc::SIGTERM,
                        mark_adapter_cancelled as *const () as libc::sighandler_t,
                    ) == libc::SIG_ERR
                {
                    return Err("cannot install adapter cancellation handlers".into());
                }
            }
            Ok(())
        })
        .clone()
}

struct AdapterTempDirectory(PathBuf);

impl Drop for AdapterTempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn adapter_temp_directory(adapter: &PythonAdapter) -> Result<AdapterTempDirectory, String> {
    let parent = std::env::temp_dir().join("ios-rust-validate");
    std::fs::create_dir_all(&parent)
        .map_err(|error| format!("cannot create adapter temp root: {error}"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))?
        .as_nanos();
    let path = parent.join(format!("{}-{}-{nonce}", adapter.id, std::process::id()));
    std::fs::create_dir(&path)
        .map_err(|error| format!("cannot create adapter temp dir: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| format!("cannot secure adapter temp dir: {error}"))?;
    }
    Ok(AdapterTempDirectory(path))
}

fn adapter_problem(pilot: &Pilot, adapter: &PythonAdapter, detail: String) -> GateResult {
    GateResult {
        capability: pilot.id.into(),
        gate: format!("Python adapter {}", adapter.id),
        phase: "python-adapter".into(),
        outcome: if adapter.required {
            Outcome::Blocked
        } else {
            Outcome::Skipped
        },
        detail,
    }
}

fn parse_python_version(output: &[u8]) -> Option<(u64, u64)> {
    let output = String::from_utf8_lossy(output);
    let mut numbers = output
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u64>().ok());
    Some((numbers.next()?, numbers.next()?))
}

fn minimum_python_version(version: &str) -> Option<(u64, u64)> {
    let mut parts = version.split('.').map(str::parse::<u64>);
    Some((parts.next()?.ok()?, parts.next()?.ok()?))
}

fn adapter_file(root: &Path, path: &str, kind: &str) -> Result<PathBuf, String> {
    let path = root
        .join(path)
        .canonicalize()
        .map_err(|error| format!("cannot read adapter {kind} `{path}`: {error}"))?;
    if !path.starts_with(root) || !path.is_file() {
        return Err(format!(
            "adapter {kind} is outside the workspace or not a file"
        ));
    }
    Ok(path)
}

fn capture_bounded<R: Read>(mut reader: R, limit: usize, exceeded: Arc<AtomicBool>) -> Vec<u8> {
    let mut output = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                let remaining = limit.saturating_sub(output.len());
                output.extend_from_slice(&buffer[..count.min(remaining)]);
                if count > remaining {
                    exceeded.store(true, Ordering::SeqCst);
                }
            }
        }
    }
    output
}

fn directory_size(path: &Path) -> io::Result<u64> {
    let mut total = 0u64;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            total = total.saturating_add(directory_size(&entry.path())?);
        } else if kind.is_file() {
            total = total.saturating_add(entry.metadata()?.len());
        }
    }
    Ok(total)
}

fn run_python_adapter(pilot: &Pilot, adapter: &PythonAdapter, workspace_root: &Path) -> GateResult {
    if let Err(error) = install_adapter_signal_handlers() {
        return adapter_problem(pilot, adapter, error);
    }
    let script = match adapter_file(workspace_root, adapter.script, "script") {
        Ok(script) => script,
        Err(error) => return adapter_problem(pilot, adapter, error),
    };
    let mut inputs = Vec::with_capacity(adapter.input_paths.len());
    for path in adapter.input_paths {
        match adapter_file(workspace_root, path, "input") {
            Ok(input) => inputs.push(input),
            Err(error) => return adapter_problem(pilot, adapter, error),
        }
    }
    let version = match Command::new(adapter.interpreter).arg("--version").output() {
        Ok(version) if version.status.success() => version,
        Ok(version) => {
            return adapter_problem(
                pilot,
                adapter,
                format!("Python interpreter check failed with {}", version.status),
            );
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return adapter_problem(
                pilot,
                adapter,
                format!(
                    "required interpreter `{}` is unavailable",
                    adapter.interpreter
                ),
            );
        }
        Err(error) => {
            return adapter_problem(
                pilot,
                adapter,
                format!("Python interpreter check failed: {error}"),
            );
        }
    };
    let version_text = [version.stdout, version.stderr].concat();
    let Some(actual_version) = parse_python_version(&version_text) else {
        return adapter_problem(
            pilot,
            adapter,
            "Python interpreter returned an invalid version".into(),
        );
    };
    let Some(minimum_version) = minimum_python_version(adapter.minimum_version) else {
        return adapter_problem(
            pilot,
            adapter,
            "adapter minimum Python version is invalid".into(),
        );
    };
    if actual_version < minimum_version {
        return adapter_problem(
            pilot,
            adapter,
            format!(
                "Python {}.{} is below required {}",
                actual_version.0, actual_version.1, adapter.minimum_version
            ),
        );
    }
    let temp = match adapter_temp_directory(adapter) {
        Ok(temp) => temp,
        Err(error) => return adapter_problem(pilot, adapter, error),
    };
    let input_paths = inputs
        .iter()
        .map(|path| path.strip_prefix(workspace_root).unwrap().to_string_lossy())
        .map(|path| path.into_owned())
        .collect::<Vec<_>>();
    let input = match serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "id": adapter.id,
        "workspace_root": workspace_root.to_string_lossy().into_owned(),
        "input_paths": input_paths,
        "evidence_type": adapter.evidence_type
    })) {
        Ok(input) if input.len() <= 65_536 => input,
        Ok(_) => {
            return adapter_problem(pilot, adapter, "adapter input exceeds 65536 bytes".into());
        }
        Err(error) => {
            return adapter_problem(
                pilot,
                adapter,
                format!("cannot encode adapter input: {error}"),
            );
        }
    };
    let path_env = std::env::var_os("PATH").unwrap_or_default();
    let mut command = Command::new(adapter.interpreter);
    command
        .arg("-I")
        .arg(script)
        .current_dir(workspace_root)
        .env_clear()
        .env("PATH", path_env)
        .env("PYTHONNOUSERSITE", "1")
        .env("TMPDIR", &temp.0)
        .env("TMP", &temp.0)
        .env("TEMP", &temp.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return adapter_problem(
                pilot,
                adapter,
                format!("cannot start Python adapter: {error}"),
            );
        }
    };
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let stdout_exceeded = Arc::new(AtomicBool::new(false));
    let stderr_exceeded = Arc::new(AtomicBool::new(false));
    let stdout_thread = {
        let exceeded = Arc::clone(&stdout_exceeded);
        let limit = adapter.stdout_limit_bytes;
        thread::spawn(move || capture_bounded(stdout, limit, exceeded))
    };
    let stderr_thread = {
        let exceeded = Arc::clone(&stderr_exceeded);
        let limit = adapter.stderr_limit_bytes;
        thread::spawn(move || capture_bounded(stderr, limit, exceeded))
    };
    let write_result = child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(&input);
    if let Err(error) = write_result {
        let _ = child.kill();
        let _ = child.wait();
        return adapter_problem(
            pilot,
            adapter,
            format!("cannot send adapter input: {error}"),
        );
    }
    let started = Instant::now();
    let mut exit_status = None;
    let mut stop_reason = None;
    loop {
        if ADAPTER_CANCELLED.load(Ordering::SeqCst) {
            stop_reason = Some("adapter cancelled; child process terminated".to_owned());
            break;
        }
        if stdout_exceeded.load(Ordering::SeqCst) || stderr_exceeded.load(Ordering::SeqCst) {
            stop_reason = Some("adapter exceeded its stdout or stderr limit".to_owned());
            break;
        }
        if started.elapsed() >= Duration::from_secs(adapter.deadline_seconds) {
            stop_reason = Some("adapter deadline exceeded; child process terminated".to_owned());
            break;
        }
        match directory_size(&temp.0) {
            Ok(size) if size > adapter.temporary_space_bytes => {
                stop_reason = Some("adapter exceeded its temporary-space limit".to_owned());
                break;
            }
            Err(error) => {
                stop_reason = Some(format!("cannot inspect adapter temp use: {error}"));
                break;
            }
            _ => {}
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                exit_status = Some(status);
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                stop_reason = Some(format!("cannot inspect adapter process: {error}"));
                break;
            }
        }
    }
    if stop_reason.is_some() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();
    if let Some(reason) = stop_reason {
        return adapter_problem(pilot, adapter, reason);
    }
    let status = exit_status.expect("child exit status is set when the process ends");
    if !status.success() {
        return adapter_problem(
            pilot,
            adapter,
            format!(
                "adapter exited with {status}; stderr_bytes={}",
                stderr.len()
            ),
        );
    }
    let response: PythonAdapterResponse = match serde_json::from_slice(&stdout) {
        Ok(response) => response,
        Err(error) => {
            return adapter_problem(
                pilot,
                adapter,
                format!("adapter returned invalid JSON: {error}"),
            );
        }
    };
    if response.schema_version != 1 || response.detail.len() > 4096 {
        return adapter_problem(
            pilot,
            adapter,
            "adapter response has an unsupported schema or oversized detail".into(),
        );
    }
    let (outcome, detail) = match response.status.as_str() {
        "PASS" => (Outcome::Passed, response.detail),
        "FAIL" if adapter.required => (Outcome::Failed, response.detail),
        "FAIL" => (
            Outcome::OptionalFailed,
            format!("optional adapter reported FAIL: {}", response.detail),
        ),
        "SKIPPED" if adapter.required => (
            Outcome::Blocked,
            format!("required adapter skipped: {}", response.detail),
        ),
        "SKIPPED" => (Outcome::Skipped, response.detail),
        "ERROR-BLOCKED" if adapter.required => (Outcome::Blocked, response.detail),
        "ERROR-BLOCKED" => (
            Outcome::OptionalBlocked,
            format!("optional adapter blocked: {}", response.detail),
        ),
        _ => return adapter_problem(pilot, adapter, "adapter returned an unknown status".into()),
    };
    GateResult {
        capability: pilot.id.into(),
        gate: format!("Python adapter {}", adapter.id),
        phase: "python-adapter".into(),
        outcome,
        detail: if detail.is_empty() {
            format!("evidence_type={}", adapter.evidence_type)
        } else {
            detail
        },
    }
}

#[cfg(test)]
fn validate_import_fixture(
    imports: &str,
    expected: &[&str],
    forbidden: &[&str],
) -> Result<(), String> {
    for item in expected {
        if !imports.contains(item) {
            return Err(format!("missing expected import `{item}`"));
        }
    }
    for item in forbidden {
        if imports.contains(item) {
            return Err(format!("forbidden import `{item}` is present"));
        }
    }
    Ok(())
}

#[cfg(test)]
fn validate_deployment_floor(build_output: &str, expected: &str) -> Result<(), String> {
    let normalized = expected.trim_end_matches(".0");
    if build_output.lines().any(|line| {
        let line = line.trim();
        line.strip_prefix("minos ")
            .is_some_and(|value| value.trim_end_matches(".0") == normalized)
    }) {
        Ok(())
    } else {
        Err(format!(
            "binary deployment floor does not match iOS {expected}"
        ))
    }
}

fn find_swift_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            find_swift_files(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "swift")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn run_pilot(
    pilot: &Pilot,
    runner: &mut impl CommandRunner,
    workspace_root: &Path,
) -> Vec<GateResult> {
    let mut results = source_guards(pilot, workspace_root);
    for directory in pilot.source_dirs {
        let mut files = Vec::new();
        let result = find_swift_files(&workspace_root.join(directory), &mut files);
        let ok = result.is_ok() && files.is_empty();
        results.push(GateResult {
            capability: pilot.id.into(),
            gate: format!("no shipping Swift in {directory}"),
            phase: "policy".into(),
            outcome: if ok { Outcome::Passed } else { Outcome::Failed },
            detail: if let Err(error) = result {
                error.to_string()
            } else if files.is_empty() {
                String::new()
            } else {
                format!("found Swift source: {}", files[0].display())
            },
        });
    }
    for path in pilot.docs {
        let exists = workspace_root.join(path).is_file();
        results.push(GateResult {
            capability: pilot.id.into(),
            gate: format!("documentation {path}"),
            phase: "documentation".into(),
            outcome: if exists {
                Outcome::Passed
            } else {
                Outcome::Failed
            },
            detail: if exists {
                String::new()
            } else {
                "required documentation file is missing".into()
            },
        });
    }
    for path in pilot.scoped_files {
        let content = std::fs::read_to_string(workspace_root.join(path));
        let trailing = content.as_ref().ok().and_then(|text| {
            text.lines()
                .position(|line| line.ends_with(' ') || line.ends_with('\t'))
        });
        let ok = content.is_ok() && trailing.is_none();
        results.push(GateResult {
            capability: pilot.id.into(),
            gate: format!("trailing whitespace {path}"),
            phase: "source-policy".into(),
            outcome: if ok { Outcome::Passed } else { Outcome::Failed },
            detail: if let Some(line) = trailing {
                format!("trailing whitespace at line {}", line + 1)
            } else if let Err(error) = content {
                error.to_string()
            } else {
                String::new()
            },
        });
    }
    for step in build_steps(pilot) {
        results.push(run_step(pilot, &step, runner));
    }
    for adapter in pilot.adapters {
        results.push(run_python_adapter(pilot, adapter, workspace_root));
    }
    for target in pilot
        .targets
        .iter()
        .filter(|target| target.sdk == "iphonesimulator")
    {
        results.push(GateResult {
            capability: pilot.id.into(),
            gate: format!("simulator execution {}", target.triple),
            phase: "simulator-execution".into(),
            outcome: Outcome::Skipped,
            detail:
                "validation builds and inspects link probes only; no simulator execution is claimed"
                    .into(),
        });
    }
    results.push(GateResult {
        capability: pilot.id.into(),
        gate: "physical device proof".into(),
        phase: "device-proof".into(),
        outcome: Outcome::Skipped,
        detail: "no attached-device runtime test is part of this validation pilot".into(),
    });
    results
}

fn status_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Passed => "pass",
        Outcome::Failed | Outcome::OptionalFailed => "fail",
        Outcome::Skipped => "skipped",
        Outcome::Blocked | Outcome::OptionalBlocked => "error-blocked",
    }
}

fn render(results: &[GateResult], json: bool) -> Result<(), String> {
    if results.len() > MAX_RESULT_COUNT {
        return Err(format!(
            "validation produced more than {MAX_RESULT_COUNT} gate results"
        ));
    }
    if json {
        let records = results
            .iter()
            .map(|result| {
                format!(
                    "{{\"capability\":{},\"gate\":{},\"phase\":{},\"status\":{},\"detail\":{}}}",
                    json_string(&result.capability),
                    json_string(&result.gate),
                    json_string(&result.phase),
                    json_string(status_name(result.outcome)),
                    json_string(&bounded_text(&result.detail, MAX_GATE_DETAIL_BYTES))
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let output = format!("{{\"schema_version\":1,\"results\":[{records}]}}");
        if output.len() > MAX_RESULT_JSON_BYTES {
            return Err(format!(
                "validation JSON exceeds the {MAX_RESULT_JSON_BYTES} byte output limit"
            ));
        }
        println!("{output}");
    } else {
        for result in results {
            let detail = bounded_text(&result.detail, MAX_GATE_DETAIL_BYTES);
            if detail.is_empty() {
                println!(
                    "{} {} {}",
                    status_name(result.outcome),
                    result.capability,
                    result.gate
                );
            } else {
                println!(
                    "{} {} {}: {}",
                    status_name(result.outcome),
                    result.capability,
                    result.gate,
                    detail
                );
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyGraph {
    edges: Vec<(String, String)>,
}

impl DependencyGraph {
    fn reverse_closure(&self, changed: impl IntoIterator<Item = String>) -> BTreeSet<String> {
        let mut seen = changed.into_iter().collect::<BTreeSet<_>>();
        let mut queue = seen.iter().cloned().collect::<VecDeque<_>>();
        while let Some(dependency) = queue.pop_front() {
            for (consumer, required) in &self.edges {
                if required == &dependency && seen.insert(consumer.clone()) {
                    queue.push_back(consumer.clone());
                }
            }
        }
        seen
    }
}

fn changed_nodes(
    paths: &[String],
    pilots: &[Pilot],
    path_rules: &[PathRuleSpec],
) -> (bool, BTreeSet<String>) {
    let mut global = false;
    let mut nodes = BTreeSet::new();
    for path in paths {
        let mut recognized = false;
        for rule in path_rules {
            let matches = if rule.path.ends_with('/') {
                path.starts_with(&rule.path)
            } else {
                path == &rule.path
            };
            if matches {
                recognized = true;
                global |= rule.global;
                if let Some(node) = &rule.node {
                    nodes.insert(node.clone());
                }
            }
        }
        for pilot in pilots {
            let source_match = pilot
                .source_dirs
                .iter()
                .any(|directory| path.starts_with(&format!("{directory}/")));
            if source_match {
                recognized = true;
                nodes.insert(pilot.package.into());
            }
            if pilot.scoped_files.contains(&path.as_str()) {
                recognized = true;
                nodes.insert(pilot.package.into());
            }
        }
        if !recognized {
            global = true;
        }
    }
    (global, nodes)
}

fn changed_paths(base: &str, runner: &mut impl CommandRunner) -> Result<Vec<String>, String> {
    let verify = runner
        .run(
            "git",
            &[
                "rev-parse".into(),
                "--verify".into(),
                format!("{base}^{{commit}}"),
            ],
            &[],
        )
        .map_err(|error| format!("validate explicit base `{base}`: {error}"))?;
    if !verify.success {
        return Err(format!(
            "invalid explicit base `{base}`: {}",
            result_text(&verify)
        ));
    }
    let diff = runner
        .run(
            "git",
            &[
                "diff".into(),
                "--name-only".into(),
                base.into(),
                "--".into(),
            ],
            &[],
        )
        .map_err(|error| format!("read changed paths from {base}: {error}"))?;
    if !diff.success {
        return Err(format!(
            "git diff from {base} failed: {}",
            result_text(&diff)
        ));
    }
    let untracked = runner
        .run(
            "git",
            &[
                "ls-files".into(),
                "--others".into(),
                "--exclude-standard".into(),
            ],
            &[],
        )
        .map_err(|error| format!("read untracked paths: {error}"))?;
    if !untracked.success {
        return Err(format!("git ls-files failed: {}", result_text(&untracked)));
    }
    Ok(diff
        .stdout
        .lines()
        .chain(untracked.stdout.lines())
        .map(str::to_owned)
        .collect())
}

fn pilot_from<'a>(id: &str, pilots: &'a [Pilot]) -> Option<&'a Pilot> {
    pilots.iter().find(|pilot| pilot.id == id)
}

#[cfg(test)]
fn default_config() -> &'static ValidationConfig {
    static CONFIG: OnceLock<ValidationConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let workspace_root = root();
        let spec = workspace_root.join("tools/validation/specs/validation-v1.json");
        load_validation_config(&workspace_root, &spec)
            .expect("checked-in validation spec must be valid")
    })
}

#[cfg(test)]
fn default_pilots() -> &'static [Pilot] {
    &default_config().pilots
}

#[cfg(test)]
fn pilot(id: &str) -> Option<&'static Pilot> {
    pilot_from(id, default_pilots())
}

fn command_snapshot(pilot: &Pilot) -> String {
    build_steps(pilot)
        .iter()
        .map(|step| format!("{} | {} {}", step.phase, step.program, step.args.join(" ")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn explain(pilot: &Pilot) {
    println!("capability: {}", pilot.id);
    println!("package: {}", pilot.package);
    println!("manifest: {}", pilot.manifest);
    println!("check script: {}", pilot.check_script);
    println!("targets:");
    for target in pilot.targets {
        println!(
            "  {} sdk={} deployment=iOS {}",
            target.triple, target.sdk, target.minimum_os
        );
    }
    println!(
        "ABI/link targets: {}",
        pilot
            .abi_targets
            .iter()
            .map(|target| target.triple)
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "expected imports: {}",
        if pilot.expected_imports.is_empty() {
            "none declared".into()
        } else {
            pilot.expected_imports.join(", ")
        }
    );
    println!(
        "ABI fixtures: {}",
        if pilot.abi_scripts.is_empty() {
            "none".into()
        } else {
            pilot.abi_scripts.join(", ")
        }
    );
    for guard in pilot.guards {
        println!(
            "{} `{}` in {} — {}",
            if guard.must_exist {
                "require"
            } else {
                "forbid"
            },
            guard.text,
            guard.path,
            guard.message
        );
    }
    for adapter in pilot.adapters {
        println!(
            "Python adapter {} ({}): interpreter={} >= {}, evidence={}, inputs={}",
            adapter.id,
            if adapter.required {
                "required"
            } else {
                "optional"
            },
            adapter.interpreter,
            adapter.minimum_version,
            adapter.evidence_type,
            adapter.input_paths.join(", ")
        );
    }
    println!("documentation gates: {}", pilot.docs.join(", "));
    println!(
        "shared gates: source/negative guards, formatting, locked host and target builds, Clippy, rustdoc, docs index, zero-Swift-source, scoped diff check"
    );
    println!(
        "link/import scans are distinct from compile; simulator execution and physical-device proof are reported as skipped unless a runtime gate exists"
    );
    println!("command snapshot:");
    for line in command_snapshot(pilot).lines() {
        println!("  {line}");
    }
}

fn explain_json(pilot: &Pilot) {
    let targets = pilot
        .targets
        .iter()
        .map(|target| {
            serde_json::json!({
                "triple": target.triple,
                "sdk": target.sdk,
                "minimum_os": target.minimum_os
            })
        })
        .collect::<Vec<_>>();
    let abi_targets = pilot
        .abi_targets
        .iter()
        .map(|target| {
            serde_json::json!({
                "triple": target.triple,
                "sdk": target.sdk,
                "minimum_os": target.minimum_os
            })
        })
        .collect::<Vec<_>>();
    let guards = pilot
        .guards
        .iter()
        .map(|guard| {
            serde_json::json!({
                "path": guard.path,
                "text": guard.text,
                "expect": if guard.must_exist { "present" } else { "absent" },
                "reason": guard.message
            })
        })
        .collect::<Vec<_>>();
    let commands = build_steps(pilot)
        .iter()
        .map(|step| {
            serde_json::json!({
                "name": step.name,
                "phase": step.phase,
                "program": step.program,
                "args": step.args,
                "target": step.target.map(|target| serde_json::json!({
                    "triple": target.triple,
                    "sdk": target.sdk,
                    "minimum_os": target.minimum_os
                })),
                "required": true,
                "unavailable_policy": "reported as skipped with an explicit reason"
            })
        })
        .collect::<Vec<_>>();
    let adapters = pilot
        .adapters
        .iter()
        .map(|adapter| {
            serde_json::json!({
                "id": adapter.id,
                "interpreter": adapter.interpreter,
                "minimum_version": adapter.minimum_version,
                "script": adapter.script,
                "input_paths": adapter.input_paths,
                "required": adapter.required,
                "evidence_type": adapter.evidence_type,
                "deadline_seconds": adapter.deadline_seconds,
                "stdout_limit_bytes": adapter.stdout_limit_bytes,
                "stderr_limit_bytes": adapter.stderr_limit_bytes,
                "temporary_space_bytes": adapter.temporary_space_bytes
            })
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::json!({
            "schema_version": 1,
            "id": pilot.id,
            "package": pilot.package,
            "inputs": {
                "manifest": pilot.manifest,
                "check_script": pilot.check_script,
                "source_dirs": pilot.source_dirs,
                "scoped_files": pilot.scoped_files,
                "docs": pilot.docs
            },
            "targets": targets,
            "abi_targets": abi_targets,
            "expected_imports": pilot.expected_imports,
            "abi_oracles_and_link_scans": pilot.abi_scripts,
            "guards": guards,
            "commands": commands,
            "adapters": adapters,
            "evidence_limitations": [
                "compile, link and import evidence do not prove simulator execution",
                "compile, link and import evidence do not prove physical-device runtime behavior"
            ]
        })
    );
}

pub(crate) fn run_at(
    args: &[String],
    workspace_root: &Path,
    spec_path: &Path,
) -> Result<(), String> {
    let config = load_validation_config(workspace_root, spec_path)?;
    let pilots = config.pilots.as_slice();
    let workspace_root = workspace_root
        .canonicalize()
        .map_err(|error| format!("invalid workspace root: {error}"))?;
    let mut mode: Option<(&str, Option<&str>)> = None;
    let mut format_json = false;
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        if arg == "--format" {
            index += 1;
            match args.get(index).map(String::as_str) {
                Some("json") => format_json = true,
                Some("human") => format_json = false,
                _ => return Err("`--format` must be `human` or `json`".into()),
            }
        } else if ["--list", "--all"].contains(&arg) {
            if mode.replace((arg, None)).is_some() {
                return Err("choose exactly one validation mode".into());
            }
        } else if ["--explain", "--capability", "--changed"].contains(&arg) {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("`{arg}` requires a value"))?;
            if mode.replace((arg, Some(value))).is_some() {
                return Err("choose exactly one validation mode".into());
            }
        } else {
            return Err(format!("unexpected validation argument `{arg}`"));
        }
        index += 1;
    }
    let (mode, value) = mode.ok_or_else(|| "usage: ios-rust-validate --workspace-root PATH --spec PATH (--list | --explain ID | --capability ID | --changed EXPLICIT_BASE | --all) [--format human|json]".to_owned())?;
    if mode == "--list" {
        if format_json {
            println!(
                "{{\"schema_version\":1,\"pilots\":[{}]}}",
                pilots
                    .iter()
                    .map(|pilot| json_string(pilot.id))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        } else {
            for pilot in pilots {
                println!("{}", pilot.id);
            }
        }
        return Ok(());
    }
    if mode == "--explain" {
        let spec = pilot_from(value.unwrap(), pilots)
            .ok_or_else(|| format!("unknown capability `{}`", value.unwrap()))?;
        if format_json {
            explain_json(spec);
        } else {
            explain(spec);
        }
        return Ok(());
    }
    let mut runner = ProcessRunner {
        workspace_root: workspace_root.clone(),
    };
    let selected: Vec<&Pilot> = match mode {
        "--capability" => vec![
            pilot_from(value.unwrap(), pilots)
                .ok_or_else(|| format!("unknown capability `{}`", value.unwrap()))?,
        ],
        "--all" => pilots.iter().collect(),
        "--changed" => {
            let paths = changed_paths(value.unwrap(), &mut runner)?;
            let (global, nodes) = changed_nodes(&paths, pilots, &config.path_rules);
            if global {
                pilots.iter().collect()
            } else {
                let affected = config.graph.reverse_closure(nodes);
                pilots
                    .iter()
                    .filter(|pilot| affected.contains(pilot.package))
                    .collect()
            }
        }
        _ => unreachable!(),
    };
    if selected.is_empty() {
        if format_json {
            println!(
                "{{\"schema_version\":1,\"results\":[],\"message\":\"no validation pilots affected by changed files\"}}"
            );
        } else {
            println!("no validation pilots affected by changed files");
        }
        return Ok(());
    }
    let mut results = Vec::new();
    for pilot in selected {
        results.extend(run_pilot(pilot, &mut runner, &workspace_root));
    }
    render(&results, format_json)?;
    if results
        .iter()
        .any(|result| matches!(result.outcome, Outcome::Failed | Outcome::Blocked))
    {
        Err("one or more validation gates failed".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    type RecordedCall = (String, Vec<String>, Vec<(String, String)>);

    #[derive(Default)]
    struct FakeRunner {
        calls: Vec<RecordedCall>,
        results: VecDeque<io::Result<CommandResult>>,
    }

    impl FakeRunner {
        fn with_results(results: impl IntoIterator<Item = io::Result<CommandResult>>) -> Self {
            Self {
                calls: Vec::new(),
                results: results.into_iter().collect(),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(
            &mut self,
            program: &str,
            args: &[String],
            env: &[(String, String)],
        ) -> io::Result<CommandResult> {
            self.calls
                .push((program.to_owned(), args.to_vec(), env.to_vec()));
            self.results.pop_front().unwrap_or_else(|| {
                Ok(CommandResult {
                    success: true,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            })
        }
    }

    fn fake_success(stdout: &str) -> io::Result<CommandResult> {
        Ok(CommandResult {
            success: true,
            stdout: stdout.to_owned(),
            stderr: String::new(),
        })
    }

    fn fake_failure(stderr: &str) -> io::Result<CommandResult> {
        Ok(CommandResult {
            success: false,
            stdout: String::new(),
            stderr: stderr.to_owned(),
        })
    }

    #[test]
    fn import_fixtures_reject_missing_and_forbidden_symbols() {
        assert!(
            validate_import_fixture(
                "RealityFoundation.framework\nPhotogrammetrySession.isSupported",
                &["RealityFoundation.framework"],
                &["privateSymbol"]
            )
            .is_ok()
        );
        assert!(
            validate_import_fixture(
                "RealityFoundation.framework",
                &["PhotogrammetrySession.isSupported"],
                &[]
            )
            .unwrap_err()
            .contains("missing expected import")
        );
        assert!(
            validate_import_fixture(
                "RealityFoundation.framework privateSymbol",
                &[],
                &["privateSymbol"]
            )
            .unwrap_err()
            .contains("forbidden import")
        );
    }

    #[test]
    fn deployment_floor_fixture_accepts_only_the_declared_floor() {
        assert!(validate_deployment_floor("minos 17.0\n", "17.0").is_ok());
        assert!(validate_deployment_floor("minos 16.0\n", "17.0").is_err());
    }

    #[test]
    fn reverse_dependency_closure_selects_transitive_consumers() {
        let graph = DependencyGraph {
            edges: vec![
                ("pilot".into(), "adapter".into()),
                ("adapter".into(), "core".into()),
            ],
        };
        let affected = graph.reverse_closure(["core".into()]);
        assert!(affected.contains("adapter"));
        assert!(affected.contains("pilot"));
    }

    #[test]
    fn shared_native_or_abi_changes_select_all_transitive_pilots() {
        let pilots = default_pilots();
        let graph = &default_config().graph;
        let path_rules = &default_config().path_rules;
        let (_, native) = changed_nodes(
            &["tools/native-build-support/src/lib.rs".into()],
            pilots,
            path_rules,
        );
        let affected = graph.reverse_closure(native);
        assert!(affected.contains("ios-activitykit-status"));
        assert!(affected.contains("ios-alarmkit-status"));
        assert!(affected.contains("ios-photogrammetry-status"));
        let (_, abi) = changed_nodes(
            &["interop/swift-abi-core/src/lib.rs".into()],
            pilots,
            path_rules,
        );
        let affected = graph.reverse_closure(abi);
        assert!(affected.contains("ios-activitykit-status"));
        assert!(affected.contains("ios-alarmkit-status"));
        assert!(affected.contains("ios-photogrammetry-status"));
        assert!(!affected.contains("ios-homekit-identify-status"));
        let (_, generated) = changed_nodes(
            &["interop/swift-abi-generated/src/lib.rs".into()],
            pilots,
            path_rules,
        );
        let affected = graph.reverse_closure(generated);
        assert!(affected.contains("ios-activitykit-status"));
        assert!(affected.contains("ios-alarmkit-status"));
        assert!(!affected.contains("ios-homekit-identify-status"));
    }

    #[test]
    fn specs_preserve_expected_command_shapes_and_targets() {
        let activity = pilot("ios-activitykit-status").unwrap();
        let steps = build_steps(activity);
        assert!(steps.iter().any(|step| {
            step.args
                .windows(3)
                .any(|window| window == ["-p", "ios-activitykit-status", "--target"])
        }));
        assert_eq!(activity.expected_imports[0], "ActivityKit.framework (weak)");
        assert_eq!(activity.abi_scripts.len(), 2);
        assert_eq!(
            pilot("ios-photogrammetry-status").unwrap().targets[0].minimum_os,
            "17.0"
        );
        assert!(steps.iter().any(
            |step| step.program == "git" && step.args.first().is_some_and(|arg| arg == "diff")
        ));
    }

    #[test]
    fn test_only_declarative_pilot_and_adapters_load_from_a_separate_spec() {
        let workspace_root = root();
        let spec = workspace_root.join("tools/validation/fixtures/example-validation-v1.json");
        let config = load_validation_config(&workspace_root, &spec).unwrap();
        assert_eq!(config.pilots.len(), 1);
        assert_eq!(config.pilots[0].id, "example-validation-fixture");
        assert_eq!(config.pilots[0].adapters.len(), 2);
        assert!(
            config.pilots[0]
                .adapters
                .iter()
                .all(|adapter| adapter.required)
        );
    }

    #[test]
    fn status_names_keep_skipped_distinct_from_passed() {
        assert_eq!(status_name(Outcome::Passed), "pass");
        assert_eq!(status_name(Outcome::Failed), "fail");
        assert_eq!(status_name(Outcome::OptionalFailed), "fail");
        assert_eq!(status_name(Outcome::Skipped), "skipped");
        assert_eq!(status_name(Outcome::Blocked), "error-blocked");
        assert_eq!(status_name(Outcome::OptionalBlocked), "error-blocked");
    }

    #[test]
    fn diagnostic_text_is_bounded_without_splitting_utf8() {
        let value = format!("{}é", "x".repeat(MAX_GATE_DETAIL_BYTES));
        let bounded = bounded_text(&value, MAX_GATE_DETAIL_BYTES);
        assert!(bounded.len() <= MAX_GATE_DETAIL_BYTES + "… [truncated]".len());
        assert!(bounded.ends_with("… [truncated]"));
    }

    #[test]
    fn changed_files_include_modified_untracked_and_deleted_paths() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ios-rust-changed-test-{nonce}"));
        std::fs::create_dir_all(&root).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            output
        };
        git(&["init", "--quiet"]);
        git(&["config", "user.email", "validation@example.invalid"]);
        git(&["config", "user.name", "Validation fixture"]);
        std::fs::write(root.join("modified.txt"), "before\n").unwrap();
        std::fs::write(root.join("deleted.txt"), "before\n").unwrap();
        git(&["add", "modified.txt", "deleted.txt"]);
        git(&["commit", "--quiet", "-m", "fixture"]);
        let base = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_owned();
        std::fs::write(root.join("modified.txt"), "after\n").unwrap();
        std::fs::remove_file(root.join("deleted.txt")).unwrap();
        std::fs::write(root.join("untracked.txt"), "new\n").unwrap();
        let paths = changed_paths(
            &base,
            &mut ProcessRunner {
                workspace_root: root.clone(),
            },
        )
        .unwrap()
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert!(paths.contains("modified.txt"));
        assert!(paths.contains("deleted.txt"));
        assert!(paths.contains("untracked.txt"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn python_version_parser_reads_major_and_minor() {
        assert_eq!(parse_python_version(b"Python 3.13.4\n"), Some((3, 13)));
        assert_eq!(minimum_python_version("3.9"), Some((3, 9)));
        assert_eq!(parse_python_version(b"unknown"), None);
    }

    #[cfg(unix)]
    #[test]
    fn python_adapter_accepts_only_versioned_json_and_reports_blocked_errors() {
        use std::os::unix::fs::PermissionsExt;

        fn fixture_adapter(python: &Path, required: bool) -> PythonAdapter {
            PythonAdapter {
                id: "adapter-fixture",
                interpreter: Box::leak(python.to_string_lossy().into_owned().into_boxed_str()),
                minimum_version: "3.9",
                script: "adapter.py",
                input_paths: Box::leak(vec!["input.txt"].into_boxed_slice()),
                required,
                evidence_type: "fixture-evidence",
                deadline_seconds: 5,
                stdout_limit_bytes: 4096,
                stderr_limit_bytes: 4096,
                temporary_space_bytes: 1_048_576,
            }
        }

        fn write_fake_python(path: &Path, response: &str) {
            let script = format!(
                "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf 'Python 3.13.1\\n'; exit 0; fi\ncat >/dev/null\nprintf '%s\\n' '{}'\n",
                response.replace('\'', "'\\''")
            );
            std::fs::write(path, script).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ios-rust-adapter-test-{nonce}"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("adapter.py"), "# test adapter\n").unwrap();
        std::fs::write(root.join("input.txt"), "fixture\n").unwrap();
        let python = root.join("python3");
        write_fake_python(
            &python,
            "{\"schema_version\":1,\"status\":\"PASS\",\"detail\":\"fixture passed\"}",
        );
        let root = root.canonicalize().unwrap();
        let pilot = pilot("ios-homekit-identify-status").unwrap();
        let pass = run_python_adapter(pilot, &fixture_adapter(&python, true), &root);
        assert_eq!(pass.outcome, Outcome::Passed);
        assert_eq!(pass.detail, "fixture passed");

        write_fake_python(
            &python,
            "{\"schema_version\":1,\"status\":\"FAIL\",\"detail\":\"fixture failed\"}",
        );
        let optional_failure = run_python_adapter(pilot, &fixture_adapter(&python, false), &root);
        assert_eq!(optional_failure.outcome, Outcome::OptionalFailed);
        assert_eq!(status_name(optional_failure.outcome), "fail");

        write_fake_python(
            &python,
            "{\"schema_version\":1,\"status\":\"ERROR-BLOCKED\",\"detail\":\"fixture blocked\"}",
        );
        let optional_blocked = run_python_adapter(pilot, &fixture_adapter(&python, false), &root);
        assert_eq!(optional_blocked.outcome, Outcome::OptionalBlocked);
        assert_eq!(status_name(optional_blocked.outcome), "error-blocked");

        write_fake_python(&python, "not json");
        let blocked = run_python_adapter(pilot, &fixture_adapter(&python, true), &root);
        assert_eq!(blocked.outcome, Outcome::Blocked);
        assert!(blocked.detail.contains("invalid JSON"));

        std::fs::write(
            &python,
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf 'Python 3.13.1\\n'; exit 0; fi\nwhile :; do :; done\n",
        )
        .unwrap();
        std::fs::set_permissions(&python, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut timeout_adapter = fixture_adapter(&python, true);
        timeout_adapter.deadline_seconds = 1;
        let timed_out = run_python_adapter(pilot, &timeout_adapter, &root);
        assert_eq!(timed_out.outcome, Outcome::Blocked);
        assert!(timed_out.detail.contains("deadline exceeded"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shell_paths_are_single_command_arguments() {
        let path = Path::new("tools/validation fixtures/check file.sh");
        let step = Step {
            name: "fixture".into(),
            phase: "abi".into(),
            program: "sh".into(),
            args: vec![path.to_string_lossy().into_owned()],
            env: Vec::new(),
            target: None,
        };
        let mut runner = FakeRunner::default();
        let result = run_step(
            pilot("ios-homekit-identify-status").unwrap(),
            &step,
            &mut runner,
        );
        assert_eq!(result.outcome, Outcome::Passed);
        assert_eq!(
            runner.calls[0].1,
            ["tools/validation fixtures/check file.sh"]
        );
    }

    #[test]
    fn unavailable_sdk_or_rust_target_is_skipped_with_a_reason() {
        let pilot = pilot("ios-homekit-identify-status").unwrap();
        let step = Step {
            name: "device check".into(),
            phase: "compile".into(),
            program: "cargo".into(),
            args: vec!["+1.94.1".into(), "check".into()],
            env: Vec::new(),
            target: Some(
                pilot_from("ios-homekit-identify-status", default_pilots())
                    .unwrap()
                    .targets[0],
            ),
        };
        let mut no_sdk = FakeRunner::with_results([fake_failure("SDK not found")]);
        let result = run_step(pilot, &step, &mut no_sdk);
        assert_eq!(result.outcome, Outcome::Skipped);
        assert!(result.detail.contains("SDK unavailable"));
        assert_eq!(no_sdk.calls.len(), 1);

        let mut no_rust_target =
            FakeRunner::with_results([fake_success("/SDK\n"), fake_success("x86_64-apple-ios\n")]);
        let result = run_step(pilot, &step, &mut no_rust_target);
        assert_eq!(result.outcome, Outcome::Skipped);
        assert!(result.detail.contains("aarch64-apple-ios is not installed"));
        assert_eq!(no_rust_target.calls.len(), 2);
    }

    #[test]
    fn a_failed_gate_is_not_reported_as_a_pass() {
        let step = Step {
            name: "fixture".into(),
            phase: "compile".into(),
            program: "fake-tool".into(),
            args: vec!["argument".into()],
            env: Vec::new(),
            target: None,
        };
        let mut runner = FakeRunner::with_results([fake_failure("compile error")]);
        let result = run_step(
            pilot("ios-homekit-identify-status").unwrap(),
            &step,
            &mut runner,
        );
        assert_eq!(result.outcome, Outcome::Failed);
        assert_eq!(status_name(result.outcome), "fail");
        assert!(result.detail.contains("compile error"));
    }

    #[test]
    fn unavailable_import_tool_is_skipped_with_a_reason() {
        let step = Step {
            name: "import fixture".into(),
            phase: "link-import-scan".into(),
            program: "sh".into(),
            args: vec!["check-link-imports.sh".into()],
            env: Vec::new(),
            target: None,
        };
        let mut runner = FakeRunner::with_results([fake_failure("missing otool")]);
        let result = run_step(
            pilot("ios-homekit-identify-status").unwrap(),
            &step,
            &mut runner,
        );
        assert_eq!(result.outcome, Outcome::Skipped);
        assert!(
            result
                .detail
                .contains("required tool `xcrun` is unavailable")
        );
        assert!(runner.calls[0].1[1].contains("xcrun"));
    }

    #[test]
    fn command_snapshots_preserve_pilot_specific_host_flags() {
        let homekit = build_steps(pilot("ios-homekit-identify-status").unwrap());
        assert_eq!(
            homekit[0].args,
            [
                "+1.94.1",
                "fmt",
                "--manifest-path",
                "platform/ios/ios-homekit-identify-status/Cargo.toml",
                "--",
                "--check"
            ]
        );
        assert_eq!(
            homekit[2].args,
            [
                "+1.94.1",
                "clippy",
                "--locked",
                "--all-targets",
                "-p",
                "ios-homekit-identify-status",
                "--",
                "-D",
                "warnings"
            ]
        );
        let alarm = build_steps(pilot("ios-alarmkit-status").unwrap());
        assert_eq!(
            alarm[1].args,
            [
                "+1.94.1",
                "check",
                "--locked",
                "--offline",
                "-p",
                "ios-alarmkit-status"
            ]
        );
        let photo = build_steps(pilot("ios-photogrammetry-status").unwrap());
        assert_eq!(photo[0].args, ["+1.94.1", "fmt", "--all", "--", "--check"]);
        assert_eq!(
            photo[2].args,
            [
                "+1.94.1",
                "clippy",
                "--locked",
                "--lib",
                "-p",
                "ios-photogrammetry-status",
                "--",
                "-D",
                "warnings"
            ]
        );
    }

    #[test]
    fn explicit_command_snapshots_match_all_four_pilot_specs() {
        assert_eq!(
            command_snapshot(pilot("ios-homekit-identify-status").unwrap()),
            include_str!("../../validation/fixtures/ios-homekit-identify-status.commands")
                .trim_end()
        );
        assert_eq!(
            command_snapshot(pilot("ios-photogrammetry-status").unwrap()),
            include_str!("../../validation/fixtures/ios-photogrammetry-status.commands").trim_end()
        );
        assert_eq!(
            command_snapshot(pilot("ios-alarmkit-status").unwrap()),
            include_str!("../../validation/fixtures/ios-alarmkit-status.commands").trim_end()
        );
        assert_eq!(
            command_snapshot(pilot("ios-activitykit-status").unwrap()),
            include_str!("../../validation/fixtures/ios-activitykit-status.commands").trim_end()
        );
    }
}

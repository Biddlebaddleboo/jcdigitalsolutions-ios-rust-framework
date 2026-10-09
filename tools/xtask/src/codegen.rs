use crate::{
    json_string, json_string_array, output_path, output_text, root, run_capture, write_report,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PROBE_MANIFEST: &str = r#"[package]
name = "codegen-probe"
version = "0.0.0"
edition = "2024"

[workspace]

[profile.release]
opt-level = 3

[dependencies]
framework-core = { path = "../../../../crates/framework-core" }
"#;

const PROBE_SOURCE: &str = r#"#![no_std]

use framework_core::OperationId;

#[unsafe(no_mangle)]
#[inline(never)]
pub fn operation_id_get(value: OperationId) -> u64 {
    value.get()
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn operation_id_new_round_trip(raw: u64) -> u64 {
    match OperationId::new(raw) {
        Some(value) => value.get(),
        None => 0,
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn operation_id_size() -> u64 {
    core::mem::size_of::<OperationId>() as u64
}

#[unsafe(no_mangle)]
#[inline(never)]
pub fn option_operation_id_size() -> u64 {
    core::mem::size_of::<Option<OperationId>>() as u64
}
"#;

const PROBES: &[Probe] = &[
    Probe {
        symbol: "operation_id_get",
        purpose: "OperationId::get Rust-ABI wrapper",
        expected: ExpectedLowering::Identity,
    },
    Probe {
        symbol: "operation_id_new_round_trip",
        purpose: "OperationId::new followed by OperationId::get",
        expected: ExpectedLowering::Identity,
    },
    Probe {
        symbol: "operation_id_size",
        purpose: "size_of::<OperationId>()",
        expected: ExpectedLowering::ConstantEight,
    },
    Probe {
        symbol: "option_operation_id_size",
        purpose: "size_of::<Option<OperationId>>() niche layout",
        expected: ExpectedLowering::ConstantEight,
    },
];

struct Probe {
    symbol: &'static str,
    purpose: &'static str,
    expected: ExpectedLowering,
}

#[derive(Clone, Copy)]
enum ExpectedLowering {
    Identity,
    ConstantEight,
}

struct FunctionAudit {
    symbol: String,
    purpose: String,
    expected_lowering: String,
    signature: String,
    alias_target: Option<String>,
    instructions: Vec<String>,
    calls: Vec<String>,
    allocations: Vec<String>,
    synchronization: Vec<String>,
    expected_shape: bool,
    passed: bool,
    error: Option<String>,
}

struct TargetAudit {
    target: String,
    passed: bool,
    llvm_ir: Option<String>,
    functions: Vec<FunctionAudit>,
    error: Option<String>,
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let output = output_path(args, "--output")?
        .unwrap_or_else(|| PathBuf::from("target/xtask/codegen-audit.json"));
    let rustc = run_capture("rustc", &["-vV"])?;
    if !rustc.status.success() {
        return Err(format!("rustc -vV failed: {}", output_text(&rustc)));
    }
    let rustc_verbose = output_text(&rustc);
    let host = rustc_verbose
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc -vV output has no host target".to_owned())?
        .to_owned();

    let installed = run_capture("rustup", &["target", "list", "--installed"])?;
    if !installed.status.success() {
        return Err(format!(
            "rustup target list --installed failed: {}",
            output_text(&installed)
        ));
    }
    let installed_targets = String::from_utf8_lossy(&installed.stdout)
        .lines()
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut installed_ios_targets = installed_targets
        .iter()
        .filter(|target| target.contains("apple-ios"))
        .cloned()
        .collect::<Vec<_>>();
    installed_ios_targets.sort();
    installed_ios_targets.dedup();

    let mut targets = vec![host.clone()];
    targets.extend(
        installed_ios_targets
            .iter()
            .filter(|target| target.as_str() != host.as_str())
            .cloned(),
    );

    let artifact_root = root().join("target/xtask/codegen-audit");
    let fixture_root = artifact_root.join("fixture");
    let build_root = artifact_root.join("build");
    fs::create_dir_all(fixture_root.join("src"))
        .map_err(|error| format!("create {}: {error}", fixture_root.join("src").display()))?;
    fs::write(fixture_root.join("Cargo.toml"), PROBE_MANIFEST)
        .map_err(|error| format!("write codegen probe manifest: {error}"))?;
    fs::write(fixture_root.join("src/lib.rs"), PROBE_SOURCE)
        .map_err(|error| format!("write codegen probe source: {error}"))?;
    if build_root.exists() {
        fs::remove_dir_all(&build_root)
            .map_err(|error| format!("remove previous {}: {error}", build_root.display()))?;
    }
    let lockfile = fixture_root.join("Cargo.lock");
    match fs::remove_file(&lockfile) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("remove {}: {error}", lockfile.display())),
    }
    let manifest = fixture_root.join("Cargo.toml");
    let generate_lock = run_cargo(
        &[
            "generate-lockfile".into(),
            "--manifest-path".into(),
            path_arg(&manifest),
        ],
        &build_root,
    )?;
    if !generate_lock.status.success() {
        return Err(format!(
            "generate the transient probe lockfile: {}",
            output_text(&generate_lock)
        ));
    }

    let mut results = Vec::with_capacity(targets.len());
    for target in &targets {
        let result = match compile_and_audit_target(&manifest, &build_root, target) {
            Ok(result) => result,
            Err(error) => TargetAudit {
                target: target.clone(),
                passed: false,
                llvm_ir: None,
                functions: Vec::new(),
                error: Some(error),
            },
        };
        results.push(result);
    }

    let passed = results.iter().all(|result| result.passed);
    let target_json = results
        .iter()
        .map(target_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let report = format!(
        "{{\n  \"schema_version\": 1,\n  \"status\": {},\n  \"subject\": \"framework_core::OperationId\",\n  \"rustc_verbose\": {},\n  \"host_target\": {},\n  \"installed_ios_targets\": {},\n  \"audited_targets\": {},\n  \"probe_profile\": \"release opt-level 3 with Rust ABI probe functions and no extern declarations\",\n  \"targets\": [\n    {}\n  ],\n  \"limits\": [\n    \"Structural optimized-IR evidence for four synthetic probe functions, not a benchmark or performance claim.\",\n    \"This does not inspect full-program LTO output, linked binaries, runtime behavior, or FFI ABI compatibility.\",\n    \"A compiler or LLVM version change can alter IR shape and requires review of the recorded function bodies.\"\n  ]\n}}\n",
        json_string(if passed { "passed" } else { "failed" }),
        json_string(&rustc_verbose),
        json_string(&host),
        json_string_array(&installed_ios_targets),
        json_string_array(&targets),
        target_json
    );
    write_report(Some(output.clone()), &report)?;
    if !passed {
        return Err(format!(
            "codegen audit failed; inspect {}",
            if output.is_absolute() {
                output.display().to_string()
            } else {
                root().join(output).display().to_string()
            }
        ));
    }
    println!(
        "OperationId optimized IR checks passed for {} target(s); no performance claim",
        targets.len()
    );
    Ok(())
}

fn run_cargo(args: &[String], target_dir: &Path) -> Result<Output, String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    Command::new(&cargo)
        .args(args)
        .current_dir(root())
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .map_err(|error| format!("run {cargo}: {error}"))
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn compile_and_audit_target(
    manifest: &Path,
    build_root: &Path,
    target: &str,
) -> Result<TargetAudit, String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let manifest_arg = path_arg(manifest);
    let output = Command::new(&cargo)
        .args([
            "rustc",
            "--locked",
            "--manifest-path",
            &manifest_arg,
            "--release",
            "--target",
            target,
            "--lib",
            "--",
            "--emit=llvm-ir",
        ])
        .current_dir(root())
        .env("CARGO_TARGET_DIR", build_root)
        .output()
        .map_err(|error| format!("run cargo rustc for {target}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo rustc optimized LLVM IR failed for {target}: {}",
            output_text(&output)
        ));
    }

    let ir_path = find_probe_ir(build_root, target)?;
    let ir = fs::read_to_string(&ir_path)
        .map_err(|error| format!("read {}: {error}", ir_path.display()))?;
    let relative_ir = ir_path
        .strip_prefix(root())
        .map_err(|error| {
            format!(
                "IR path {} is outside repository: {error}",
                ir_path.display()
            )
        })?
        .to_string_lossy()
        .into_owned();
    let functions = PROBES
        .iter()
        .map(|probe| audit_function(&ir, probe))
        .collect::<Vec<_>>();
    let passed = functions.iter().all(|function| function.passed);
    Ok(TargetAudit {
        target: target.into(),
        passed,
        llvm_ir: Some(relative_ir),
        functions,
        error: None,
    })
}

fn find_probe_ir(build_root: &Path, target: &str) -> Result<PathBuf, String> {
    let deps = build_root.join(target).join("release/deps");
    let entries = fs::read_dir(&deps)
        .map_err(|error| format!("read codegen artifacts {}: {error}", deps.display()))?;
    let mut ir_files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("read codegen artifact entry: {error}"))?;
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "ll")
            && path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("codegen_probe-"))
        {
            ir_files.push(path);
        }
    }
    if ir_files.len() != 1 {
        return Err(format!(
            "expected one optimized codegen_probe LLVM IR file for {target}, found {}",
            ir_files.len()
        ));
    }
    Ok(ir_files.remove(0))
}

fn audit_function(ir: &str, probe: &Probe) -> FunctionAudit {
    let expected_lowering = match probe.expected {
        ExpectedLowering::Identity => "identity i64 return",
        ExpectedLowering::ConstantEight => "constant i64 return 8",
    };
    let (signature, instructions) = match extract_function(ir, probe.symbol) {
        Ok(extracted) => extracted,
        Err(error) => {
            return FunctionAudit {
                symbol: probe.symbol.into(),
                purpose: probe.purpose.into(),
                expected_lowering: expected_lowering.into(),
                signature: String::new(),
                alias_target: None,
                instructions: Vec::new(),
                calls: Vec::new(),
                allocations: Vec::new(),
                synchronization: Vec::new(),
                expected_shape: false,
                passed: false,
                error: Some(error),
            };
        }
    };
    let alias_target = signature
        .split_once(" alias ")
        .and_then(|(_, alias)| alias.rsplit_once("ptr @"))
        .map(|(_, target)| target.trim_end_matches('}').to_owned());
    let calls = instructions
        .iter()
        .filter(|instruction| is_call(instruction))
        .cloned()
        .collect::<Vec<_>>();
    let allocations = instructions
        .iter()
        .filter(|instruction| {
            opcode(instruction) == "alloca"
                || instruction.contains("__rust_alloc")
                || instruction.contains("malloc")
                || instruction.contains("alloc::")
        })
        .cloned()
        .collect::<Vec<_>>();
    let synchronization = instructions
        .iter()
        .filter(|instruction| {
            matches!(opcode(instruction), "atomicrmw" | "cmpxchg" | "fence")
                || instruction.to_ascii_lowercase().contains(" atomic ")
                || instruction.to_ascii_lowercase().contains("mutex")
                || instruction.to_ascii_lowercase().contains("lock")
        })
        .cloned()
        .collect::<Vec<_>>();
    let expected_shape = match probe.expected {
        ExpectedLowering::Identity => identity_return(&signature, probe.symbol, &instructions),
        ExpectedLowering::ConstantEight => {
            if alias_target.is_some() {
                probe.symbol == "option_operation_id_size"
                    && signature.contains(" alias i64 (),")
                    && alias_target.as_deref() == Some("operation_id_size")
            } else {
                return_type(&signature, probe.symbol) == Some("i64")
                    && function_arguments(&signature, probe.symbol)
                        .is_some_and(|arguments| arguments.is_empty())
                    && instructions == ["ret i64 8"]
            }
        }
    };
    let passed =
        expected_shape && calls.is_empty() && allocations.is_empty() && synchronization.is_empty();
    FunctionAudit {
        symbol: probe.symbol.into(),
        purpose: probe.purpose.into(),
        expected_lowering: expected_lowering.into(),
        signature,
        alias_target,
        instructions,
        calls,
        allocations,
        synchronization,
        expected_shape,
        passed,
        error: None,
    }
}

fn extract_function(ir: &str, symbol: &str) -> Result<(String, Vec<String>), String> {
    let marker = format!("@{symbol}(");
    let mut in_header = false;
    let mut in_body = false;
    let mut signature = String::new();
    let mut instructions = Vec::new();
    for line in ir.lines() {
        let trimmed = line.trim();
        if !in_header && !in_body {
            if trimmed.starts_with(&format!("@{symbol} =")) && trimmed.contains(" alias ") {
                return Ok((trimmed.to_owned(), vec![trimmed.to_owned()]));
            }
            if trimmed.starts_with("define ") && trimmed.contains(&marker) {
                in_header = true;
                signature.push_str(trimmed);
                if trimmed.contains('{') {
                    in_header = false;
                    in_body = true;
                }
            }
            continue;
        }
        if in_header {
            signature.push(' ');
            signature.push_str(trimmed);
            if trimmed.contains('{') {
                in_header = false;
                in_body = true;
            }
            continue;
        }
        if trimmed == "}" {
            return Ok((signature, instructions));
        }
        let instruction = line.split(';').next().unwrap_or_default().trim();
        if !instruction.is_empty() && !instruction.ends_with(':') {
            instructions.push(instruction.to_owned());
        }
    }
    Err(format!(
        "optimized LLVM IR has no complete definition for @{symbol}"
    ))
}

fn return_type<'a>(signature: &'a str, symbol: &str) -> Option<&'a str> {
    signature
        .split_once(&format!("@{symbol}("))?
        .0
        .split_whitespace()
        .last()
}

fn function_arguments(signature: &str, symbol: &str) -> Option<Vec<String>> {
    let start = signature.find(&format!("@{symbol}("))? + symbol.len() + 2;
    let remaining = &signature[start..];
    let mut depth = 0_usize;
    let mut end = None;
    for (index, character) in remaining.char_indices() {
        match character {
            '(' => depth += 1,
            ')' if depth == 0 => {
                end = Some(index);
                break;
            }
            ')' => depth -= 1,
            _ => {}
        }
    }
    let parameters = &remaining[..end?];
    if parameters.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut pieces = Vec::new();
    let mut depth = 0_usize;
    let mut piece_start = 0;
    for (index, character) in parameters.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                pieces.push(parameters[piece_start..index].trim());
                piece_start = index + 1;
            }
            _ => {}
        }
    }
    pieces.push(parameters[piece_start..].trim());
    pieces
        .into_iter()
        .map(|piece| piece.split_whitespace().last().map(str::to_owned))
        .collect::<Option<Vec<_>>>()
}

fn identity_return(signature: &str, symbol: &str, instructions: &[String]) -> bool {
    if return_type(signature, symbol) != Some("i64") {
        return false;
    }
    let Some(arguments) = function_arguments(signature, symbol) else {
        return false;
    };
    if arguments.len() != 1 || !arguments[0].starts_with('%') || instructions.len() != 1 {
        return false;
    }
    instructions[0] == format!("ret i64 {}", arguments[0])
}

fn opcode(instruction: &str) -> &str {
    let operation = instruction
        .split_once(" = ")
        .map(|(_, operation)| operation.trim_start())
        .unwrap_or(instruction);
    let mut words = operation.split_whitespace();
    match words.next().unwrap_or_default() {
        "tail" | "musttail" | "notail" => words.next().unwrap_or_default(),
        operation => operation,
    }
}

fn is_call(instruction: &str) -> bool {
    matches!(opcode(instruction), "call" | "invoke" | "callbr")
}

fn function_json(function: &FunctionAudit) -> String {
    format!(
        "{{\"symbol\":{},\"purpose\":{},\"expected_lowering\":{},\"signature\":{},\"alias_target\":{},\"body_instructions\":{},\"identity_or_constant_shape\":{},\"call_instructions\":{},\"ffi_round_trip_absent\":{},\"allocation_instructions\":{},\"synchronization_instructions\":{},\"passed\":{},\"error\":{}}}",
        json_string(&function.symbol),
        json_string(&function.purpose),
        json_string(&function.expected_lowering),
        json_string(&function.signature),
        function
            .alias_target
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        json_string_array(&function.instructions),
        function.expected_shape,
        json_string_array(&function.calls),
        function.calls.is_empty(),
        json_string_array(&function.allocations),
        json_string_array(&function.synchronization),
        function.passed,
        function
            .error
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into())
    )
}

fn target_json(target: &TargetAudit) -> String {
    let functions = target
        .functions
        .iter()
        .map(function_json)
        .collect::<Vec<_>>()
        .join(",");
    let error = target
        .error
        .as_deref()
        .map(json_string)
        .unwrap_or_else(|| "null".into());
    let ir = target
        .llvm_ir
        .as_deref()
        .map(json_string)
        .unwrap_or_else(|| "null".into());
    format!(
        "{{\"target\":{},\"status\":{},\"optimized_llvm_ir\":{},\"functions\":[{}],\"error\":{}}}",
        json_string(&target.target),
        json_string(if target.passed { "passed" } else { "failed" }),
        ir,
        functions,
        error
    )
}

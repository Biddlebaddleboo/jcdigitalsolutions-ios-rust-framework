use crate::{json_string, output_path, output_text, root, run_capture, write_report};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn dependency_audit(args: &[String]) -> Result<(), String> {
    let output_dir = if args.is_empty() {
        PathBuf::from("target/xtask/dependency-audit")
    } else if args.len() == 2 && args[0] == "--output-dir" {
        PathBuf::from(&args[1])
    } else {
        return Err("usage: cargo xtask dependency-audit [--output-dir PATH]".into());
    };
    let output_dir = if output_dir.is_absolute() {
        output_dir
    } else {
        root().join(output_dir)
    };
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("create {}: {error}", output_dir.display()))?;
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let graphs: [(&str, &[&str]); 3] = [
        (
            "workspace-features.txt",
            &[
                "tree",
                "--locked",
                "--workspace",
                "--edges",
                "normal,build,dev",
                "--prefix",
                "depth",
                "--format",
                "{p} [{f}]",
                "--no-dedupe",
            ],
        ),
        (
            "duplicates.txt",
            &[
                "tree",
                "--locked",
                "--workspace",
                "--duplicates",
                "--prefix",
                "depth",
                "--format",
                "{p} [{f}]",
            ],
        ),
        (
            "build-dependencies.txt",
            &[
                "tree",
                "--locked",
                "--workspace",
                "--edges",
                "build",
                "--prefix",
                "depth",
                "--format",
                "{p} [{f}]",
                "--no-dedupe",
            ],
        ),
    ];
    let mut normal_graph = String::new();
    let mut duplicate_graph = String::new();
    let mut build_graph = String::new();
    for (name, args) in graphs {
        let result = run_capture(&cargo, args)?;
        if !result.status.success() {
            return Err(format!(
                "cargo {} failed: {}",
                args.join(" "),
                output_text(&result)
            ));
        }
        let graph = String::from_utf8_lossy(&result.stdout).into_owned();
        match name {
            "workspace-features.txt" => normal_graph = graph.clone(),
            "duplicates.txt" => duplicate_graph = graph.clone(),
            "build-dependencies.txt" => build_graph = graph.clone(),
            _ => unreachable!(),
        }
        let path = output_dir.join(name);
        fs::write(&path, graph).map_err(|error| format!("write {}: {error}", path.display()))?;
    }
    let (package_nodes, direct_nodes, transitive_nodes) = graph_counts(&normal_graph);
    let (_, build_direct_nodes, build_transitive_nodes) = graph_counts(&build_graph);
    let build_dependency_nodes = build_direct_nodes + build_transitive_nodes;
    let duplicate_versions = graph_counts(&duplicate_graph).0;
    let std_feature_rows = normal_graph
        .lines()
        .filter(|line| {
            line.rsplit_once(" [")
                .and_then(|(_, features)| features.strip_suffix(']'))
                .is_some_and(|features| features.split(", ").any(|feature| feature == "std"))
        })
        .count();
    let summary = format!(
        "{{\n  \"schema_version\": 1,\n  \"status\": \"graph-captured\",\n  \"workspace_manifest\": \"Cargo.toml\",\n  \"reports\": [\"workspace-features.txt\", \"duplicates.txt\", \"build-dependencies.txt\"],\n  \"package_nodes_including_workspace_roots\": {package_nodes},\n  \"direct_dependency_nodes\": {direct_nodes},\n  \"transitive_dependency_nodes\": {transitive_nodes},\n  \"build_dependency_package_nodes\": {build_dependency_nodes},\n  \"duplicate_package_version_rows\": {duplicate_versions},\n  \"packages_with_named_std_feature_enabled\": {std_feature_rows},\n  \"dependency_count_baseline\": null,\n  \"proc_macro_classification\": \"package nodes remain in Cargo graph; crate target kind is not classified\",\n  \"linked_native_libraries\": \"not inferred from Cargo metadata; use linkage-audit on a built Mach-O binary\",\n  \"std_requirements\": \"feature graph and named std feature recorded; Cargo does not identify all transitive std use\",\n  \"approval_policy\": \"no growth threshold is configured in this bootstrap\"\n}}\n"
    );
    fs::write(output_dir.join("summary.json"), summary)
        .map_err(|error| format!("write dependency summary: {error}"))?;
    println!("wrote Cargo graphs to {}", output_dir.display());
    println!("inventory only: no dependency growth budget or transitive std proof is claimed");
    Ok(())
}

fn graph_counts(graph: &str) -> (usize, usize, usize) {
    let mut package_nodes = 0;
    let mut direct_nodes = 0;
    let mut transitive_nodes = 0;
    for line in graph.lines() {
        let line = line.trim_start();
        let depth_length = line.bytes().take_while(u8::is_ascii_digit).count();
        if depth_length == 0 {
            continue;
        }
        let Ok(depth) = line[..depth_length].parse::<usize>() else {
            continue;
        };
        let mut fields = line[depth_length..].split_whitespace();
        if fields.next().is_none() || !fields.next().is_some_and(|field| field.starts_with('v')) {
            continue;
        }
        package_nodes += 1;
        if depth == 1 {
            direct_nodes += 1;
        }
        if depth > 1 {
            transitive_nodes += 1;
        }
    }
    (package_nodes, direct_nodes, transitive_nodes)
}

pub(crate) fn abi_audit(args: &[String]) -> Result<(), String> {
    let output = output_path(args, "--output")?;
    let mut sources = Vec::new();
    collect_rs(&root().join("crates"), &mut sources)?;
    let mut declarations = Vec::new();
    let mut creators = Vec::new();
    let mut destroyers = Vec::new();
    let mut abi_major = None;
    let mut abi_minor = None;
    for source in sources {
        let text = fs::read_to_string(&source)
            .map_err(|error| format!("read {}: {error}", source.display()))?;
        let relative = source
            .strip_prefix(root())
            .map_err(|error| error.to_string())?
            .to_string_lossy();
        for (line_index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(value) = constant_value(trimmed, "ABI_VERSION_MAJOR") {
                abi_major = Some(value);
            }
            if let Some(value) = constant_value(trimmed, "ABI_VERSION_MINOR") {
                abi_minor = Some(value);
            }
            let category = if trimmed.contains("#[repr(C)]") {
                Some("repr_c")
            } else if trimmed.contains("#[repr(transparent)]") {
                Some("repr_transparent")
            } else if trimmed.contains("extern \"C\"") || trimmed.contains("extern \"system\"") {
                Some("foreign_call")
            } else if trimmed.contains("no_mangle") {
                Some("exported_symbol")
            } else if trimmed.contains("catch_unwind") {
                Some("panic_containment")
            } else if trimmed.contains("ABI_VERSION_") {
                Some("abi_version")
            } else {
                None
            };
            if let Some(category) = category {
                declarations.push(format!(
                    "{{\"file\":{},\"line\":{},\"category\":{},\"source\":{}}}",
                    json_string(&relative),
                    line_index + 1,
                    json_string(category),
                    json_string(trimmed)
                ));
            }
            if trimmed.contains("fn framework_")
                && (trimmed.contains("_create") || trimmed.contains("_new"))
                && let Some(name) = fn_name(trimmed)
            {
                creators.push(name);
            }
            if trimmed.contains("fn framework_")
                && trimmed.contains("_destroy")
                && let Some(name) = fn_name(trimmed)
            {
                destroyers.push(name);
            }
        }
    }
    let report = format!(
        "{{\n  \"schema_version\": 1,\n  \"status\": \"source-inventory-only\",\n  \"abi_version\": {{\"major\": {}, \"minor\": {}}},\n  \"declarations\": [{}],\n  \"ownership_symbols\": {{\"creator_functions\": {}, \"destroyer_functions\": {}}},\n  \"not_verified\": [\"compiled C header layout\", \"final export list\", \"calling convention at link time\", \"Swift runtime provenance\", \"Apple framework imports\", \"panic behavior across each exported symbol\"]\n}}\n",
        abi_major
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        abi_minor
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        declarations.join(","),
        crate::json_string_array(&creators),
        crate::json_string_array(&destroyers)
    );
    write_report(output, &report)?;
    println!("source inventory only; this report does not certify a linked ABI");
    Ok(())
}

pub(crate) fn linkage_audit(args: &[String]) -> Result<(), String> {
    let mut binary = None;
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--binary" => {
                index += 1;
                binary = Some(PathBuf::from(
                    args.get(index).ok_or("`--binary` requires a path")?,
                ));
            }
            "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).ok_or("`--output` requires a path")?,
                ));
            }
            other => return Err(format!("unexpected argument `{other}`")),
        }
        index += 1;
    }
    let binary = binary.ok_or("linkage-audit requires `--binary PATH`")?;
    let binary = if binary.is_absolute() {
        binary
    } else {
        root().join(binary)
    };
    if !binary.is_file() {
        return Err(format!("binary does not exist: {}", binary.display()));
    }
    let size = fs::metadata(&binary)
        .map_err(|error| error.to_string())?
        .len();
    let result = run_capture("otool", &["-L", &binary.to_string_lossy()])?;
    if !result.status.success() {
        return Err(format!("otool -L failed: {}", output_text(&result)));
    }
    let imports = String::from_utf8_lossy(&result.stdout)
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let joined = imports.join("\n");
    let report = format!(
        "{{\n  \"schema_version\": 1,\n  \"status\": \"binary-inspected\",\n  \"binary\": {},\n  \"bytes\": {},\n  \"imported_libraries\": {},\n  \"swift_runtime_imported\": {},\n  \"python_runtime_imported\": {},\n  \"capability_framework_absence\": \"not asserted; define an expected-import policy before gating\"\n}}\n",
        json_string(&binary.to_string_lossy()),
        size,
        crate::json_string_array(&imports),
        joined.contains("/usr/lib/swift") || joined.contains("libswift"),
        joined.contains("Python.framework") || joined.contains("libpython")
    );
    write_report(output, &report)
}

fn collect_rs(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(directory).map_err(|error| format!("read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            collect_rs(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn constant_value(line: &str, name: &str) -> Option<String> {
    if !line.contains(name) || !line.contains("const") {
        return None;
    }
    line.split_once('=')
        .map(|(_, value)| value.trim().trim_end_matches(';').to_owned())
}

fn fn_name(line: &str) -> Option<String> {
    line.split_once("fn ")?
        .1
        .split(['(', '<', ' '])
        .next()
        .map(str::to_owned)
}

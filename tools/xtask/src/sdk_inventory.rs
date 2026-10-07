use crate::{json_string, json_string_array, run_capture, write_report};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct FrameworkInventory {
    headers: Vec<String>,
    module_maps: Vec<String>,
    swift_interfaces: Vec<String>,
    c_declaration_lines: u64,
    swift_declarations: u64,
    availability_annotations: u64,
    objc_exposed_declarations: u64,
    async_declarations: u64,
    throwing_declarations: u64,
    generic_declarations: u64,
    actor_isolated_declarations: u64,
    protocol_conformance_declarations: u64,
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let mut sdk = "iphoneos".to_owned();
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--sdk" => {
                index += 1;
                sdk = args.get(index).ok_or("`--sdk` requires a name")?.clone();
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
    let names = match sdk.as_str() {
        "iphoneos" => vec!["iphoneos"],
        "iphonesimulator" => vec!["iphonesimulator"],
        "all" => vec!["iphoneos", "iphonesimulator"],
        _ => return Err("`--sdk` must be iphoneos, iphonesimulator, or all".into()),
    };
    let inventories = names
        .iter()
        .map(|name| inventory(name))
        .collect::<Result<Vec<_>, _>>()?;
    let body = format!(
        "{{\n  \"schema_version\": 1,\n  \"status\": \"public-sdk-file-inventory\",\n  \"scope_note\": \"path and declaration-line inventory only; not a public API or compliance review\",\n  \"sdks\": [{}]\n}}\n",
        inventories.join(",")
    );
    let output = output.or_else(|| {
        Some(PathBuf::from(format!(
            "target/xtask/sdk-inventory-{sdk}.json"
        )))
    });
    write_report(output, &body)
}

fn inventory(name: &str) -> Result<String, String> {
    let path_result = run_capture("xcrun", &["--sdk", name, "--show-sdk-path"])?;
    if !path_result.status.success() {
        return Err(format!(
            "cannot locate {name} SDK: {}",
            String::from_utf8_lossy(&path_result.stderr).trim()
        ));
    }
    let sdk_path = PathBuf::from(String::from_utf8_lossy(&path_result.stdout).trim());
    if !sdk_path.is_dir() {
        return Err(format!("SDK path does not exist: {}", sdk_path.display()));
    }
    let version_result = run_capture("xcrun", &["--sdk", name, "--show-sdk-version"])?;
    let version = if version_result.status.success() {
        String::from_utf8_lossy(&version_result.stdout)
            .trim()
            .to_owned()
    } else {
        "unknown".to_owned()
    };
    let mut frameworks = BTreeMap::<String, FrameworkInventory>::new();
    let public_framework_root = sdk_path.join("System/Library/Frameworks");
    if public_framework_root.is_dir() {
        collect_frameworks(&sdk_path, &public_framework_root, &mut frameworks)?;
    }
    let mut root_headers = Vec::new();
    let include_root = sdk_path.join("usr/include");
    if include_root.is_dir() {
        collect_header_paths(&sdk_path, &include_root, &mut root_headers)?;
    }
    let framework_count = frameworks.len();
    let header_count = frameworks
        .values()
        .map(|item| item.headers.len())
        .sum::<usize>()
        + root_headers.len();
    let interface_count = frameworks
        .values()
        .map(|item| item.swift_interfaces.len())
        .sum::<usize>();
    let declarations: Vec<String> = frameworks.iter().map(|(framework, item)| {
        format!(
            "{{\"name\":{},\"headers\":{},\"module_maps\":{},\"swift_interfaces\":{},\"public_c_declaration_lines\":{},\"swift_declarations\":{{\"count\":{},\"availability\":{},\"objc_exposed\":{},\"async\":{},\"throws_or_rethrows\":{},\"generic_signature\":{},\"actor_isolated\":{},\"protocol_conformance\":{}}}}}",
            json_string(framework),
            json_string_array(&item.headers),
            json_string_array(&item.module_maps),
            json_string_array(&item.swift_interfaces),
            item.c_declaration_lines,
            item.swift_declarations,
            item.availability_annotations,
            item.objc_exposed_declarations,
            item.async_declarations,
            item.throwing_declarations,
            item.generic_declarations,
            item.actor_isolated_declarations,
            item.protocol_conformance_declarations
        )
    }).collect();
    Ok(format!(
        "{{\"name\":{},\"version\":{},\"sdk_path\":{},\"framework_count\":{},\"public_header_count\":{},\"swiftinterface_count\":{},\"usr_include_headers\":{},\"frameworks\":[{}]}}",
        json_string(name),
        json_string(&version),
        json_string(&sdk_path.to_string_lossy()),
        framework_count,
        header_count,
        interface_count,
        json_string_array(&root_headers),
        declarations.join(",")
    ))
}

fn collect_frameworks(
    sdk: &Path,
    directory: &Path,
    frameworks: &mut BTreeMap<String, FrameworkInventory>,
) -> Result<(), String> {
    for entry in
        fs::read_dir(directory).map_err(|error| format!("read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_symlink()
        {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() && filename.ends_with(".framework") {
            let framework = filename.trim_end_matches(".framework").to_owned();
            let item = frameworks.entry(framework).or_default();
            collect_framework_files(sdk, &path, item)?;
        } else if path.is_dir() {
            collect_frameworks(sdk, &path, frameworks)?;
        }
    }
    Ok(())
}

fn collect_framework_files(
    sdk: &Path,
    directory: &Path,
    item: &mut FrameworkInventory,
) -> Result<(), String> {
    for entry in
        fs::read_dir(directory).map_err(|error| format!("read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_dir() {
            if entry.file_name() == "PrivateHeaders" {
                continue;
            }
            collect_framework_files(sdk, &path, item)?;
            continue;
        }
        let relative = path
            .strip_prefix(sdk)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned();
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        if filename == "module.modulemap" || filename == "module.private.modulemap" {
            item.module_maps.push(relative);
        } else if path.extension().is_some_and(|extension| extension == "h")
            && (path.components().any(|part| part.as_os_str() == "Headers")
                || path.starts_with(sdk.join("usr/include")))
        {
            item.c_declaration_lines += count_c_declaration_lines(&path)?;
            item.headers.push(relative);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "swiftinterface")
        {
            count_swift_declarations(&path, item)?;
            item.swift_interfaces.push(relative);
        }
    }
    item.headers.sort();
    item.module_maps.sort();
    item.swift_interfaces.sort();
    Ok(())
}

fn collect_header_paths(
    sdk: &Path,
    directory: &Path,
    headers: &mut Vec<String>,
) -> Result<(), String> {
    for entry in
        fs::read_dir(directory).map_err(|error| format!("read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            collect_header_paths(sdk, &path, headers)?;
        } else if path.extension().is_some_and(|extension| extension == "h") {
            headers.push(
                path.strip_prefix(sdk)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    headers.sort();
    Ok(())
}

fn count_c_declaration_lines(path: &Path) -> Result<u64, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let content = String::from_utf8_lossy(&bytes);
    Ok(content
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.starts_with("//")
                && !line.starts_with("/*")
                && (line.contains("FOUNDATION_EXPORT")
                    || line.contains("CF_EXPORT")
                    || line.contains("NS_EXPORT")
                    || line.contains("OBJC_EXPORT")
                    || line.starts_with("@interface ")
                    || line.starts_with("@protocol ")
                    || line.starts_with("@class ")
                    || line.starts_with("extern "))
        })
        .count() as u64)
}

fn count_swift_declarations(path: &Path, item: &mut FrameworkInventory) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let content = String::from_utf8_lossy(&bytes);
    let mut context = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let declaration = (trimmed.contains("public ") || trimmed.contains("open "))
            && [
                "func ",
                "var ",
                "let ",
                "struct ",
                "class ",
                "enum ",
                "protocol ",
                "typealias ",
                "init(",
                "subscript(",
                "actor ",
            ]
            .iter()
            .any(|part| trimmed.contains(part));
        if declaration {
            item.swift_declarations += 1;
            let attributes = context.join(" ");
            if trimmed.contains("@available") || attributes.contains("@available") {
                item.availability_annotations += 1;
            }
            if trimmed.contains("@objc") || attributes.contains("@objc") {
                item.objc_exposed_declarations += 1;
            }
            if trimmed.contains(" async ") {
                item.async_declarations += 1;
            }
            if trimmed.contains(" throws") || trimmed.contains(" rethrows") {
                item.throwing_declarations += 1;
            }
            if trimmed.contains(" where ") || generic_signature(trimmed) {
                item.generic_declarations += 1;
            }
            if [
                "@MainActor",
                "@GlobalActor",
                "nonisolated",
                "isolated ",
                "@preconcurrency",
            ]
            .iter()
            .any(|part| trimmed.contains(part) || attributes.contains(part))
            {
                item.actor_isolated_declarations += 1;
            }
            if is_type_declaration(trimmed)
                && trimmed.split_once(':').is_some_and(|(_, rest)| {
                    !rest.split('{').next().unwrap_or("").trim().is_empty()
                })
            {
                item.protocol_conformance_declarations += 1;
            }
        }
        if declaration {
            context.clear();
        } else if trimmed.starts_with('@') {
            context.push(trimmed.to_owned());
        }
        if context.len() > 6 {
            context.remove(0);
        }
    }
    Ok(())
}

fn generic_signature(line: &str) -> bool {
    let Some((_, rest)) = line.split_once("func ") else {
        return false;
    };
    let Some(open) = rest.find('<') else {
        return false;
    };
    rest.find('(').is_some_and(|paren| open < paren)
}

fn is_type_declaration(line: &str) -> bool {
    ["struct ", "class ", "enum ", "protocol ", "actor "]
        .iter()
        .any(|part| line.contains(part))
}

#[cfg(test)]
mod tests {
    use super::{FrameworkInventory, count_c_declaration_lines, count_swift_declarations};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("sdk-inventory-test-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn c_inventory_counts_exported_declaration_lines_not_comments() {
        let path = temp_file("sample.h");
        fs::write(&path, "// FOUNDATION_EXPORT ignored\nFOUNDATION_EXPORT void sample(void);\nextern int sample_value;\n").unwrap();
        assert_eq!(count_c_declaration_lines(&path).unwrap(), 2);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn swift_inventory_tracks_signature_capabilities() {
        let path = temp_file("sample.swiftinterface");
        fs::write(&path, "@available(iOS 13, *)\n@MainActor\n@objc public func fetch<T>(_ item: T) async throws -> T\npublic struct Item: Codable {}\n").unwrap();
        let mut item = FrameworkInventory::default();
        count_swift_declarations(&path, &mut item).unwrap();
        assert_eq!(item.swift_declarations, 2);
        assert_eq!(item.availability_annotations, 1);
        assert_eq!(item.objc_exposed_declarations, 1);
        assert_eq!(item.async_declarations, 1);
        assert_eq!(item.throwing_declarations, 1);
        assert_eq!(item.generic_declarations, 1);
        assert_eq!(item.actor_isolated_declarations, 1);
        assert_eq!(item.protocol_conformance_declarations, 1);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}

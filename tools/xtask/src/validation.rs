use crate::{json_string, root};
use std::collections::{BTreeSet, VecDeque};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    offline: bool,
    fmt_all: bool,
    clippy_all_targets: bool,
    clippy_lib: bool,
    rustdoc_warnings: bool,
    cargo_tree: bool,
}

const HOMEKIT_TARGETS: &[Target] = &[
    Target {
        triple: "aarch64-apple-ios",
        sdk: "iphoneos",
        minimum_os: "11.3",
    },
    Target {
        triple: "aarch64-apple-ios-sim",
        sdk: "iphonesimulator",
        minimum_os: "14.0",
    },
];
const ACTIVITY_TARGETS: &[Target] = &[
    Target {
        triple: "aarch64-apple-ios",
        sdk: "iphoneos",
        minimum_os: "16.1",
    },
    Target {
        triple: "aarch64-apple-ios-sim",
        sdk: "iphonesimulator",
        minimum_os: "16.1",
    },
];
const PHOTO_TARGETS: &[Target] = &[
    Target {
        triple: "aarch64-apple-ios",
        sdk: "iphoneos",
        minimum_os: "17.0",
    },
    Target {
        triple: "aarch64-apple-ios-sim",
        sdk: "iphonesimulator",
        minimum_os: "17.0",
    },
    Target {
        triple: "x86_64-apple-ios",
        sdk: "iphonesimulator",
        minimum_os: "17.0",
    },
];
const ARM64_STATUS_TARGETS: &[Target] = &[
    Target {
        triple: "aarch64-apple-ios",
        sdk: "iphoneos",
        minimum_os: "17.0",
    },
    Target {
        triple: "aarch64-apple-ios-sim",
        sdk: "iphonesimulator",
        minimum_os: "17.0",
    },
];
const ALARM_TARGETS: &[Target] = &[
    Target {
        triple: "aarch64-apple-ios",
        sdk: "iphoneos",
        minimum_os: "26.0",
    },
    Target {
        triple: "aarch64-apple-ios-sim",
        sdk: "iphonesimulator",
        minimum_os: "26.0",
    },
];

const HOMEKIT_GUARDS: &[Guard] = &[
    Guard {
        path: "platform/ios/ios-homekit-identify-status/src/lib.rs",
        text: "unsafe { accessory.supportsIdentify() }",
        must_exist: true,
        message: "typed HomeKit supportsIdentify getter is required",
    },
    Guard {
        path: "platform/ios/ios-homekit-identify-status/src/lib.rs",
        text: "objc2::available!(ios = 11.3, ..)",
        must_exist: true,
        message: "iOS 11.3 availability guard is required",
    },
    Guard {
        path: "platform/ios/ios-homekit-identify-status/src/lib.rs",
        text: "HMHomeManager::",
        must_exist: false,
        message: "HomeKit manager lifecycle is outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-homekit-identify-status/src/lib.rs",
        text: "requestAuthorization(",
        must_exist: false,
        message: "HomeKit authorization is outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-homekit-identify-status/src/lib.rs",
        text: ".identifyWithCompletionHandler(",
        must_exist: false,
        message: "HomeKit identify operation is outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-homekit-identify-status/src/lib.rs",
        text: ".identify(",
        must_exist: false,
        message: "HomeKit identify operation is outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-homekit-identify-status/Cargo.toml",
        text: "objc2-home-kit = { version = \"=0.3.2\", default-features = false, features = [\"HMAccessory\"] }",
        must_exist: true,
        message: "HomeKit dependency and feature scope changed",
    },
];
const ACTIVITY_GUARDS: &[Guard] = &[
    Guard {
        path: "platform/ios/ios-activitykit-status/Cargo.toml",
        text: "features = [\"apple-runtime\"]",
        must_exist: true,
        message: "swift-abi-core apple-runtime feature is required",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh",
        text: "ActivityAuthorizationInfo().areActivitiesEnabled",
        must_exist: true,
        message: "ActivityKit compiler oracle must retain the selected getter",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: "0A17AuthorizationInfoC20areActivitiesEnabledSbvg",
        must_exist: true,
        message: "ActivityKit native thunk must retain the compiler-derived getter symbol",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: "Activity.request",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: "activityEnablementUpdates",
        must_exist: false,
        message: "ActivityKit update streams are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: "frequentPushesEnabled",
        must_exist: false,
        message: "ActivityKit push settings are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: ".update(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: ".end(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: "ActivityAttributes",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/src/lib.rs",
        text: "WidgetCenter",
        must_exist: false,
        message: "WidgetKit APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: "Activity.request",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: ".update(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: ".end(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: "ActivityAttributes",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: "Activity.request",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: "activityEnablementUpdates",
        must_exist: false,
        message: "ActivityKit update streams are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: "frequentPushesEnabled",
        must_exist: false,
        message: "ActivityKit push settings are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: ".update(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: ".end(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: "ActivityAttributes",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
        text: "WidgetCenter",
        must_exist: false,
        message: "WidgetKit APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: "activityEnablementUpdates",
        must_exist: false,
        message: "ActivityKit update streams are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: "frequentPushesEnabled",
        must_exist: false,
        message: "ActivityKit push settings are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/native/activitykit_status.c",
        text: "WidgetCenter",
        must_exist: false,
        message: "WidgetKit APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: "activityEnablementUpdates",
        must_exist: false,
        message: "ActivityKit update streams are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: "frequentPushesEnabled",
        must_exist: false,
        message: "ActivityKit push settings are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: "Activity.request",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: ".update(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: ".end(",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: "ActivityAttributes",
        must_exist: false,
        message: "ActivityKit lifecycle APIs are outside this pilot",
    },
    Guard {
        path: "platform/ios/ios-activitykit-status/build.rs",
        text: "WidgetCenter",
        must_exist: false,
        message: "WidgetKit APIs are outside this pilot",
    },
];
const ALARM_GUARDS: &[Guard] = &[Guard {
    path: "platform/ios/ios-alarmkit-status/build.rs",
    text: "frameworks: &[\"AlarmKit\"]",
    must_exist: true,
    message: "AlarmKit must remain the only public native framework link for this pilot",
}];
const PHOTO_GUARDS: &[Guard] = &[];

const HOMEKIT_SCOPED: &[&str] = &[
    "platform/ios/ios-homekit-identify-status/src/lib.rs",
    "platform/ios/ios-homekit-identify-status/Cargo.toml",
    "platform/ios/ios-homekit-identify-status/check.sh",
    "platform/ios/ios-homekit-identify-status/check-link-imports.sh",
    "platform/ios/ios-homekit-identify-status/examples/homekit_identify_link_probe.rs",
    "docs/ios/homekit-identify-status.md",
    "PLAN_CAPABILITIES_HOMEKIT.md",
];
const ALARM_SCOPED: &[&str] = &[
    "interop/swift-abi-core/include/swift_abi_runtime.h",
    "PLAN_CAPABILITIES_ALARMKIT.md",
    "PLAN_SWIFT_ABI.md",
    "docs/ios/alarmkit-status.md",
    ".github/workflows/ci.yml",
    "platform/ios/ios-alarmkit-status/Cargo.toml",
    "platform/ios/ios-alarmkit-status/build.rs",
    "platform/ios/ios-alarmkit-status/check.sh",
    "platform/ios/ios-alarmkit-status/check-link-imports.sh",
    "platform/ios/ios-alarmkit-status/native/alarmkit_status.c",
    "platform/ios/ios-alarmkit-status/src/lib.rs",
    "platform/ios/ios-alarmkit-status/examples/alarmkit_status_link_probe.rs",
];
const ACTIVITY_SCOPED: &[&str] = &[
    "interop/swift-abi-core/include/swift_abi_runtime.h",
    "PLAN_CAPABILITIES_ACTIVITYKIT.md",
    "platform/ios/ios-activitykit-status/Cargo.toml",
    "platform/ios/ios-activitykit-status/README.md",
    "platform/ios/ios-activitykit-status/check.sh",
    "platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh",
    "platform/ios/ios-activitykit-status/scripts/check-link-imports.sh",
    "platform/ios/ios-activitykit-status/src/lib.rs",
    "platform/ios/ios-activitykit-status/native/activitykit_status.c",
    "platform/ios/ios-activitykit-status/build.rs",
    "platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs",
    "docs/ios/activitykit-status.md",
];
const PHOTO_SCOPED: &[&str] = &[
    "interop/swift-abi-core/include/swift_abi_runtime.h",
    "platform/ios/ios-photogrammetry-status/Cargo.toml",
    "platform/ios/ios-photogrammetry-status/check.sh",
    "platform/ios/ios-photogrammetry-status/check-swiftcall.sh",
    "platform/ios/ios-photogrammetry-status/check-link-imports.sh",
    "platform/ios/ios-photogrammetry-status/src/lib.rs",
    "platform/ios/ios-photogrammetry-status/native/photogrammetry_status.c",
    "platform/ios/ios-photogrammetry-status/build.rs",
    "platform/ios/ios-photogrammetry-status/examples/photogrammetry_status_link_probe.rs",
    "PLAN_CAPABILITIES_REALITYKIT.md",
    "docs/ios/photogrammetry-status.md",
];

const PILOTS: &[Pilot] = &[
    Pilot {
        id: "ios-homekit-identify-status",
        package: "ios-homekit-identify-status",
        manifest: "platform/ios/ios-homekit-identify-status/Cargo.toml",
        check_script: "platform/ios/ios-homekit-identify-status/check.sh",
        targets: HOMEKIT_TARGETS,
        abi_targets: HOMEKIT_TARGETS,
        expected_imports: &["HomeKit.framework", "objc_msgSend"],
        abi_scripts: &["platform/ios/ios-homekit-identify-status/check-link-imports.sh"],
        shell_scripts: &[
            "platform/ios/ios-homekit-identify-status/check.sh",
            "platform/ios/ios-homekit-identify-status/check-link-imports.sh",
        ],
        docs: &["docs/ios/homekit-identify-status.md"],
        scoped_files: HOMEKIT_SCOPED,
        source_dirs: &["platform/ios/ios-homekit-identify-status"],
        guards: HOMEKIT_GUARDS,
        offline: false,
        fmt_all: false,
        clippy_all_targets: true,
        clippy_lib: false,
        rustdoc_warnings: false,
        cargo_tree: false,
    },
    Pilot {
        id: "ios-photogrammetry-status",
        package: "ios-photogrammetry-status",
        manifest: "platform/ios/ios-photogrammetry-status/Cargo.toml",
        check_script: "platform/ios/ios-photogrammetry-status/check.sh",
        targets: PHOTO_TARGETS,
        abi_targets: ARM64_STATUS_TARGETS,
        expected_imports: &[
            "RealityFoundation.framework",
            "PhotogrammetrySession.isSupported",
            "PhotogrammetrySession.limits",
        ],
        abi_scripts: &[
            "platform/ios/ios-photogrammetry-status/check-swiftcall.sh",
            "platform/ios/ios-photogrammetry-status/check-link-imports.sh",
        ],
        shell_scripts: &[
            "platform/ios/ios-photogrammetry-status/check.sh",
            "platform/ios/ios-photogrammetry-status/check-swiftcall.sh",
            "platform/ios/ios-photogrammetry-status/check-link-imports.sh",
        ],
        docs: &["docs/ios/photogrammetry-status.md"],
        scoped_files: PHOTO_SCOPED,
        source_dirs: &["platform/ios/ios-photogrammetry-status"],
        guards: PHOTO_GUARDS,
        offline: false,
        fmt_all: true,
        clippy_all_targets: false,
        clippy_lib: true,
        rustdoc_warnings: true,
        cargo_tree: false,
    },
    Pilot {
        id: "ios-alarmkit-status",
        package: "ios-alarmkit-status",
        manifest: "platform/ios/ios-alarmkit-status/Cargo.toml",
        check_script: "platform/ios/ios-alarmkit-status/check.sh",
        targets: ALARM_TARGETS,
        abi_targets: ALARM_TARGETS,
        expected_imports: &["AlarmKit.framework"],
        abi_scripts: &["platform/ios/ios-alarmkit-status/check-link-imports.sh"],
        shell_scripts: &[
            "platform/ios/ios-alarmkit-status/check.sh",
            "platform/ios/ios-alarmkit-status/check-link-imports.sh",
        ],
        docs: &["docs/ios/alarmkit-status.md"],
        scoped_files: ALARM_SCOPED,
        source_dirs: &["platform/ios/ios-alarmkit-status"],
        guards: ALARM_GUARDS,
        offline: true,
        fmt_all: false,
        clippy_all_targets: false,
        clippy_lib: true,
        rustdoc_warnings: true,
        cargo_tree: false,
    },
    Pilot {
        id: "ios-activitykit-status",
        package: "ios-activitykit-status",
        manifest: "platform/ios/ios-activitykit-status/Cargo.toml",
        check_script: "platform/ios/ios-activitykit-status/check.sh",
        targets: ACTIVITY_TARGETS,
        abi_targets: ACTIVITY_TARGETS,
        expected_imports: &[
            "ActivityKit.framework (weak)",
            "ActivityKit.AuthorizationInfo.areActivitiesEnabled",
        ],
        abi_scripts: &[
            "platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh",
            "platform/ios/ios-activitykit-status/scripts/check-link-imports.sh",
        ],
        shell_scripts: &[
            "platform/ios/ios-activitykit-status/check.sh",
            "platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh",
            "platform/ios/ios-activitykit-status/scripts/check-link-imports.sh",
        ],
        docs: &["docs/ios/activitykit-status.md"],
        scoped_files: ACTIVITY_SCOPED,
        source_dirs: &["platform/ios/ios-activitykit-status"],
        guards: ACTIVITY_GUARDS,
        offline: false,
        fmt_all: false,
        clippy_all_targets: false,
        clippy_lib: true,
        rustdoc_warnings: true,
        cargo_tree: true,
    },
];

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

struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run(
        &mut self,
        program: &str,
        args: &[String],
        env: &[(String, String)],
    ) -> io::Result<CommandResult> {
        let mut command = Command::new(program);
        command.args(args).current_dir(root());
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
    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout.to_owned(),
        (true, false) => stderr.to_owned(),
        (true, true) => String::new(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    Passed,
    Failed,
    Skipped,
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

fn source_guards(pilot: &Pilot) -> Vec<GateResult> {
    pilot
        .guards
        .iter()
        .map(|guard| {
            let content = std::fs::read_to_string(root().join(guard.path));
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

fn run_pilot(pilot: &Pilot, runner: &mut impl CommandRunner) -> Vec<GateResult> {
    let mut results = source_guards(pilot);
    for directory in pilot.source_dirs {
        let mut files = Vec::new();
        let result = find_swift_files(&root().join(directory), &mut files);
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
        let exists = root().join(path).is_file();
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
        let content = std::fs::read_to_string(root().join(path));
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
        Outcome::Failed => "fail",
        Outcome::Skipped => "skipped",
    }
}

fn render(results: &[GateResult], json: bool) {
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
                    json_string(&result.detail)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"schema_version\":1,\"results\":[{records}]}}");
    } else {
        for result in results {
            if result.detail.is_empty() {
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
                    result.detail
                );
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyGraph {
    edges: Vec<(String, String)>,
}

impl DependencyGraph {
    fn pilot_graph() -> Self {
        Self {
            edges: vec![
                ("ios-activitykit-status".into(), "swift-abi-core".into()),
                (
                    "ios-activitykit-status".into(),
                    "ios-native-build-support".into(),
                ),
                ("ios-alarmkit-status".into(), "swift-abi-core".into()),
                ("ios-photogrammetry-status".into(), "swift-abi-core".into()),
                (
                    "ios-alarmkit-status".into(),
                    "ios-native-build-support".into(),
                ),
                (
                    "ios-photogrammetry-status".into(),
                    "ios-native-build-support".into(),
                ),
                (
                    "ios-homekit-identify-status".into(),
                    "objc2-home-kit".into(),
                ),
            ],
        }
    }

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

fn changed_nodes(paths: &[String]) -> (bool, BTreeSet<String>) {
    let mut global = false;
    let mut nodes = BTreeSet::new();
    for path in paths {
        if matches!(
            path.as_str(),
            "Cargo.toml" | "Cargo.lock" | "rust-toolchain.toml" | ".github/workflows/ci.yml"
        ) || path.starts_with(".cargo/")
            || path.starts_with("docs/")
            || path.starts_with("tools/xtask/")
            || path.starts_with("tools/validation-harness/")
        {
            global = true;
        }
        if path.starts_with("tools/native-build-support/") {
            nodes.insert("ios-native-build-support".into());
        }
        if path.starts_with("interop/swift-abi-core/")
            || path.starts_with("interop/swift-abi-generated/")
        {
            nodes.insert("swift-abi-core".into());
        }
        for pilot in PILOTS {
            if path.starts_with(&format!("platform/ios/{}/", pilot.package)) {
                nodes.insert(pilot.package.into());
            }
            if pilot.scoped_files.contains(&path.as_str()) {
                nodes.insert(pilot.package.into());
            }
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

fn pilot(id: &str) -> Option<&'static Pilot> {
    PILOTS.iter().find(|pilot| pilot.id == id)
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
            format!(
                "{{\"triple\":{},\"sdk\":{},\"minimum_os\":{}}}",
                json_string(target.triple),
                json_string(target.sdk),
                json_string(target.minimum_os)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let guards = pilot
        .guards
        .iter()
        .map(|guard| {
            format!(
                "{{\"path\":{},\"text\":{},\"expect\":{},\"reason\":{}}}",
                json_string(guard.path),
                json_string(guard.text),
                json_string(if guard.must_exist {
                    "present"
                } else {
                    "absent"
                }),
                json_string(guard.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let abi_targets = pilot
        .abi_targets
        .iter()
        .map(|target| json_string(target.triple))
        .collect::<Vec<_>>()
        .join(",");
    let commands = build_steps(pilot)
        .iter()
        .map(|step| {
            format!(
                "{{\"phase\":{},\"program\":{},\"args\":[{}]}}",
                json_string(&step.phase),
                json_string(&step.program),
                step.args
                    .iter()
                    .map(|arg| json_string(arg))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"id\":{},\"package\":{},\"manifest\":{},\"targets\":[{}],\"abi_targets\":[{}],\"expected_imports\":[{}],\"abi_fixtures\":[{}],\"docs\":[{}],\"guards\":[{}],\"commands\":[{}]}}",
        json_string(pilot.id),
        json_string(pilot.package),
        json_string(pilot.manifest),
        targets,
        abi_targets,
        pilot
            .expected_imports
            .iter()
            .map(|item| json_string(item))
            .collect::<Vec<_>>()
            .join(","),
        pilot
            .abi_scripts
            .iter()
            .map(|item| json_string(item))
            .collect::<Vec<_>>()
            .join(","),
        pilot
            .docs
            .iter()
            .map(|item| json_string(item))
            .collect::<Vec<_>>()
            .join(","),
        guards,
        commands
    );
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
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
    let (mode, value) = mode.ok_or_else(|| "usage: cargo xtask validate (--list | --explain ID | --capability ID | --changed BASE | --all) [--format human|json]".to_owned())?;
    if mode == "--list" {
        if format_json {
            println!(
                "[{}]",
                PILOTS
                    .iter()
                    .map(|pilot| json_string(pilot.id))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        } else {
            for pilot in PILOTS {
                println!("{}", pilot.id);
            }
        }
        return Ok(());
    }
    if mode == "--explain" {
        let spec = pilot(value.unwrap())
            .ok_or_else(|| format!("unknown capability `{}`", value.unwrap()))?;
        if format_json {
            explain_json(spec);
        } else {
            explain(spec);
        }
        return Ok(());
    }
    let mut runner = ProcessRunner;
    let selected: Vec<&'static Pilot> = match mode {
        "--capability" => vec![
            pilot(value.unwrap())
                .ok_or_else(|| format!("unknown capability `{}`", value.unwrap()))?,
        ],
        "--all" => PILOTS.iter().collect(),
        "--changed" => {
            let paths = changed_paths(value.unwrap(), &mut runner)?;
            let (global, nodes) = changed_nodes(&paths);
            if global {
                PILOTS.iter().collect()
            } else {
                let affected = DependencyGraph::pilot_graph().reverse_closure(nodes);
                PILOTS
                    .iter()
                    .filter(|pilot| affected.contains(pilot.package))
                    .collect()
            }
        }
        _ => unreachable!(),
    };
    if selected.is_empty() {
        println!("no validation pilots affected by changed files");
        return Ok(());
    }
    let mut results = Vec::new();
    for pilot in selected {
        results.extend(run_pilot(pilot, &mut runner));
    }
    render(&results, format_json);
    if results
        .iter()
        .any(|result| result.outcome == Outcome::Failed)
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
        let (_, native) = changed_nodes(&["tools/native-build-support/src/lib.rs".into()]);
        let affected = DependencyGraph::pilot_graph().reverse_closure(native);
        assert!(affected.contains("ios-activitykit-status"));
        assert!(affected.contains("ios-alarmkit-status"));
        assert!(affected.contains("ios-photogrammetry-status"));
        let (_, abi) = changed_nodes(&["interop/swift-abi-core/src/lib.rs".into()]);
        let affected = DependencyGraph::pilot_graph().reverse_closure(abi);
        assert!(affected.contains("ios-activitykit-status"));
        assert!(affected.contains("ios-alarmkit-status"));
        assert!(affected.contains("ios-photogrammetry-status"));
        assert!(!affected.contains("ios-homekit-identify-status"));
        let (_, generated) = changed_nodes(&["interop/swift-abi-generated/src/lib.rs".into()]);
        let affected = DependencyGraph::pilot_graph().reverse_closure(generated);
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
    fn status_names_keep_skipped_distinct_from_passed() {
        assert_eq!(status_name(Outcome::Passed), "pass");
        assert_eq!(status_name(Outcome::Failed), "fail");
        assert_eq!(status_name(Outcome::Skipped), "skipped");
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
            target: Some(HOMEKIT_TARGETS[0]),
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
            include_str!("../fixtures/validation/ios-homekit-identify-status.commands").trim_end()
        );
        assert_eq!(
            command_snapshot(pilot("ios-photogrammetry-status").unwrap()),
            include_str!("../fixtures/validation/ios-photogrammetry-status.commands").trim_end()
        );
        assert_eq!(
            command_snapshot(pilot("ios-alarmkit-status").unwrap()),
            include_str!("../fixtures/validation/ios-alarmkit-status.commands").trim_end()
        );
        assert_eq!(
            command_snapshot(pilot("ios-activitykit-status").unwrap()),
            include_str!("../fixtures/validation/ios-activitykit-status.commands").trim_end()
        );
    }
}

# Shared iOS Rust build and validation tools

**Audience:** ordinary API-development Codex sessions, human maintainers, and separately authorized tooling-maintenance sessions. **Contract:** use pinned PATH tools without reading engine source. The repository's product behavior and ABI invariants in `AGENTS.md` remain authoritative.

> **Scope and evidence:** This handbook describes the interfaces verified in repository `Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework` at commit `8b68d2909de9b5e926130265ba241c80ab653e99`. It is a usage guide, not a certification that all CI checks pass. At the inspected commit, shared-tool artifact CI passed while ordinary macOS and Ubuntu CI each retained a separately observed failure. Verify the current `main` SHA and current binary `--help`/`--version` before acting on a later revision.

## 1. Quick start (read this first)

From the repository root on a supported build host:

```sh
# Offline install of checked-in immutable, SHA-256-verified archives.
tools/install-tools.sh --prefix "$PWD/target/ios-rust-tools"
export PATH="$PWD/target/ios-rust-tools/bin:$PATH"

# Verify the executables selected by PATH and inspect compatibility metadata.
command -v ios-rust-build
command -v ios-rust-validate
ios-rust-build --version --format json
ios-rust-validate --version --format json

# Discover and explain registered checks, without opening engine source.
ios-rust-validate --workspace-root "$PWD" \
  --spec "$PWD/tools/validation/specs/validation-v1.json" --list
ios-rust-validate --workspace-root "$PWD" \
  --spec "$PWD/tools/validation/specs/validation-v1.json" \
  --explain ios-homekit-identify-status

# Run only the selected capability's checks.
ios-rust-validate --workspace-root "$PWD" \
  --spec "$PWD/tools/validation/specs/validation-v1.json" \
  --capability ios-homekit-identify-status
```

The pinned tools are host executables. They are **not** embedded into shipping iOS binaries. Python is needed only when a selected validation specification declares a Python adapter. Do not confuse this with the optional Python *binding* layer of the framework.

### Currently registered pilot capabilities

- `ios-homekit-identify-status`: Objective-C HomeKit getter, availability and negative assertions.
- `ios-activitykit-status`: ActivityKit public getter, Swift compiler ABI oracle and import/link assertions.
- `ios-alarmkit-status`: AlarmKit authorization state, availability, weak-link/import assertions.
- `ios-photogrammetry-status`: RealityFoundation hardware-support status, Swift ABI/value-witness and linkage checks; additionally covers supported Intel simulator target.

These are **pilot tests**, not proof that every Apple framework capability is supported or fully implemented. Use `--list` on the installed tool to confirm the effective list. The optional Python HomeKit adapter is `tools/validation/adapters/check_homekit_contract.py`.

## 2. Responsibility boundaries

| What changes | Ordinary API Codex may edit? | Where to start |
|---|---|---|
| Platform API Rust/C implementation | Yes, within assigned API | `platform/ios/<capability>/src`, `native` |
| API-specific validation declaration | Yes | `tools/validation/specs/validation-v1.json` |
| API-specific Python 3 validation adapter | Yes, if needed | `tools/validation/adapters/` |
| API-specific link/Swift compiler oracle, fixtures | Yes, if needed | Existing package `scripts/`, `check-*.sh`, examples |
| New C archive configuration | Yes | Per-package `build-spec.json` and minimal `build.rs` |
| Pinned release archives, release manifest, installed binary internals | **No** | Separate maintenance session |
| Shared Rust engine source / historic engine revision | **No** | Separate maintenance session only |
| Existing non-validation `xtask` utilities | Only when independently authorized | `tools/xtask/` |

The source-isolation arrangement prevents accidental source searches in the current tree; it does **not** prohibit access to historical Git objects or remote GitHub. Observe the workflow boundary even when such access is possible.

## 3. Tool installation, integrity and platforms

Canonical files:

- `tools/install-tools.sh`: offline installer and verification.
- `tools/releases/manifest-v1.tsv`: pin by **host triple**, release ID, full immutable source SHA, compiler/tool versions, archive path and SHA-256.
- `tools/releases/ios-rust-tools-<host>.tar.gz`: checked-in host archives with binary checksums and metadata.
- `tools/releases/README.md`: provenance, build instructions and release expectations.
- `.github/workflows/shared-tools.yml`: maintenance build workflow (checks out immutable historical source) and artifact verification.

Supported pins in the inspected release: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`. The installer chooses from OS + machine architecture. Other hosts fail closed; do not substitute an unverified archive or claim the target is supported. Rust 1.94.1 is the version used in the recorded maintenance build workflow; individual framework packages may have separate minimum Rust versions.

**Installation checklist:** Use an explicit writable `--prefix`; put `<prefix>/bin` before unrelated copies in PATH; inspect both `--version --format json` reports; retain the exact manifest/archives. The installer makes no network requests, verifies SHA-256, validates archive members and metadata, and rejects missing or mismatched pins. On CI, installation is shown in `.github/workflows/ci.yml` and prepends the installation path to `GITHUB_PATH`.

**What not to do:** avoid `cargo install` from the ordinary checkout, downloading “latest”, substituting host architectures, overwriting a validated pin without coordinated release work, and recompiling source from Git history in ordinary API sessions.

**Troubleshooting:** `command -v ios-rust-build` or `command -v ios-rust-validate` identifies the executable actually invoked. If not found, repeat the offline installer with a supported host and fix PATH. If the installer rejects checksum, metadata, host or release ID, stop: re-download only through the separately reviewed release process. If multiple copies are installed, do not silently fall back to another version. An incompatible tool version is a **blocking error**.

## 4. Build engine: `ios-rust-build`

### Normal use

For ordinary API development, run Cargo as usual after installing the tool. Only the three existing native pilot packages have minimal `build.rs` bridges:

- `platform/ios/ios-activitykit-status/build.rs`
- `platform/ios/ios-alarmkit-status/build.rs`
- `platform/ios/ios-photogrammetry-status/build.rs`

Cargo invokes the corresponding Rust build script; it checks the PATH executable's name, tool version, schema major and host and invokes `ios-rust-build build ...` with explicit arguments. The bridge reads **JSON result directives** and emits allow-listed Cargo directives. The external binary performs SDK discovery, C compile and archive creation. The bridge is product-integration glue, not another shared engine.

Example ordinary build on a configured macOS host with the device target installed:

```sh
cargo +1.94.1 build --locked -p ios-activitykit-status --target aarch64-apple-ios
```

The direct engine interface, used mainly for diagnosis, includes:

```sh
ios-rust-build --help
ios-rust-build build --workspace-root "$PWD" \
  --spec "$PWD/platform/ios/ios-activitykit-status/build-spec.json" \
  --target aarch64-apple-ios --target-os ios \
  --out-dir /path/to/cargo/OUT_DIR \
  --manifest-dir "$PWD/platform/ios/ios-activitykit-status" --format json
```

`--out-dir` above is a placeholder for a real writable Cargo output directory; ordinary Codex users should normally use `cargo build` rather than manually inventing this path. Do not inject global link flags into all workspace crates.

### Build specification contract

Source of truth: `tools/native-build/specs/schema-v1.json`, with examples in the three `build-spec.json` files. These schema-v1 fields are **required**:

| Field | Meaning |
|---|---|
| `schema_version` | Must be `1` for this release |
| `capability` | Human-readable capability ID |
| `library` | Archive/link name, identifier-constrained |
| `sources` | Capability-relative C source paths |
| `headers` | Header dependencies (including shared public ABI headers), for rerun tracking |
| `frameworks` | Only public Apple frameworks the capability actually needs |
| `minimum_os` | Explicit deployment floor, e.g. `16.1` |
| `targets` | Supported `{triple,sdk}` pairs (schema enumerates permitted triples) |
| `feature_guards` | Explicit named Boolean Cargo feature guards |
| `compiler_options` | Capability-specific permitted options; avoid unreviewed flags |
| `compile_description`, `archive_description` | Descriptions for diagnostic/provenance review |

The engine validates paths and rejects escapes outside the workspace. Keep configs declarative, avoid ad hoc environment-dependent logic, and preserve supported target behavior. When adding an API that **does not require native C compilation**, do **not** create a `build.rs` merely to mimic existing pilots. If native compilation is necessary, copy the minimal integration pattern while preserving relevant platform/ABI semantics; the exact code is a per-package integration contract, not a new general engine.

### Cargo bridge expectations

- Validate executable `--version --format json`, status, name, `schema_major`, host triple.
- Invoke executable via `std::process::Command` with argv (not shell).
- Use explicit workspace root, spec and Cargo target/output environment.
- Parse structured success response; reject unexpected, missing, malformed or injected directives.
- Allow only known Cargo rerun/link directives and reject newlines/NUL bytes.
- Track specification, native sources, headers and environment changes so incremental builds cannot silently reuse stale archives.
- On non-iOS targets perform no native Apple linking. Device/simulator deployment floors, public imports and weak symbols remain intact.

### What build success proves

A host `cargo check` is not proof of iOS linkage; an iOS target `cargo check` is not a completed binary link; `nm`, `otool` and `vtool` on an Apple artifact give symbol, dependency and deployment metadata but not device-runtime behavior. Match the required evidence type to the capability.

## 5. Validation engine: `ios-rust-validate`

### Command reference

Every command below takes the explicit `--workspace-root` and `--spec` arguments. This reduces dependence on the current working directory and makes configuration reproducible.

```sh
ROOT="$PWD"
SPEC="$ROOT/tools/validation/specs/validation-v1.json"

ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --help
ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --list
ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --explain ios-alarmkit-status
ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --capability ios-alarmkit-status
ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --all
ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --changed HEAD~1
ios-rust-validate --workspace-root "$ROOT" --spec "$SPEC" --all --format json
```

`HEAD~1` is an illustration: choose an **explicit, valid Git base** appropriate to the real branch, especially after shallow clones or rebases. Do not assume an unavailable ancestor exists. The effective `--help` and `--explain ID` output take priority if the installed tool uses a more restrictive argument order.

| Operation | Purpose | Avoid |
|---|---|---|
| `--list` | Inspect registered capability IDs | Reading engine source to find IDs |
| `--explain ID` | Inspect resolved gates, targets, paths, ABI evidence and adapter requirements | Guessing coverage |
| `--capability ID` | Run a focused capability | Running full workspace unnecessarily |
| `--changed EXPLICIT_BASE` | Select impacted pilots from worktree/Git diff and reverse dependencies | Treating diff omissions as proof unrelated code is safe |
| `--all` | Run every registered pilot | Claiming it validates APIs not registered in spec |
| `--format json` | Obtain versioned machine-readable report | Scraping human output for automation |

Validation results distinguish PASS, FAIL, SKIPPED(reason) and ERROR-BLOCKED. When a required gate is skipped because a toolchain/SDK is unavailable, it is *not* a passing verification. Record which gates actually executed and exactly what evidence they provide.

### Validation spec structure

Source of truth: `tools/validation/specs/schema-v1.json`. The top-level spec has four properties:

```json
{
  "schema_version": 1,
  "dependency_edges": [],
  "path_rules": [],
  "pilots": []
}
```

- `dependency_edges` describe declared dependencies between capability/shared nodes. Keep direction consistent with the existing spec and test reverse closure when adding shared dependencies.
- `path_rules` map repository-relative paths or prefixes to a dependency node or to `global: true` to select all pilots. For example, a modification to the shared validator schema or workspace config should conservatively select all relevant pilots.
- `pilots` are registered validation profiles. A pilot declares package, manifest, check script, targets, ABI targets, expected imports, ABI and shell scripts, required docs/scoped source files, required/forbidden textual guards, Python adapters, and standard Cargo check/lint/doc flags. It is a **validation profile**, not proof of full API support.

All pilot fields enumerated by the schema are required; use an empty array when a supported optional collection has no entries. Important pilot flags include `offline`, `fmt_all`, `clippy_all_targets`, `clippy_lib`, `rustdoc_warnings`, `cargo_tree`. Use existing profiles as examples and preserve unique negative guards. The schema constrains target triples to `aarch64-apple-ios`, `aarch64-apple-ios-sim`, and `x86_64-apple-ios` as currently encoded. Changes to target support are **tool-maintenance issues** if the installed engine cannot express the new target.

### Standard API implementation procedure

1. Identify the existing capability package, public contract, deployment floor and tests; read only its relevant product source and local docs.
2. Install/check the pinned binary if not already available; run `--explain ID`. Do not inspect historical engine code.
3. Make the scoped product change. Extend an existing capability pilot's declarative guards and fixtures. For a new profile, add an ID, package, manifest, targets, accurate import/ABI expectations, source paths, docs, coverage rules and any needed adapter. Do not invent public Apple symbols or ABI signatures.
4. Prefer built-in checks for fmt, Cargo check/Clippy/rustdoc, link/import scans, source guards, ABI oracles and docs. Add Python only for behavior a declaration cannot faithfully express.
5. Run `--capability ID`; test one **negative regression** relevant to the original failure mode (e.g., remove required guard or introduce forbidden call in an isolated fixture/worktree), confirm FAIL or ERROR-BLOCKED, then restore test fixture. Run `--changed EXPLICIT_BASE` for reverse dependency coverage and broader checks when shared surfaces changed.
6. Report exact validation commands, completed gates, skipped/unavailable gates, artifact types and remaining risks. Never substitute source-string matching for compiler or runtime proof.

### Required versus optional checks

An adapter with `required: true` is an acceptance gate. Missing Python interpreter, timeout, invalid JSON, malformed status, nonzero exit, missing script, or unsafe input must not be treated as success. An adapter with `required: false` can be omitted only where the declared contract allows it; report the absence explicitly. A failure in a required gate blocks completion even when other checks pass.

## 6. Python 3 adapter protocol

Adapters are short, capability-specific subprocesses running outside shipping applications; they are not part of the native Rust fast path. The protocol is **JSON on stdin and JSON on stdout**, with diagnostics allowed on stderr. No shell invocation, no global module import into the engine. Existing examples: `tools/validation/adapters/check_homekit_contract.py`, `tools/validation/fixtures/passing_adapter.py`, `invalid_adapter.py`.

Each adapter declaration requires these schema-v1 fields:

| Property | Purpose |
|---|---|
| `id` | Unique adapter identifier |
| `interpreter` | Currently literal `python3` |
| `minimum_version` | Python version floor (e.g. `3.10`) |
| `script` | Repository-relative adapter file |
| `input_paths` | Explicit source/fixture files accessible to the adapter |
| `required` | Whether adapter success is mandatory |
| `evidence_type` | What the adapter actually proves |
| `deadline_seconds` | Per-adapter runtime deadline, 1–300 seconds |
| `stdout_limit_bytes`, `stderr_limit_bytes` | Output caps, at most 1 MiB each |
| `temporary_space_bytes` | Temporary-data budget, at most 512 MiB |

Example adapter request shape observed in current fixtures:

```json
{"schema_version":1,"id":"adapter-pass-fixture","workspace_root":"/absolute/workspace","input_paths":["platform/ios/example/src/lib.rs"]}
```

The validator constructs the actual paths and request; copy the exact requirements from the working fixture rather than hardcoding workspace values. The response is one JSON document:

```json
{"schema_version":1,"status":"PASS","detail":"required guard verified"}
```

Accepted status vocabulary in current adapters: `PASS`, `FAIL`, `SKIPPED`, `ERROR-BLOCKED` (case-sensitive). Do **not** infer that a PASS from textual inspection proves runtime ABI correctness. Use output detail to state the exact evidence.

Minimal illustrative adapter:

```python
import json
from pathlib import Path
import sys


def main():
    req = json.load(sys.stdin)
    if req.get("schema_version") != 1 or len(req.get("input_paths", [])) != 1:
        print(json.dumps({"schema_version": 1, "status": "ERROR-BLOCKED", "detail": "bad input"}))
        return
    root = Path(req["workspace_root"]).resolve(strict=True)
    source = (root / req["input_paths"][0]).resolve(strict=True)
    if root not in source.parents:
        print(json.dumps({"schema_version": 1, "status": "ERROR-BLOCKED", "detail": "path outside workspace"}))
        return
    ok = "required_api_marker" in source.read_text(encoding="utf-8")
    print(json.dumps({"schema_version": 1, "status": "PASS" if ok else "FAIL",
                      "detail": "source guard present" if ok else "required guard missing"}))


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(json.dumps({"schema_version": 1, "status": "ERROR-BLOCKED", "detail": str(exc)[:256]}))
```

Use the example solely for a source assertion; choose native tests/oracles for ABI behavior. Code should be standard-library-only when feasible, bounded, deterministic, not access external services, and not inspect secrets. The validator sanitizes the child environment except selected values, imposes time/output/space limits and terminates on timeout/cancel; Python adapters are still **trusted local code**, not sandboxes.

To exercise the fixture rather than adding a real API, use `tools/validation/fixtures/example-validation-v1.json` with `--list` / `--explain` / its registered ID. It includes a passing and intentionally invalid adapter to ensure protocol failure does not produce false PASS.

## 7. Failure classifications and troubleshooting

| Symptom | First diagnosis | Correct action |
|---|---|---|
| Binary not found on PATH | Installer/prefix/PATH mismatch | Install pinned tools; verify `command -v`, `--version` |
| Unsupported host triple | Check release manifest | Mark blocked; separate maintenance for new host support |
| Archive hash/metadata mismatch | Artifact corruption or wrong pin | Do not run binary; report and restore verified artifact |
| Schema/version mismatch | Compare tool `--version` and schema constants | Use matching pinned release/spec; otherwise maintenance escalation |
| Missing SDK, target, `clang`, `nm`, `otool`, `vtool` | Host/toolchain limitation | Record SKIPPED/blocked; run on suitable macOS host, don't claim PASS |
| Clippy fails on Linux for Apple-only `objc2` | Wrong host/target for that check; also known CI category | Scope a safe iOS target test or file separate regression report; don't disable checks |
| iOS Files import-order audit fails | Existing macOS CI failure category | Compare sorted canonical imports and record exact diff; separate fix if needed |
| Link/framework import missing | Wrong native bridge/target/framework or API wiring | Inspect capability source/spec, linker artifact and compiler oracle; do not touch engine source |
| Source guard fails | Product contract or expected string changed | Verify intended API, update explicit guard only if semantically correct |
| Required adapter unavailable | Python or adapter path/config missing | Install supported Python or correct spec, rerun; no passing skip |
| Adapter ERROR-BLOCKED/invalid JSON | Script crash, malformed request/response, path or budget error | Minimal reproduce using documented stdin JSON; fix adapter if it is API-specific |
| `--changed` selects too few pilots | `path_rules` or dependency closure not declared | Correct declarative graph, conservatively run `--all` until verified |
| `--changed` base unavailable in shallow clone | Missing local Git ancestor | Choose existing explicit base or fetch only authorized branch history; do not fetch shared engine source to diagnose tooling |
| Executable returns contradictory or undocumented status | Suspected engine bug | Write BUG_REPORT and stop affected work |

**Known CI baseline at inspected commit**: Shared-tool artifact workflow completed successfully; ordinary Ubuntu CI failed at Clippy and macOS CI at the iOS app-data import/deployment audit. Do not automatically classify all later failures as pre-existing. Compare exact failing commands, diagnostics, affected symbols and commit histories.

## 8. Evidence taxonomy and truthful completion

| Evidence | Establishes | Does NOT establish |
|---|---|---|
| Rust host compile/Clippy | Host compilation, lint | Apple link or iOS runtime |
| iOS cross-target compile | Target typing/compilation | Final link/import or simulator/device behavior |
| Apple binary `nm`/`otool`/`vtool` | Symbol imports, linked frameworks, minimum OS metadata | Dynamic API semantics, ownership safety in use |
| Swift/Clang compiler oracle | ABI lowering consistency for tested toolchain/targets | Future toolchain support or complete runtime safety |
| Source guards | Presence/absence of exact source patterns | Semantic security/ABI correctness |
| Python adapter | Declared deterministic assertion | More than its explicit `evidence_type` |
| Simulator runtime test | Behavior in that tested simulator/environment | Physical device behavior |
| Physical device test | Behavior on that device/OS | Untested OS/hardware configurations |

When an SDK/target/device is unavailable, mark the relevant check unverified. A skipped check is not a passing check. Keep regression fixtures for the original failure mode. Respect zero-shipping-Swift, public API-only and all existing framework constraints.

## 9. Tooling defects: bug report and separate maintenance

Ordinary API Codex should first verify installed pinned tool version, correct flags, spec schema, host, SDKs and a minimal independent reproduction. If API-specific Python/JSON is defective, fix it within the API's assigned scope; don't misclassify it as an engine defect. If the documented engine behavior remains inconsistent, commit a **sanitized** report **without fixing the engine**.

Suggested filename: `BUG_REPORT_VALIDATOR_REQUIRED_ADAPTER_SKIPPED.md` (use a descriptive uppercase slug, one issue per report). Template:

```markdown
# Tooling bug: <short descriptive issue>

- State: suspected tooling defect / confirmed tooling defect
- Affected tool: ios-rust-build / ios-rust-validate
- Tool version, release ID, full source SHA: <from --version/manifest>
- Host triple, OS, SDK, Rust toolchain: <details>
- Affected capability and task: <ID>

## Minimal reproduction
<exact safe command, minimal input/spec, stable paths and prerequisites>

## Expected behavior
<documented contract and reference section>

## Actual behavior
<exit code, structured diagnostic, sanitized stdout/stderr>

## Troubleshooting already performed
<version checks, config validation, prerequisites, adapter test>

## Impact
<which required checks/API work are blocked; independent work still possible>

## Evidence and attachments
<small fixtures/log excerpts; no credentials or personal data>

## Recommended maintenance investigation
<suspected component based on black-box evidence; no guessed fix>
```

Commit that report as a separate scoped commit if the repository task permits commits, and tell the user its path and SHA. Never commit credentials, private keys, account data, sensitive paths or full environment dumps. Do not bypass a failing assertion, silently change required-to-optional, or fabricate evidence. A separate user-authorized maintenance session, coordinated with the assistant, retrieves the exact immutable engine source revision, fixes the issue, runs engine tests and all downstream compatibility gates, then releases a new signed/verified-or-checksummed pin.

**Maintenance source record:** Look up the exact single source SHA for the current binary release in `tools/releases/manifest-v1.tsv`. At the inspected source-isolation commit, maintenance source is historical Git source, *not* present in ordinary `main` paths. Tool releases may be reproduced through `.github/workflows/shared-tools.yml`, whose maintenance checkout points to that immutable SHA. A 90-day uploaded CI artifact retention does not replace the checked-in immutable archives. A new release requires reviewing updates to manifest, checked-in archives, installer expectations, target coverage and provenance together.

## 10. CI, non-tooling commands and developer handoff

- `.github/workflows/ci.yml` provisions pinned PATH tools in each job before normal framework checks, and invokes selected capability validation.
- `.github/workflows/shared-tools.yml` is **maintenance tooling** and builds from immutable historical source; ordinary API sessions should not edit it.
- Remaining `cargo xtask ...` commands are **not** the shared validator. They include `toolchain-manifest`, `ios-build`, `archive-smoke`, `no-std-check`, `no-std-link-probe`, `sdk-inventory`, `dependency-audit`, `abi-audit`, `codegen-audit`, `linkage-audit`, `zero-swift-source`, and `docs-check`—inspect `cargo xtask --help` for authoritative current flags.
- The preexisting `docs/VALIDATION.md` contains extensive framework-wide validation examples. It is not necessary to read that entire file to use the two PATH-installed tools; start here and follow targeted references.

**Task handoff:** Report the exact commit SHA, product/spec/adapter files changed, tool release, commands executed, PASS/FAIL/SKIPPED/ERROR-BLOCKED gates, evidence category, original bug regression tested, unresolved assumptions and any separate BUG_REPORT commit. Do not read or restore historical engine source in ordinary API work. Do not start unapproved API, tooling-maintenance or unrelated workstreams.

## 11. Quick navigation

| Need | Read |
|---|---|
| Install pinned tools | `tools/releases/README.md`, `tools/install-tools.sh` |
| Build spec schema | `tools/native-build/specs/schema-v1.json`, relevant `build-spec.json` |
| Minimal Cargo invocation | Relevant `platform/ios/<pilot>/build.rs` |
| Validation schema | `tools/validation/specs/schema-v1.json`, `validation-v1.json` |
| Adapter example | `tools/validation/adapters/check_homekit_contract.py` |
| Adapter edge-case tests | `tools/validation/fixtures/{passing_adapter.py,invalid_adapter.py,example-validation-v1.json}` |
| Pilot compiler oracles/imports | Relevant capability `scripts/check-*.sh`, `check-link-imports.sh` |
| Historical maintenance engine | Immutable SHA from `tools/releases/manifest-v1.tsv` **maintenance session only** |
| Framework-wide regression references | `docs/VALIDATION.md` (search targeted section) |
| Architecture and ownership | `AGENTS.md` |
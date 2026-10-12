# Tooling bug: native build rejects system-only API bridge

- State: suspected tooling defect
- Affected tool: ios-rust-build
- Tool version, release ID, full source SHA: `0.1.0`, `ios-rust-tools-0.1.0+2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`, `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`
- Pinned executable SHA-256: `7962ad29cb3a201fcf80bdf4befedacc616b77a493652a9df408af8ff70533a2`
- Host triple, OS, SDK, Rust toolchain: `x86_64-apple-darwin`; macOS `26.6.2 (25G83)`; Xcode `26.6 (17F113)`; iPhoneOS SDK `26.5`; `rustc 1.94.1 (e408947bf 2026-03-25)`
- Affected capability and task: B575, small iOS Rust API over the public Unified Logging C API

## Minimal reproduction

Run from the repository worktree root after installing the repository-pinned tools. Set the executable path explicitly, then run the reproduction:

```sh
tools/install-tools.sh --prefix "$PWD/target/ios-rust-tools"
PINNED_IOS_RUST_BUILD="$PWD/target/ios-rust-tools/bin/ios-rust-build"
"$PINNED_IOS_RUST_BUILD" build \
  --workspace-root "$PWD" \
  --spec "$PWD/platform/ios/ios-system-logging/build-spec.json" \
  --target aarch64-apple-ios \
  --target-os ios \
  --out-dir "$PWD/target/b575-build-out" \
  --manifest-dir "$PWD/platform/ios/ios-system-logging" \
  --format json
```

Minimal source `platform/ios/ios-system-logging/native/log_bridge.c`:

```c
#include <os/log.h>
#include <stdint.h>

void ios_system_logging_write(void *log, uint8_t type, const char *message) {
    os_log_with_type((os_log_t)log, (os_log_type_t)type, "%{private}s", message);
}
```

Minimal spec `platform/ios/ios-system-logging/build-spec.json`:

```json
{
  "schema_version": 1,
  "capability": "ios-system-logging",
  "library": "ios_system_logging",
  "sources": ["native/log_bridge.c"],
  "headers": [],
  "frameworks": [],
  "minimum_os": "10.0",
  "targets": [{"triple": "aarch64-apple-ios", "sdk": "iphoneos"}],
  "feature_guards": [],
  "compiler_options": ["-std=c11", "-Wall", "-Wextra", "-Werror"],
  "compile_description": "iOS Unified Logging public macro compile",
  "archive_description": "iOS Unified Logging public macro static archive"
}
```

The source calls only Apple's public `os_log_with_type` macro. It does not name `_os_log_impl` or reproduce its compiler-generated encoding. The iOS 10 deployment floor follows the public Unified Logging API availability; no Apple framework is used because these C APIs are supplied by `libSystem`.

## Expected behavior

The build tool should be able to compile and archive a valid native C bridge whose only Apple dependency is a system library and whose required Apple framework dependency list is empty. In [docs/SHARED_TOOLING.md](docs/SHARED_TOOLING.md), the build specification contract describes `frameworks` as only the public Apple frameworks a capability actually needs. This capability needs none.

The source-of-truth schema at `tools/native-build/specs/schema-v1.json` sets `frameworks.minItems` to `1`; the expected behavior above is therefore a suspected contract/schema gap, not a claim that this schema currently permits the reproduction.

## Actual behavior

The exact pinned-tool command exited `1` before native compile/archive work and returned this structured diagnostic:

```json
{"code":"build_failed","message":"at least one public framework dependency is required","schema_version":1,"status":"error"}
```

## Troubleshooting already performed

- Verified the executable path under the repository-pinned `target/ios-rust-tools/bin/ios-rust-build` directory, its `--version --format json`, `--help`, and SHA-256.
- Confirmed the reported `source_sha` and `release_id` match the repository pin `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`.
- Read the public build-spec schema and used its required fields, a supported `aarch64-apple-ios` target, the available iPhoneOS SDK, and explicit workspace/spec/output/manifest paths.
- Reproduced with a minimal C source and spec; the structured failure identifies the empty framework list as the blocker.
- Did not inspect or modify shared engine source, pinned archives, release manifests, or build tooling.
- No product source, package, validator profile, capability status, or workspace manifest was changed.

## Impact

Schema v1 and the pinned builder cannot represent a native iOS capability that compiles C for public system-library APIs but requires no Apple framework. B575 cannot use the required per-capability build-spec/build.rs integration without adding a false framework dependency. This report does not block unrelated capabilities that use actual frameworks.

## Evidence and attachments

The reproduction source and spec are included above. The command produced only the structured diagnostic shown above; no credentials, personal data, or runtime logs were involved.

## Recommended maintenance investigation

Review whether schema v1 should allow an empty `frameworks` array and whether the build result can omit framework link directives when that array is empty. Do not add an unrelated framework merely to satisfy the current minimum.

# PLAN_REUSE_BUILD.md — R1: native bridge build-system reuse

## Implementation scope

Inspect first: `platform/ios/ios-activitykit-status/build.rs::main`, `platform/ios/ios-alarmkit-status/build.rs::main`, `platform/ios/ios-photogrammetry-status/build.rs::main`, their `native/*.c`, `Cargo.toml`, and `check.sh`. Inspect workspace membership. Proposed `tools/native-build-support/**` build-dependency crate or a small shared build module.

## Verified facts

All three scripts duplicate command invocation, Xcode SDK discovery through `xcrun`, Clang target compilation, `ar crs` and Cargo linker directives. They differ in frameworks, supported architectures, deployment floors and sources. Swift ABI signatures are compiler-derived and must remain capability-specific.

## Required behavior

- Define a build-time-only typed configuration: sources, library, public framework dependencies, targets/SDKs, minimum OS versions, supported architectures and explicit compiler options.
- Use one invocation/error-reporting implementation and deterministic rerun/archive/link output; reject unsupported targets.
- Keep capability crates separate; each `build.rs` becomes a thin declarative adapter. Avoid forcing unrelated frameworks into one library.
- Migrate only the three pilot crates first; broader migration requires demonstrated equivalence. Preserve generated exports, weak linking and exact target-specific behavior.
- No runtime dependency, Swift source, new Apple capability, guessed mangled symbol or change to the public Rust/C ABI.

## Tests and validation

Use injected fake command runners and SDK paths to test target mapping, no SDK, Clang/ar failure, invalid names and path spaces. Compare baseline versus new actual compile/link commands and static artifacts where feasible. Run `cargo +1.94.1 fmt --all -- --check`, locked checks for pilot crates, pilot `check.sh` on appropriate macOS host, device/simulator imports and swiftcall checks, `cargo +1.94.1 xtask docs-check` and `cargo +1.94.1 xtask zero-swift-source`. Never equate host check with device link.

## Ownership and handoff

R1 owns helper and the three pilot build scripts. R2 owns CI/xtask. R3 consumes the helper. Report exact changed symbols/files, command comparison, evidence, tests, unsupported cases and commit SHA.

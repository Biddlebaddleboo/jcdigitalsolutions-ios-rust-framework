# Validation and Tooling

Run shared checks from the repository root with the `cargo xtask` alias. Reports go under `target/xtask/` by default and are build artifacts, not machine-specific source files.

```bash
cargo xtask toolchain-manifest --output target/xtask/toolchain-manifest.json
cargo xtask ios-build --simulator --release
cargo xtask ios-build --device --release
cargo xtask no-std-check
cargo xtask sdk-inventory --sdk all --output target/xtask/sdk-inventory-all.json
cargo xtask dependency-audit
cargo xtask abi-audit --output target/xtask/abi-audit.json
cargo xtask linkage-audit --binary path/to/consumer
cargo xtask zero-swift-source
cargo xtask docs-check
```

## What the checks prove

- `no-std-check` checks each portable crate with its default features and with `--no-default-features`, and verifies `#![no_std]` in each crate root. The package list is maintained in `tools/xtask/src/main.rs`; add each new portable crate there.
- `ios-build` requires exactly one of `--simulator` or `--device` plus `--release`. It maps the flag to the `simulator` or `device` positional argument accepted by `examples/ios-minimal/build.sh`, which builds and bundles that minimal example in Release mode. This is an example build path, not proof of full-framework V1 support.
- `dependency-audit` saves Cargo feature, duplicate-version, and build-dependency graphs, with direct/transitive package-node counts and a count of enabled features named `std`. It is an inventory only: there is no dependency-growth baseline, Cargo metadata does not prove that every transitive dependency avoids `std`, and proc-macro target kinds are not classified.
- `sdk-inventory` records public framework header, module-map, and `.swiftinterface` paths plus declaration counts and heuristic flags for availability, Objective-C exposure, async/throws, generics, actor isolation, and protocol conformance. It is useful for drift triage, not a complete parser or public API/compliance review.
- `abi-audit` records Rust source declarations and the ABI version constants. It does not verify compiled C layouts, the final symbol table, the linked calling convention, Swift runtime provenance, or panic containment in a linked consumer.
- `linkage-audit` uses `otool -L` on a supplied Mach-O binary. The minimal example results below are scoped to those two artifacts; no general expected-import policy or unrelated-capability absence gate is claimed.
- `zero-swift-source` rejects every committed `.swift` file, including under `tools/swift-oracle/`; Swift oracle input must be generated transiently outside the checkout.

Parity, physical-device performance, optimized codegen, and Xcode archive checks remain unavailable until reference adapters, benchmark cases, and an archive scheme exist. `parity` has no Apple reference adapter or Rust candidate; `archive-smoke` has no archive script or scheme. These commands fail with an explicit reason and emit no placeholder evidence.

## Current host baseline

The inspected host reports macOS 26.6.2 (build 25G83), Xcode 26.6 (build 17F113), iPhoneOS and iPhoneSimulator SDK 26.5, Swift 6.3.3, Apple Clang 21.0.0 (LLVM build `clang-2100.1.1.101`), and Rust 1.94.1. The plan requires Xcode 27.x, so this host does **not** satisfy its Xcode baseline. `toolchain-manifest` records the mismatch and emits a warning; it does not fail the host Rust checks or relabel Xcode 26.6 as 27.x.

Rust 1.94.1 has these targets installed on this host: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, and `x86_64-apple-darwin`. The repository has no full framework Xcode project or iOS framework crate, and no shared deployment target is defined; the minimal example script sets `IPHONEOS_DEPLOYMENT_TARGET=17.0` for its own build.

## iOS minimal example results (2026-10-07)

On the recorded Xcode 26.6 host, both `cargo xtask ios-build --simulator --release` and `cargo xtask ios-build --device --release` passed. They produced unsigned bundles at `target/ios-minimal/simulator/ios-minimal.app` and `target/ios-minimal/device/ios-minimal.app`. Both bundle executables imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`; neither imported the Swift or Python runtime. `plutil -lint` passed for `target/ios-minimal/simulator/ios-minimal.app/Info.plist` and `target/ios-minimal/device/ios-minimal.app/Info.plist`.

These results do not meet the plan's Xcode 27.x baseline. No simulator launch, signing, archive, or physical-device validation was performed. They show successful unsigned example cross-builds on this Xcode 26.6 host only.

## CI checks

The shared workflow runs formatting, Clippy, workspace tests, portable `no_std` checks, dependency and ABI inventories, rustdoc, the docs index check, and the zero-Swift-source gate on macOS and Linux. macOS installs the iOS simulator Rust target, runs the minimal example's simulator Release build, audits that binary's imports, lints its `Info.plist`, and records the Xcode/SDK environment. The Xcode 27.x mismatch is a warning until a runner with the planned toolchain is available; simulator launch, device execution, parity, benchmark, signing, and archive gates are not marked passed.

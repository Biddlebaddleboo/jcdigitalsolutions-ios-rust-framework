# Validation and Tooling

Run shared checks from the repository root with the `cargo xtask` alias. Reports go under `target/xtask/` by default and are build artifacts, not machine-specific source files.

```bash
cargo xtask toolchain-manifest --output target/xtask/toolchain-manifest.json
cargo xtask ios-build --simulator --release
cargo xtask ios-build --device --release
cargo xtask archive-smoke
cargo xtask no-std-check
cargo xtask sdk-inventory --sdk all --output target/xtask/sdk-inventory-all.json
cargo xtask dependency-audit
cargo xtask abi-audit --output target/xtask/abi-audit.json
cargo xtask linkage-audit --binary path/to/consumer
cargo xtask zero-swift-source
cargo xtask docs-check
cargo test -p parity-harness
cargo test -p bench-harness
```

## What the checks prove

- `no-std-check` checks each portable crate with its default features and with `--no-default-features`, and verifies `#![no_std]` in each crate root. The package list is maintained in `tools/xtask/src/main.rs`; add each new portable crate there.
- `ios-build` requires exactly one of `--simulator` or `--device` plus `--release`. It maps the flag to the `simulator` or `device` positional argument accepted by `examples/ios-minimal/build.sh`, which builds and bundles that minimal example in Release mode. This is an example build path, not proof of full-framework V1 support.
- `dependency-audit` saves Cargo feature, duplicate-version, and build-dependency graphs, with direct/transitive package-node counts and a count of enabled features named `std`. It is an inventory only: there is no dependency-growth baseline, Cargo metadata does not prove that every transitive dependency avoids `std`, and proc-macro target kinds are not classified.
- `sdk-inventory` records public framework header, module-map, and `.swiftinterface` paths plus declaration counts and heuristic flags for availability, Objective-C exposure, async/throws, generics, actor isolation, and protocol conformance. It is useful for drift triage, not a complete parser or public API/compliance review.
- `abi-audit` records Rust source declarations and the ABI version constants. It does not verify compiled C layouts, the final symbol table, the linked calling convention, Swift runtime provenance, or panic containment in a linked consumer.
- `linkage-audit` uses `otool -L` on a supplied Mach-O binary. The minimal example results below are scoped to those two artifacts; no general expected-import policy or unrelated-capability absence gate is claimed.
- `sh bindings/c/check.sh` compiles public headers as C11 and C++17, checks the recorded C layouts and static-library symbol set, then links and runs a C consumer. CI runs this check on macOS.
- `sh bindings/c/check-secure-storage.sh` validates the opt-in Keychain C ABI: feature isolation, C11/C++17 headers, symbols, host `UNSUPPORTED` stubs, and device/simulator static-library imports. It does not exercise a live Keychain; CI runs it on macOS.
- `zero-swift-source` rejects every committed `.swift` file, including under `tools/swift-oracle/`; Swift oracle input must be generated transiently outside the checkout.

The parity and benchmark packages provide harness unit fixtures only; they do not register real Apple/reference suites or framework workloads and produce no parity/performance evidence. `cargo xtask parity` remains unavailable until an Apple reference adapter and Rust candidate suite are registered. Physical-device performance and optimized codegen checks also remain unavailable. `archive-smoke` runs the shared `ios-minimal` Xcode scheme with the standard `xcodebuild archive` action, disables code signing, and verifies the resulting app bundle, plist, imports, and absence of `.swift` files in the archive. It is an unsigned packaging check, not a signing, provisioning, export, installation, or runtime check.

## Current host baseline

The inspected host reports macOS 26.6.2 (build 25G83), Xcode 26.6 (build 17F113), iPhoneOS and iPhoneSimulator SDK 26.5, Swift 6.3.3, Apple Clang 21.0.0 (LLVM build `clang-2100.1.1.101`), and Rust 1.94.1. The plan requires Xcode 27.x, so this host does **not** satisfy its Xcode baseline. `toolchain-manifest` records the mismatch and emits a warning; it does not fail the host Rust checks or relabel Xcode 26.6 as 27.x.

Rust 1.94.1 has these targets installed on this host: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, and `x86_64-apple-darwin`. The repository has no full framework Xcode project or iOS framework crate, and no shared deployment target is defined; the minimal example script sets `IPHONEOS_DEPLOYMENT_TARGET=17.0` for its own build.

## iOS minimal example results (2026-10-07)

On the recorded Xcode 26.6 host, `cargo xtask archive-smoke` passed on 2026-10-07. It built the Release device executable with Cargo, then ran the standard Xcode archive action using the `ios-minimal` shared scheme, `-destination 'generic/platform=iOS'`, and `CODE_SIGNING_ALLOWED=NO`. The archive is at `target/ios-minimal/archive/ios-minimal.xcarchive`; its app is `target/ios-minimal/archive/ios-minimal.xcarchive/Products/Applications/ios-minimal.app`. `plutil -lint` passed for the archive and app `Info.plist` files; `codesign -dv` reported `code object is not signed at all`. The archived executable imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`, with no Swift or Python runtime imports. The archive contains no `.swift` source.

On the same host, both `cargo xtask ios-build --simulator --release` and `cargo xtask ios-build --device --release` passed. They produced unsigned bundles at `target/ios-minimal/simulator/ios-minimal.app` and `target/ios-minimal/device/ios-minimal.app`. Both bundle executables imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`; neither imported the Swift or Python runtime. `plutil -lint` passed for `target/ios-minimal/simulator/ios-minimal.app/Info.plist` and `target/ios-minimal/device/ios-minimal.app/Info.plist`.

The archive emitted Xcode warnings that all interface orientations must be supported unless full-screen is required, and that a launch configuration/storyboard must be provided unless full-screen is required. These results do not meet the plan's Xcode 27.x baseline. No signing/provisioning, archive export, simulator launch, installation, or physical-device validation was performed. The archive result is unsigned packaging evidence from this Xcode 26.6 host only.

## CI checks

The shared workflow runs formatting, Clippy, workspace tests (including harness fixtures), portable `no_std` checks, dependency and ABI inventories, rustdoc, the docs index check, and the zero-Swift-source gate on macOS and Linux. macOS also runs `sh bindings/c/check.sh` and `sh bindings/c/check-secure-storage.sh`, installs both iOS device and simulator Rust targets, builds the minimal example in Release mode, audits that binary's imports, lints its `Info.plist`, and records the Xcode/SDK environment. The archive smoke is a documented manual macOS command rather than a CI gate. The Xcode 27.x mismatch is a warning until a runner with the planned toolchain is available; simulator launch, device execution, real parity suites, benchmark evidence, signing, archive export, and physical-device gates are not marked passed.

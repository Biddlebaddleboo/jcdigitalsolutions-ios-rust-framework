# PLAN_VALIDATION_IOS_BLUETOOTH_DISCOVERY.md — D29: Bluetooth discovery package gates

## Scope and prerequisites
D29 validates the existing discovery slice across `framework-bluetooth` and `ios-bluetooth`. Start with their package manifests, package-local target check script, `PLAN_VALIDATION_IOS_BLUETOOTH.md`, and `docs/ios/bluetooth.md`. The integrated root `Cargo.lock` must resolve the required `objc2-core-bluetooth` dependency. The historical instruction to add that dependency applies only if it is missing from the current lockfile; do not re-add it.

## Installed tooling contract
R1/R2 engines are completed pinned PATH executables. Follow `docs/SHARED_TOOLING.md`. D29 is **not** one of the four registered profiles in `tools/validation/specs/validation-v1.json`; do not claim `ios-rust-validate --capability ios-bluetooth` passes. Continue running the existing package-local script and commands until a schema-v1 profile has been added with equivalent positive and negative gate coverage. A future profile may describe standard compilation/lint/doc gates, source/ABI requirements and dependency edges. Use a small Python adapter only for a discovery-specific assertion not expressible in JSON. Do not implement or inspect engine source.

## Required checks
Execute from the workspace root on a suitable macOS host:
```sh
cargo fmt --all -- --check
cargo test -p framework-bluetooth
cargo check -p framework-bluetooth --no-default-features
cargo test -p ios-bluetooth
cargo check --locked -p ios-bluetooth --target aarch64-apple-ios
cargo clippy --locked -p ios-bluetooth --all-targets --target aarch64-apple-ios -- -D warnings
cargo check --locked -p ios-bluetooth --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-bluetooth --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p framework-bluetooth -p ios-bluetooth --no-deps
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
```
The package-local script may run the same target checks as a shortcut, **provided** its coverage is checked against the required gates; do not count a script invocation and its internal commands as independent proof. Preserve relevant feature isolation and native-import expectations if the package's focused scripts assert them.

## Evidence and non-claims
Portable queue/fixed-value tests use fake or copy-only values; host tests do not call CoreBluetooth. Device/simulator `cargo check` and Clippy demonstrate compilation, not creation of `CBCentralManager`, permission UI, radio scans, peer discovery, RSSI measurements, or callback/drop correctness. These gates do not perform signing, background-mode/restoration, physical-device or network tests.

## Handoff
Record exactly which gates executed, toolchain/SDK/target versions, commands, skips and failed evidence. For a future validator registration prove parity (including a deliberately failing guard) before retiring direct scripts. Suspected engine defects require a sanitized `BUG_REPORT_*.md` and separate tooling maintenance.
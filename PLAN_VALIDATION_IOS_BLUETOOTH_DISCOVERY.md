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

## D29 execution record — 2026-10-09

Host: Xcode 26.6 (17F113), iPhoneOS and iPhoneSimulator SDK 26.5, `rustc 1.94.1 (e408947bf 2026-03-25)`. Rust targets `aarch64-apple-ios` and `aarch64-apple-ios-sim` were installed. The pinned `ios-rust-build` and `ios-rust-validate` 0.1.0 host tools were installed from this checkout; their source SHA is `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`. `ios-rust-validate --list` did not include `ios-bluetooth`, so no D29 validator-profile result is claimed.

PASS:
- `cargo +1.94.1 fmt --all -- --check`
- `cargo +1.94.1 test --locked -p framework-bluetooth` — 4 passed
- `cargo +1.94.1 check --locked -p framework-bluetooth --no-default-features`
- `cargo +1.94.1 test --locked -p ios-bluetooth` — 5 passed
- `cargo +1.94.1 check --locked -p ios-bluetooth --target aarch64-apple-ios`
- `cargo +1.94.1 clippy --locked -p ios-bluetooth --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 check --locked -p ios-bluetooth --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-bluetooth --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked -p framework-bluetooth -p ios-bluetooth --no-deps`
- `cargo +1.94.1 xtask docs-check`
- `cargo +1.94.1 xtask zero-swift-source`
- `cargo +1.94.1 tree --locked -p ios-bluetooth --target aarch64-apple-ios -e features` — reviewed the package's explicit objc2 feature set and generated API feature closure; defaults remain disabled for the Objective-C framework crates.

The package-local `check-targets.sh` contains the same four device/simulator check and strict Clippy commands; they were run individually above, so the wrapper was not run as duplicate evidence. Native-import audit: SKIPPED because no separate D29 package-local final-link/native-import assertion or linked app target exists. Cargo target checks and Clippy compile the bindings but do not prove the linked app's final import set.

`git diff --check` is run after the D29 documentation updates and before commit. No prompt, scan, app, device, or simulator runtime test was run. No runtime, permission, radio, peer-discovery, RSSI, callback-teardown, background, or restoration behavior is claimed. No required D29 test/build/doc gate failed or was skipped.

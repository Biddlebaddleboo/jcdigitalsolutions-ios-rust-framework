# Bluetooth Discovery Package Gates — D29

## Required commands

Run from the repository root after the orchestrator adds the new `objc2-core-bluetooth` dependency to `Cargo.lock`:

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

Run the package-local target script as a shortcut for the device/simulator checks and strict Clippy.

## Evidence limits

- The portable contract and fixed queue tests use fake/copy-only values; no CoreBluetooth call occurs on the host.
- Device/simulator `cargo check` and Clippy compile the native bindings. They do not instantiate `CBCentralManager`, request permission, show UI, scan hardware, discover a peer, prove RSSI, or exercise drop/callback behavior.
- No network, signing, background-mode, restoration, or physical-device test is part of this slice.

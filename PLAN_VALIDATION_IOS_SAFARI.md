# PLAN_VALIDATION_IOS_SAFARI.md — Workstream G58: SafariServices Compile and Link Gates

## Objective

Gate the B64 typed SafariServices controller wrapper without presenting UI or starting a request

## Required local gates

- `cargo +1.94.1 check --locked -p ios-safari --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-safari --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-safari --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-safari --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `sh platform/ios/ios-safari/check-link-imports.sh`
- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-safari/Cargo.toml -- --check`
- `cargo +1.94.1 doc --locked -p ios-safari --no-deps`
- `cargo +1.94.1 --locked xtask docs-check`
- `cargo +1.94.1 --locked xtask zero-swift-source`
- `git diff --check`

The import probe links a caller-owned controller accessor but does not create the controller or
execute the binary. Rust 1.94.1's arm64 device target supports minimum iOS 10.0, so the link script
sets device minimum iOS 10.0 and arm64 Simulator minimum iOS 14.0, then verifies each with
`vtool -show-build`. It checks the direct framework allowlist and the class/selector strings plus
Objective-C lookup/message imports. The SDK declares the SafariServices initializer from iOS 9.0,
but this toolchain's target floor is the stricter device build floor

## Non-claims

No tests, app run, view presentation, URL request, page load, Safari routing, privacy prompt,
network response, or browser parity check is included

## Integrated result

B64/G58 is integrated in the root checkout. The focused device/Simulator checks, strict Clippy,
Release link/import audit, formatting, rustdoc, docs-check, zero-Swift-source, and diff gates passed
there. Workspace check and strict all-target/all-feature Clippy also passed after the probe example
was guarded for iOS and given a host no-op `main`. The probes were inspected, not executed; no live
Safari behavior is claimed

# PLAN_VALIDATION_IOS_CONNECTION.md — Workstream G18: Secure TCP Stream Gates

## Status

Validation is complete. These gates cover the D19 portable contract and B24 iOS compile/link
boundary only; they do not perform or prove live network behavior

## Objective

Record deterministic portable contract tests, iOS device/Simulator compile and lint, and focused
Release link/import checks for `framework-connection` and `ios-connection`

## Dependencies

- D19 portable facade: `PLAN_CAPABILITIES_CONNECTION.md`
- B24 iOS backend: `PLAN_IOS_CONNECTION.md`
- Existing workspace Rust toolchain and Xcode SDK

## Required checks

- `cargo +1.94.1 test --locked -p framework-connection`
- `cargo +1.94.1 check --locked -p framework-connection --no-default-features`
- `cargo +1.94.1 check --locked -p ios-connection --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked -p ios-connection --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-connection --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked -p ios-connection --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `sh platform/ios/ios-connection/check-link-imports.sh`
- `cargo +1.94.1 fmt --manifest-path crates/framework-connection/Cargo.toml -- --check`
- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-connection/Cargo.toml -- --check`
- `cargo +1.94.1 fmt --manifest-path tools/xtask/Cargo.toml -- --check`
- `cargo +1.94.1 doc --locked -p framework-connection -p ios-connection --no-deps`
- `cargo +1.94.1 --locked xtask docs-check`
- `cargo +1.94.1 --locked xtask zero-swift-source`
- `cargo +1.94.1 xtask no-std-check`
- `cargo +1.94.1 xtask no-std-link-probe`
- `git diff --check`

The link/import script must check device minimum iOS 12.0 and simulator minimum iOS 14.0. The
no_std link gate executes the portable host probe but does not execute its Apple-target artifacts

## Runtime boundary

No live network call, local-network permission prompt, TLS handshake, certificate trust result,
server receive, remote acknowledgement, cancellation timing, physical-device behavior, or
Simulator networking behavior is verified. There is no external endpoint, sleep, or timing test.
Portable fake-backend tests and the host-only no_std API probe establish only portable contract
behavior

## Handoff

Record exact command results, Xcode/SDK and Rust versions, direct imports, deployment targets,
changed paths, and failures. Do not mark API parity, security behavior, performance, or runtime
delivery as verified by compilation or link evidence

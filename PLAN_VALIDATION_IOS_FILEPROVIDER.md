# PLAN_VALIDATION_IOS_FILEPROVIDER.md — G78: File Provider snapshot gates

## Scope

Validate only `platform/ios/ios-file-provider` and its registered-domain presence query. The shared workspace and lock now include `ios-file-provider` and `objc2-file-provider` 0.3.2. Do not run tests or execute link probes

## Required gate

Run from the repository root after lock integration:

```sh
sh platform/ios/ios-file-provider/check.sh
```

The script is expected to run these non-test checks:

```sh
cargo fmt --manifest-path platform/ios/ios-file-provider/Cargo.toml -- --check
cargo check --locked -p ios-file-provider
cargo check --locked -p ios-file-provider --target aarch64-apple-ios
cargo check --locked -p ios-file-provider --target aarch64-apple-ios-sim
cargo clippy --locked --lib -p ios-file-provider --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --lib -p ios-file-provider --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked --no-deps -p ios-file-provider --target aarch64-apple-ios
sh platform/ios/ios-file-provider/check-link-imports.sh
cargo xtask docs-check
```

The link/import script builds Release probes for device iOS 11.0 and arm64 Simulator iOS 14.0. It inspects but never executes them. Expected frameworks are FileProvider and Foundation plus `libSystem.B.dylib` and `libobjc.A.dylib`; expected Objective-C symbols/selectors are the manager class and `getDomainsWithCompletionHandler:` only. It rejects Swift runtime imports and picker, file access, domain mutation, or provider-lifecycle APIs

The package gate also scans its Rust/Swift surface: no Swift source; no file picker, file coordinator, file read/write, security-scoped URL, domain add/remove, sync, or provider protocol implementation

## Results

Passed with `sh platform/ios/ios-file-provider/check.sh`. The script runs shell syntax, static feature/API/Swift-scope checks, `cargo fmt --manifest-path platform/ios/ios-file-provider/Cargo.toml -- --check`, host/device/Simulator `cargo check --locked`, strict device/Simulator `cargo clippy --locked --lib ... -- -D warnings`, iOS rustdoc, the link/import audit, and `cargo xtask docs-check`. The checks passed. The first run stopped at the link example because the returned future is `#[must_use]`; the example now binds that value, and the full rerun passed

The device link probe has minos iOS 11.0; the arm64 Simulator probe has minos iOS 14.0. Both link only FileProvider, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; the audit found the expected manager/query symbols and no Swift runtime imports. Probes were built and inspected only, never executed. No tests, provider requests, or runtime queries ran

## Runtime limits

These checks can establish compilation, feature selection, minimum link target, linked frameworks, and imports only. They cannot establish that the caller has a provider extension, domain registration, user enablement, service availability, network access, sync, document access, callback queue, or live iOS behavior

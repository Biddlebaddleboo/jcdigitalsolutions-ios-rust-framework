# PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md — Workstream G65: MediaPlayer Status Gates

## Required gates

- Format check for `platform/ios/ios-media-library-status`.
- Host, iOS device, and iOS Simulator library checks.
- Strict Clippy on device and Simulator targets.
- iOS-target rustdoc.
- Feature-tree inspection confirms `objc2-media-player` is absent off iOS and only `MPMediaLibrary` is selected on iOS; `default` and `block2` remain disabled.
- `sh -n platform/ios/ios-media-library-status/check.sh` and `git diff --check`.
- Do not run tests, link probes, authorization queries, prompts, item reads, service calls, or playback.

## Local evidence

The source-format check passed in the shared worktree. `sh -n platform/ios/ios-media-library-status/check.sh`, `git diff --check`, and per-file new-file whitespace checks also passed. Before root integration, the package was copied to `/tmp/ios-media-library-status-check.ymWzCe` under a minimal temporary workspace with the repository's package metadata and lint settings.

The following commands passed there on the installed Rust toolchain:

```sh
cargo check --offline --manifest-path /tmp/ios-media-library-status-check.ymWzCe/Cargo.toml --package ios-media-library-status --lib
cargo check --offline --locked --manifest-path /tmp/ios-media-library-status-check.ymWzCe/Cargo.toml --package ios-media-library-status --lib --target aarch64-apple-ios
cargo check --offline --locked --manifest-path /tmp/ios-media-library-status-check.ymWzCe/Cargo.toml --package ios-media-library-status --lib --target aarch64-apple-ios-sim
cargo clippy --offline --locked --manifest-path /tmp/ios-media-library-status-check.ymWzCe/Cargo.toml --package ios-media-library-status --lib --target aarch64-apple-ios -- -D warnings
cargo clippy --offline --locked --manifest-path /tmp/ios-media-library-status-check.ymWzCe/Cargo.toml --package ios-media-library-status --lib --target aarch64-apple-ios-sim -- -D warnings
cargo doc --offline --locked --no-deps --manifest-path /tmp/ios-media-library-status-check.ymWzCe/Cargo.toml --package ios-media-library-status --lib --target aarch64-apple-ios
```

The iOS feature tree selected only `objc2-media-player feature "MPMediaLibrary"` and required generated Foundation features; the host-target feature tree did not include MediaPlayer. After root lock integration, `sh platform/ios/ios-media-library-status/check.sh` passed on 2026-10-08 for locked host/device/Simulator checks, strict target Clippy, iOS rustdoc, format, shell syntax, and feature isolation. No tests or probe binaries were built or run.

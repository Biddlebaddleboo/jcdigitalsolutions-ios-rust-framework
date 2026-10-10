# G53 — Natural Language asset-status gate

## Scope

Gate D54/B59's iOS-only status query for Apple’s built-in English contextual-model assets. It does
not load a model, accept text, compute vectors, request assets, or start a download.

## Commands

- `sh platform/ios/ios-natural-language-status/scripts/check.sh`
- `sh platform/ios/ios-natural-language-status/scripts/check-link-imports.sh`

## Evidence boundary

The package check covers format, host/device/Simulator compilation and strict Clippy, rustdoc,
documentation and zero-Swift gates, API-surface bounds, and device/Simulator Release link-import
inspection. The link probes are built and inspected, never executed. They do not establish live asset
state, model-load success, vector output, or language coverage beyond the built-in English constant.
No tests are added or run by this gate.

## Status

The focused package gate passed in the isolated D54/B59 worktree and again in the integrated root
checkout. Local validation used Rust 1.94.1, Xcode 26.6 build 17F113,
and iPhoneOS/iPhoneSimulator SDK 26.5; that local run is below the planned Xcode 27.x baseline.

## Hosted Xcode 27 qualification

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check Natural Language English model asset status` ran `sh platform/ios/ios-natural-language-status/scripts/check.sh` and passed
- `Rust checks (macos-15)` also passed this package gate, but that runner reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped the Apple-only gate
- `Optional iOS Natural Language status C ABI check` and `Optional iOS Natural Language status C ABI link/import check` passed in the Xcode 27 job; logs state that C/C++ consumers and probes were not executed
- No diff from the run SHA to current main `cb807e43b2693f950293599f729d4445e8f827d6` in `platform/ios/ios-natural-language-status`, `bindings/c`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers static, compile, and link checks only; no model load, text processing, vector output, asset request, or download

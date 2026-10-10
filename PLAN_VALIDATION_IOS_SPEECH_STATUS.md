# G52 — Speech authorization-status gate

## Scope

Gate D53/B58's iOS-only read of `SFSpeechRecognizer::authorizationStatus` and the
`SpeechAuthorizationStatus` value. It does not request permission, create a recognizer, accept
audio, or start recognition.

## Commands

- `sh platform/ios/ios-speech-status/scripts/check.sh`
- `sh platform/ios/ios-speech-status/scripts/check-link-imports.sh`

## Evidence boundary

The package check covers format, host/device/Simulator compilation and strict Clippy, rustdoc,
documentation and zero-Swift gates, API-surface bounds, and device/Simulator Release link-import
inspection. The link probes are built and inspected, never executed. They do not establish live
authorization values, service availability, permission prompt behavior, microphone access, or
recognition output. No tests are added or run by this gate.

## Status

The focused package gate passed in the isolated D53/B58 worktree and again in the integrated root
checkout. Local validation used Rust 1.94.1, Xcode 26.6 build 17F113,
and iPhoneOS/iPhoneSimulator SDK 26.5; that local run is below the planned Xcode 27.x baseline.

## Hosted Xcode 27 qualification

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check Speech authorization status` ran `sh platform/ios/ios-speech-status/scripts/check.sh` and passed
- `Rust checks (macos-15)` also passed this package gate, but that runner reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped the Apple-only gate
- `Optional iOS Speech status C ABI check` and `Optional iOS Speech status C ABI link/import check` passed in the Xcode 27 job; logs state that C/C++ consumers and probes were not executed
- No diff from the run SHA to current main `cb807e43b2693f950293599f729d4445e8f827d6` in `platform/ios/ios-speech-status`, `bindings/c`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers static, compile, and link checks only; no permission prompt, recognizer, audio input, service availability, or recognition output

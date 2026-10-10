# PLAN_VALIDATION_IOS_MPS_STATUS.md — Workstream G62: MPS Preferred-Device Gate

## Status

After root integration, both the package gate and link/import gate passed on Rust 1.94.1, Xcode 26.6,
and iOS SDK 26.5. No tests or probe binaries were run.

## Commands

```sh
sh platform/ios/ios-mps-status/check.sh
sh platform/ios/ios-mps-status/check-link-imports.sh
```

The package gate checks format, host/device/Simulator builds, strict Clippy on both Apple targets,
rustdoc, MPSCore-only binding features, and docs. The link gate builds Release arm64 device and
Simulator examples, checks exact linked imports and selected symbols, and verifies deployment
metadata. The scripts inspect consumer probes; they do not execute them.

## Evidence

- Exact imports: Foundation, Metal, MetalPerformanceShaders, `libSystem.B.dylib`, and
  `libobjc.A.dylib`.
- Required symbol: `_MPSGetPreferredDevice`; no Swift runtime, `MTLCreateSystemDefaultDevice`, GPU
  commands, or unrelated MPS operation symbols.
- Embedded `minos`: iOS 12.2 device and iOS 14.0 Simulator.
- The API declaration floor is iOS 12.2; the Simulator deployment setting is only a link-check
  setting.
- No live device query, GPU work, operation/model support, runtime cost, or performance result is
  established.

## Hosted Xcode 27 evidence

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check iOS MPS preferred-device status package` ran `sh platform/ios/ios-mps-status/check.sh`, and `Audit iOS MPS preferred-device link imports` ran `sh platform/ios/ios-mps-status/check-link-imports.sh`; both passed
- Both gates also passed in `Rust checks (macos-15)`, which reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped these Apple-only gates
- No diff from the run SHA to current main `f7a6d0cd98d9a23ecd718fbc98751f46c048a204` in `platform/ios/ios-mps-status`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers package compile and link/import gates only; probes were not executed, and no live preferred-device query, GPU work, support, runtime cost, or performance is claimed

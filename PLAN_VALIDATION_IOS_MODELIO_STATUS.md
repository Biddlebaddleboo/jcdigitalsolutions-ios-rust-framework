# PLAN_VALIDATION_IOS_MODELIO_STATUS.md — Workstream G61: ModelIO Extension-Query Gate

## Status

After root workspace and lockfile integration, `sh platform/ios/ios-modelio-status/check.sh` passed
on Rust 1.94.1, Xcode 26.6, and iOS SDK 26.5. No tests or probe binaries were run.

## Gate

```sh
sh platform/ios/ios-modelio-status/check.sh
```

The script checks formatting, host/device/Simulator builds, strict Clippy on both Apple targets,
`MDLAsset` feature isolation, exact device/Simulator Release link imports and symbols, deployment
metadata, and iOS rustdoc. It builds but does not execute the consumer probes.

## Evidence and limits

- Exact imports: Foundation, ModelIO, `libSystem.B.dylib`, and `libobjc.A.dylib`.
- The consumer retains `_objc_msgSend` and `canImportFileExtension:`.
- Embedded `minos`: iOS 10.0 device and iOS 14.0 Simulator. The framework API floor is iOS 9.0.
- Compilation/linkage does not establish which extensions work on every OS/device, parsing of any
  asset, runtime file access, rendering, or GPU behavior.

## Hosted Xcode 27 evidence

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check iOS ModelIO extension-status package` ran `sh platform/ios/ios-modelio-status/check.sh` and passed
- The package step also passed in `Rust checks (macos-15)`, which reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped this Apple-only gate
- No diff from the run SHA to current main `f7a6d0cd98d9a23ecd718fbc98751f46c048a204` in `platform/ios/ios-modelio-status`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence adds compile/link gate results only; the consumer probes were not executed, and no extension availability, asset parse, file access, rendering, GPU behavior, or runtime result is claimed

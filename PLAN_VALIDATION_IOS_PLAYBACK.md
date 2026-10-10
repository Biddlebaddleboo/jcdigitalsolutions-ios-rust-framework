# PLAN_VALIDATION_IOS_PLAYBACK.md — Workstream G42: HDR Playback Query Gates

## Status

The package gate passes on Rust 1.94.1 with Xcode 26.6 build 17F113 / iOS SDK 26.5. It checks
device and Simulator target compilation and strict Clippy, host compilation and Clippy, formatting,
rustdoc, and Release import/minimum-OS audits. The arm64 probes import AVFoundation, Foundation,
`libSystem.B.dylib`, and `libobjc.A.dylib`; device minos is 13.4 and Simulator minos is 14.0. Probes
were linked but not executed. No tests, HDR device, or runtime playback check is part of this
evidence. CI wiring is present; hosted Xcode 27 static-gate evidence is recorded below.

## Objective

Keep persistent compile, lint, format, and documentation checks for D43 and B48 without claiming
HDR hardware or playback behavior.

## Required gates

- Run `sh platform/ios/ios-playback/check.sh`.
- Keep `aarch64-apple-ios` and `aarch64-apple-ios-sim` checks locked.
- Keep strict Clippy for both iOS targets and host check/Clippy.
- Keep the gate build-only; do not add or run tests in this resumed pass.
- Record Xcode/SDK version and state that target compilation does not establish live HDR behavior.

## Local result

`sh platform/ios/ios-playback/check.sh` passed in the integrated checkout and is wired in macOS CI.

## Hosted Xcode 27 static-gate evidence (2026-10-10)

The Xcode 27 lane in [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed step `Check iOS playback capability package`, which ran `sh platform/ios/ios-playback/check.sh` at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The runner recorded Xcode 27.0 build 27A266a, iPhoneOS and iPhoneSimulator SDK 27.0, and Rust 1.94.1. The script's format, host/device/Simulator checks and strict Clippy, rustdoc, and Release link/import/minimum-OS audits passed. The `crates/framework-audio/`, `platform/ios/ios-playback/`, workspace manifest/lockfile, and `.github/workflows/ci.yml` gate inputs are unchanged from the run SHA through current `origin/main` `5b59d9d`.

This is compile, lint, documentation, and link/import evidence only. The Release probes were linked and inspected, not executed. No HDR hardware, live playback behavior, or runtime parity/performance is established.

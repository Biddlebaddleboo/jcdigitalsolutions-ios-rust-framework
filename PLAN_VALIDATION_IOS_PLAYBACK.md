# PLAN_VALIDATION_IOS_PLAYBACK.md — Workstream G42: HDR Playback Query Gates

## Status

The package gate passes on Rust 1.94.1 with Xcode 26.6 build 17F113 / iOS SDK 26.5. It checks
device and Simulator target compilation and strict Clippy, host compilation and Clippy, formatting,
rustdoc, and Release import/minimum-OS audits. The arm64 probes import AVFoundation, Foundation,
`libSystem.B.dylib`, and `libobjc.A.dylib`; device minos is 13.4 and Simulator minos is 14.0. Probes
were linked but not executed. No tests, HDR device, or runtime playback check is part of this
evidence. CI wiring is present; no passing CI workflow run is recorded.

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

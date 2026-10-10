# PLAN_VALIDATION_IOS_SAFETYKIT.md — G47: SafetyKit Crash Detection Availability

## Objective

Gate D48/B53's single `SACrashDetectionManager.isAvailable` query without claiming event access,
authorization, or emergency response.

## Required checks

- Run `sh platform/ios/ios-safety/scripts/check.sh` on macOS.
- The script checks formatting, host package check/strict Clippy/rustdoc, device and Simulator check/
  strict Clippy, device rustdoc, and the exact SafetyKit/Foundation/libSystem/libobjc import set.
- Run docs freshness, zero-Swift-source, and whitespace checks.
- Build and inspect device/Simulator probes only; never execute them or simulate an event.
- Do not add or run tests, request permission, sign an app, or validate entitlement admission.

## Evidence boundary

Compile, lint, and import results do not establish the getter's live hardware result or whether the
getter alone requires the class-level entitlement. Event authorization/delivery and emergency
response remain unverified. The inspected Xcode 26.6 / SDK 26.5 host is below the planned Xcode
27.x baseline.

## Hosted Xcode 27 static-gate evidence (2026-10-10)

The Xcode 27 lane in [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed step `Check SafetyKit crash-detection availability gates`, which ran `sh platform/ios/ios-safety/scripts/check.sh` at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The runner recorded Xcode 27.0 build 27A266a, iPhoneOS and iPhoneSimulator SDK 27.0, and Rust 1.94.1. The script's source guards, format, host/device/Simulator check and strict Clippy, device rustdoc, exact link/import audit, docs-index, zero-Swift-source, and diff checks passed. The `platform/ios/ios-safety/`, workspace manifest/lockfile, `xtask/`, and `.github/workflows/ci.yml` gate inputs are unchanged from the run SHA through current `origin/main` `5b59d9d`.

This is static compile, lint, documentation, and link/import evidence only. The probe was built and inspected, not executed. It does not establish a live hardware result, class-level entitlement requirement, event authorization/delivery, or emergency response.

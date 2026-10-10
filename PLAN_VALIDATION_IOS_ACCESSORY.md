# PLAN_VALIDATION_IOS_ACCESSORY.md — Workstream G38: ExternalAccessory Gates

## Objective

Gate D39/B44's current connected-list presence snapshot without running accessory communication.

## Required gates

- Package formatting, portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator compile plus strict Clippy
- Source guard for the three allowed native calls and the absence of picker, event, session, protocol, stream, or transfer APIs
- Release device and Simulator link/import audit for `ExternalAccessory`, Foundation, libobjc, and libSystem; inspect Objective-C symbols and exclude Swift runtime symbols
- Repository docs, zero-Swift-source, and whitespace checks

The probe is linked, not run. No live list result, connected hardware, protocol eligibility, or communication is claimed.

## 2026-10-10 hosted CI recheck

[CI run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431)
completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; the Ubuntu
24.04, macOS 15, and xcode-27 jobs all passed. The ExternalAccessory gate
(`sh platform/ios/ios-accessory/scripts/check.sh`, step 175) passed on both Apple jobs; its
macOS-only step was skipped on Ubuntu. The xcode-27 job recorded Xcode 27.0 build `27A266a`,
iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0. The run SHA is an ancestor of current `main`
(`f7197e7b48bb35136f308792ae51617011e8e619`); the targeted plan, CI workflow, Cargo
manifests/lockfile, and ExternalAccessory capability paths are unchanged since the run.

This records hosted static compile/lint and link/import checks only. The probe was not run; no
connected accessory, hardware list result, protocol eligibility, or accessory communication is established.

# PLAN_VALIDATION_IOS_WATCH_CONNECTIVITY.md — Workstream G37: Watch Connectivity Gates

## Objective

Gate D38/B43's platform support Boolean without session lifecycle or watch communication.

## Required gates

- portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator check, strict Clippy, and iOS rustdoc
- source guard allowing only `WCSession::isSupported()` in the adapter
- Release import audit for WatchConnectivity, Foundation, libobjc, and libSystem
- docs, zero-Swift-source, and diff checks

The binary probes are not executed. No paired-device, activation, reachability, delivery, or runtime claim is made.

## 2026-10-10 hosted CI recheck

[CI run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431)
completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; the Ubuntu
24.04, macOS 15, and xcode-27 jobs all passed. The Watch Connectivity gate
(`sh platform/ios/ios-watch-connectivity/scripts/check.sh`, step 174) passed on both Apple jobs;
its macOS-only step was skipped on Ubuntu. The xcode-27 job recorded Xcode 27.0 build `27A266a`,
iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0. The run SHA is an ancestor of current `main`
(`f7197e7b48bb35136f308792ae51617011e8e619`); the targeted plan, CI workflow, Cargo
manifests/lockfile, and Watch Connectivity capability paths are unchanged since the run.

This records hosted static compile/lint and link/import checks only. The probes were not executed;
session activation, pairing, reachability, message delivery, and runtime behavior remain unverified.

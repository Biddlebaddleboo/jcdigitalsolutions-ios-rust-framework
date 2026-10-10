# PLAN_VALIDATION_IOS_GAMEKIT.md — G49: Game Center Local-Player Status

## Objective

Gate D50/B55's portable point-in-time authentication-status contract and the iOS
`GKLocalPlayer.localPlayer.isAuthenticated` read.

## Required checks

- Run `sh platform/ios/ios-game/check.sh` on macOS.
- Check `framework-game` without default features, strict Clippy, and rustdoc; check `ios-game` for
  device and Simulator targets with strict Clippy.
- Enforce that the adapter reads only the local player and its authentication Boolean; reject auth
  handlers, deprecated auth methods, listeners, player identity, and UI.
- Build, but do not execute, device and Simulator link probes; require GameKit/Foundation and reject
  Swift runtime or unrelated capability frameworks.
- Run no-std, formatter, documentation, zero-Swift-source, and diff checks. Do not add or run tests,
  launch a simulator, sign/install an app, or initiate authentication.

## Evidence boundary

Compile/link checks do not establish signed entitlement configuration, live player/account state,
authentication transitions, or UI behavior. The inspected Xcode 26.6 / SDK 26.5 host is below the
planned Xcode 27.x baseline.

## Hosted Xcode 27 static-gate evidence (2026-10-10)

The Xcode 27 lane in [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed step `Check Game Center status contract and iOS gates`, which ran `sh platform/ios/ios-game/check.sh` at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The runner recorded Xcode 27.0 build 27A266a, iPhoneOS and iPhoneSimulator SDK 27.0, and Rust 1.94.1. The script's source guard, format, portable no-default-features check/strict Clippy/rustdoc, device/Simulator check and strict Clippy, link/import checks, no-std, docs-index, zero-Swift-source, and diff checks passed. The `crates/framework-game/`, `platform/ios/ios-game/`, workspace manifest/lockfile, `xtask/`, and `.github/workflows/ci.yml` gate inputs are unchanged from the run SHA through current `origin/main` `5b59d9d`.

The link/import logs report that device and Simulator binaries were not executed. This is compile, lint, documentation, no-std, and link/import evidence only; it does not establish signed entitlement configuration, live player/account state, authentication transitions, or UI behavior.

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

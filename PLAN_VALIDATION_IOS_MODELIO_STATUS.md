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

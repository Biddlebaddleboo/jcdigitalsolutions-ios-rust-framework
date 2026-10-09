# PLAN_CAPABILITIES_MODELIO_STATUS.md — Workstream D61: ModelIO Import-Extension Status

## Status

D61 adds no portable `framework-ml` contract. This platform-exclusive slice exposes only the
ModelIO answer to whether an asset file extension is readable by `MDLAsset`.

## Contract boundary

- The iOS API is `ios_modelio_status::can_import_file_extension(&str) -> bool`.
- Forward the caller's string to `+[MDLAsset canImportFileExtension:]` without wrapper-side
  normalization or path handling.
- A `true` result reports ModelIO extension support only. It does not prove that a particular
  file is valid or readable, nor does it claim parsing, rendering, or GPU support.
- Do not create an `MDLAsset`, load a URL, access file data, or add an importer pipeline.
- This slice adds no portable crate, permission, entitlement, privacy prompt, UI, network, or service
  contract.

## API floor and evidence

- The inspected iPhoneOS 26.5 SDK declares `MDLAsset` available from iOS 9.0.
- `+[MDLAsset canImportFileExtension:]` has no later availability annotation in `MDLAsset.h`.
- The generated `objc2-model-io` 0.3.2 binding exposes `MDLAsset::canImportFileExtension` under
  feature `MDLAsset`; its signature takes `&NSString` and returns `bool`.
- Device and Simulator link probes use deployment `minos` 10.0 and 14.0, the Rust 1.94.1 target
  floors inspected for this workstream. Those probe floors are distinct from the API floor.

## Evidence boundary

The package build and link probes show that the typed binding compiles and links with the expected
framework set. The isolated package gate passed host check, device and Simulator target checks,
strict Clippy on both targets, target feature-tree checks, build-only device and Simulator link
probes, and iOS rustdoc. Exact probe imports were `Foundation`, `ModelIO`, `libSystem.B.dylib`, and
`libobjc.A.dylib`; embedded `minos` values were 10.0 (device) and 14.0 (Simulator). The probes were
not run. The integrated root locked package gate also passed. No
supported-extension parity, asset parse, file-access, rendering, GPU, or runtime result is claimed.

See `PLAN_IOS_MODELIO_STATUS.md` and `docs/ios/modelio-status.md`.

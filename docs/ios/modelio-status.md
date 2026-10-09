# ModelIO file-extension status

`ios-modelio-status` exposes one iOS-only query:

```rust
pub fn can_import_file_extension(extension: &str) -> bool
```

It passes the supplied string to Apple's `+[MDLAsset canImportFileExtension:]`. Apple defines the
Boolean as whether `MDLAsset` can read asset data from files with that extension. The wrapper does
not normalize the string or supply a URL or file data.

The query reports extension-level importer support only. It does not parse or validate a particular
file, guarantee that a file can be read, render an asset, or report GPU support. It adds no portable
`framework-ml` contract.

The inspected iPhoneOS 26.5 SDK declares `MDLAsset` available from iOS 9.0; the method has no later
availability annotation. The generated `objc2-model-io` 0.3.2 binding exposes the class method under
its `MDLAsset` feature. Link probes use iOS 10.0 for device and iOS 14.0 for Simulator, Rust 1.94.1
target floors; they build and inspect the consumer but do not run it.

The package enables `objc2-model-io` with default features off and only `MDLAsset`. The generated
binding enables the Foundation types required by that feature; the wrapper uses `NSString` to pass
the caller's string. See [the iOS plan](../../PLAN_IOS_MODELIO_STATUS.md).

The isolated package gate passed for host, iOS device, and iOS Simulator checks, strict Clippy on
both Apple targets, feature-tree inspection, build-only link probes, and iOS rustdoc. The probes
imported `Foundation`, `ModelIO`, `libSystem.B.dylib`, and `libobjc.A.dylib`, with deployment
`minos` 10.0 (device) and 14.0 (Simulator). They were inspected but not run.

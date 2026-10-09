# PLAN_IOS_IMAGE_METADATA.md — Workstream B26: iOS ImageIO Metadata Backend

## Objective

Implement D21 with public ImageIO metadata APIs for caller-owned encoded bytes. Do not claim full CoreImage or general ImageIO support.

## Dependencies

- Foundation A and iOS runtime B are integrated
- D21 `framework-image` is integrated
- Inspect the active public iOS SDK headers before any minimum-version or import claim
- `objc2-image-io` 0.3.2 generated bindings expose the required ImageIO C API

## Read first

- `PLAN_CAPABILITIES_IMAGE_METADATA.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- `docs/APP_STORE_COMPLIANCE.md`
- Apple [`CGImageSource`](https://developer.apple.com/documentation/imageio/cgimagesource)
- Apple [`CGImageSourceCopyPropertiesAtIndex`](https://developer.apple.com/documentation/imageio/cgimagesourcecopypropertiesatindex%28_%3A_%3A_%3A%29)
- Apple [`Image Properties`](https://developer.apple.com/documentation/imageio/image-properties)

## Write scope

- `platform/ios/ios-image-io/**`
- `docs/ios/image-metadata.md`

Do not edit D21 portable files, root workspace/Cargo files, the canonical capability JSON, aggregate plans, CI, docs indexes, `tools/xtask`, other iOS backends, or Swift ABI code. Root owns central reconciliation.

## Required implementation

- Implement `ImageMetadataBackend` with a zero-sized/caller-owned backend and no global service
- Use only `CGImageSourceCreateWithData`, `CGImageSourceGetCount`, `CGImageSourceCopyPropertiesAtIndex` at index zero, and `kCGImagePropertyPixelWidth` / `kCGImagePropertyPixelHeight`
- Use generated `objc2-image-io` 0.3.2 bindings, with default features off and only `CGImageSource` + `CGImageProperties`; enable only required CoreFoundation types
- Copy the borrowed encoded slice into owned `CFData`; never retain the Rust slice after the call
- Read only image count and first-image encoded pixel width/height; do not call an image-creation/raster API
- Keep platform/generated types inside this iOS crate; no CoreImage, CoreGraphics image API, UIKit, Swift, private API, global runtime, or framework-wide setup
- Map null source, absent properties, wrong property types, zero/invalid dimensions, and count overflow into documented portable errors
- State that ImageIO may parse/allocate internal state; no claim that its internals avoid pixel work, only that this API requests and returns no raster/`CGImage`
- State that the operation is synchronous, runs on the caller's thread, has no cancellation, and has no encoded-byte size limit

## Validation and handoff

- Run device/simulator `cargo check` and strict Clippy for `ios-image-io`
- Run the package-local link/import check for ImageIO + CoreFoundation symbols and absence of CoreImage, CoreGraphics raster-creation, UIKit, Swift, or unrelated frameworks
- Record Xcode/SDK/Rust versions; report the SDK header API floor separately from the SDK's minimum link deployment target
- Document CFData copy cost, unspecified ImageIO internal memory/CPU, error mapping, and compile/link-only evidence limits
- Report changed files, commit SHA, exact commands, deviations, and unresolved assumptions. Do not push.

# iOS ImageIO metadata

## Scope

`ios-image-io` implements the portable `framework-image` contract with public ImageIO C APIs. It exposes one synchronous operation over caller-owned encoded bytes. It returns source image count and encoded pixel width/height for source index zero. It is not a CoreImage backend, image renderer, pixel decoder, file reader, or general image API.

## Use

```rust,ignore
use framework_image::ImageMetadataReader;
use ios_image_io::IosImageMetadataBackend;

fn dimensions(encoded: &[u8]) -> Result<framework_image::ImageMetadata, framework_image::ImageMetadataError> {
    let mut reader = ImageMetadataReader::new(IosImageMetadataBackend::new());
    reader.read_metadata(encoded)
}
```

`IosImageMetadataBackend` is zero-sized and needs no setup or permission. The operation runs synchronously on the caller's thread. It does not dispatch, start an executor, retain the input, or offer cancellation. Keep the call off latency-sensitive threads when the encoded input or platform codec cost is not known.

## Native path, ownership, and cost

For each call, the backend copies the entire `&[u8]` into an owned `CFData`, creates a `CGImageSource`, reads `CGImageSourceGetCount`, then requests `CGImageSourceCopyPropertiesAtIndex` for index zero. It reads only `kCGImagePropertyPixelWidth` and `kCGImagePropertyPixelHeight` as checked `CFNumber` values. All native values are local to the call and are released when it returns. No Rust byte pointer or native object escapes.

The full encoded input is copied once into `CFData`; this adds memory proportional to the encoded input size plus the caller's original buffer for the duration of the call. `CFData` creation can fail with a resource-exhausted error. There is no byte-size limit, peak-memory bound, CPU/latency bound, or promise that ImageIO avoids internal parsing, buffering, or codec work. The code does not request or return a raster, `CGImage`, pixel buffer, or decoded image. This is an API/result limit, not a guarantee about ImageIO implementation internals.

Dimensions are encoded pixel dimensions. The backend does not apply EXIF orientation, crop, scale, color profile, or display transforms. `image_count` reports ImageIO's source count; this may include frames for some formats, but the crate does not expose frame iteration, timing, or format-specific guarantees. No explicit format allow-list is imposed; accepted formats and metadata details may vary with iOS/ImageIO versions.

## Errors

- `InvalidImageData`: ImageIO returns no source/properties, a required key is absent, or a property has a non-`CFNumber` value
- `InvalidDimensions`: width/height is zero, negative, fractional, non-finite, or above `u32::MAX`
- `InvalidImageCount`: source count is zero or above `u32::MAX`
- `Backend(ErrorKind::ResourceExhausted)`: CoreFoundation cannot create the copied `CFData`

The selected APIs return null/optional values without an NSError or detailed parser code, so malformed/unsupported metadata maps to `InvalidInput` with no platform code. The backend does not distinguish every codec failure from malformed input.

## Availability and framework imports

The active iPhoneOS 26.5 SDK header marks the selected `CGImageSourceCreateWithData`, `CGImageSourceGetCount`, `CGImageSourceCopyPropertiesAtIndex`, `kCGImagePropertyPixelWidth`, and `kCGImagePropertyPixelHeight` declarations `IMAGEIO_AVAILABLE_STARTING(10.4, 4.0)`. The API floor for this selected surface is iOS 4.0. The same SDK's `SDKSettings.plist` sets `MinimumDeploymentTarget` to iOS 12.0, so this host cannot verify a binary deployment target below iOS 12.0. This execution used macOS 26.6.2, Xcode 26.6 build 17F113, iPhoneOS/iPhoneSimulator SDK 26.5, Rust 1.94.1, LLVM 21.1.8. The repository's planned Xcode 27.x baseline is not met.

The backend depends on generated `objc2-image-io` 0.3.2 with only `CGImageSource` and `CGImageProperties` enabled, plus `objc2-core-foundation` 0.3.2 with `CFData`, `CFDictionary`, `CFNumber`, and `CFString`. Both dependencies disable default features. ImageIO and CoreFoundation are the intended Apple frameworks; CoreGraphics, CoreImage, UIKit, Photos, Swift ABI, and Swift runtime are not part of the selected surface. No permission, `Info.plist` key, entitlement, or main-thread proof is needed for these C/CoreFoundation calls.

The generated bindings provide the public function declarations, ownership wrappers, and CoreFoundation type checks; a handwritten ABI or metadata parser would add more unsafe code without narrowing the API. The `CFData` binding feature brings a small `bitflags` dependency in the generated crate graph. No parser, image decoder crate, executor, allocator crate, or Objective-C runtime feature is enabled. All binding types stay private to `ios-image-io`, so another ImageIO binding or direct public C adapter can replace this implementation without a portable API change.

Apple references: [`CGImageSource`](https://developer.apple.com/documentation/imageio/cgimagesource), [`CGImageSourceCopyPropertiesAtIndex`](https://developer.apple.com/documentation/imageio/cgimagesourcecopypropertiesatindex%28_%3A_%3A_%3A%29), and [`Image Properties`](https://developer.apple.com/documentation/imageio/image-properties).

## Validation limits

The package-local `check-ios.sh` runs device/simulator compile and strict Clippy gates. `check-link-imports.sh` builds a temporary static-library consumer outside the repo, checks selected undefined ImageIO/CoreFoundation symbols, and rejects unrelated symbol families/features. This is archive symbol-import evidence, not a final app binary's `otool -L` framework list. No live image was passed to ImageIO, so format coverage, metadata parity, malformed-file behavior, memory use, latency, and runtime thread behavior are not measured.

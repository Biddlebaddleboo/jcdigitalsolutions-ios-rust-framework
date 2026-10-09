# Image metadata

## Scope

`framework-image` defines a portable, `no_std` contract for encoded image dimensions and source image count. Its current contract is narrow: dimensions for source index zero plus the total count reported by a backend. It does not claim full CoreImage or general ImageIO support.

## Rust API

```rust,ignore
use framework_image::{ImageMetadataBackend, ImageMetadataReader};
use ios_image_io::IosImageMetadataBackend;

fn inspect(encoded: &[u8]) -> Result<framework_image::ImageMetadata, framework_image::ImageMetadataError> {
    let mut reader = ImageMetadataReader::new(IosImageMetadataBackend::new());
    reader.read_metadata(encoded)
}
```

The generic `ImageMetadataReader<B>` owns a concrete `ImageMetadataBackend`; selection is static. The call is synchronous. The portable layer borrows the input only for the call, makes no byte copy, retains no pointer, creates no executor, and exposes framework-owned scalar values only.

`ImageDimensions` has nonzero `u32` encoded pixel width and height. `pixel_count()` returns their exact `u64` product. `ImageMetadata::dimensions()` describes image index zero; `image_count()` is the backend's nonzero `u32` count for the full source. A backend may report animation frames or other source images in that count. The contract does not define animation timing or per-frame dimensions.

Dimensions describe the encoded image. The portable contract does not apply orientation, crop, display scale, color profile, or output-size transforms. It also has no format allow-list, normalization, or cross-OS codec-parity promise. Errors classify malformed/unsupported data or unrepresentable metadata as `InvalidInput`; a backend may return `Backend(Error)` to preserve another stable category and optional native code.

## iOS backend

The separate [iOS ImageIO guide](../ios/image-metadata.md) describes the caller-thread ImageIO implementation. It reads metadata only through `CGImageSource`; it does not call a raster/`CGImage` creation API and returns no pixel buffer. This statement describes the requested API/result, not a guarantee about ImageIO's internal parsing, temporary memory, or codec work.

## Limits

- No pixel decode, raster buffer, encode, transform, filter, edit, render, or display API
- No CoreImage, CoreGraphics image object, GPU, UIKit, Photos, or file/path integration
- No EXIF/GPS/ICC export or arbitrary metadata dictionary
- No animation scheduling, frame iteration, orientation transform, color management, or format equivalence claim
- No encoded-byte size, CPU-time, latency, or peak-memory bound
- No async operation or cancellation behavior

Portable validation commands:

```sh
cargo fmt --all -- --check
cargo test -p framework-image
cargo check -p framework-image --no-default-features
git diff --check
```

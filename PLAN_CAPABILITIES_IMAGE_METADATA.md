# PLAN_CAPABILITIES_IMAGE_METADATA.md — Workstream D21: Portable Image Metadata Contract

## Objective

Add a small `no_std` Rust contract for image dimensions and source image count. This is a metadata-only slice of row 057 (`CoreImage/ImageIO`); it is not full CoreImage or general image support.

## Dependencies

- Foundation A is integrated
- D1 portable capability conventions are integrated

## Write scope

- `crates/framework-image/**`
- `docs/capabilities/image-metadata.md`

Do not edit root workspace/Cargo files, the canonical capability JSON, aggregate plans, CI, docs indexes, `tools/xtask`, iOS backends, or other capability crates. Root owns central reconciliation.

## Required contract

- `#![no_std]`, no third-party dependency, no platform type, no `std`, no global runtime, and no dynamic dispatch in the public backend contract
- Framework-owned `ImageDimensions` with nonzero `u32` encoded pixel width and height, plus a `u64` pixel count
- Framework-owned `ImageMetadata` with first-image dimensions and a nonzero `u32` source image count
- Explicit source index semantics: dimensions describe image index zero; count describes all source images/frames reported by the backend
- Static synchronous `ImageMetadataBackend` selected by a generic `ImageMetadataReader<B>`
- Input is a caller-owned `&[u8]` borrowed only for the call; the portable layer makes no copy and has no buffer-size limit
- Stable portable errors for invalid data/metadata and a backend error that keeps `ErrorKind` and optional platform code
- No async, executor, cancellation, permission, or hidden initialization claim

## Explicit non-goals

- Pixel decode or raster/pixel-buffer output
- Image encode, transform, edit, filter, render, or display
- CoreImage, CoreGraphics image objects, GPU operations, or image view/UI APIs
- EXIF/GPS/ICC or arbitrary metadata export
- Format allow-list, format normalization, or cross-OS codec parity claim
- Animation timing, per-frame iteration, orientation normalization, color management, or image-size policy

## Deterministic tests and gates

- Validate zero dimensions, valid bounds, pixel-count math, and zero image count
- Validate static fake-backend delegation and borrowed-input semantics
- Validate portable error-category/native-code access
- Run `cargo fmt --all -- --check`, `cargo test -p framework-image`, `cargo check -p framework-image --no-default-features`, and `git diff --check`
- Inspect public API for platform types, `std`, dynamic dispatch, runtime setup, and unrelated dependencies

## Handoff

Report changed files, commit SHA, exact checks, any deviation, and unresolved backend/runtime limits. Root reconciles row 057 only after review.

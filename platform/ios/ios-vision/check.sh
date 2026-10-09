#!/bin/sh
set -eu

cargo fmt --manifest-path crates/framework-vision/Cargo.toml -- --check
cargo fmt --manifest-path platform/ios/ios-vision/Cargo.toml -- --check
cargo check --locked --no-default-features -p framework-vision
cargo clippy --locked --lib --no-default-features -p framework-vision -- -D warnings
cargo doc --locked --no-deps -p framework-vision
cargo check --locked --lib -p ios-vision
cargo clippy --locked --lib -p ios-vision -- -D warnings

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked --lib -p ios-vision --target "$target"
    cargo clippy --locked --lib -p ios-vision --target "$target" -- -D warnings
done

cargo doc --locked --no-deps -p ios-vision

features=$(cargo tree --locked -p ios-vision --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-vision feature "VNRequest"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-vision feature "VNRecognizeTextRequest"' >/dev/null
if printf '%s\n' "$features" | grep -E 'objc2-vision feature "(default|VNRequestHandler|VNVideoProcessor|VNCoreMLRequest|VNDetectBarcodesRequest|VNDetectFaceRectanglesRequest|VNImageRegistrationRequest)"|objc2-(core-ml|image-io|core-video) feature'; then
    echo "out-of-scope Vision request or data features are enabled" >&2
    exit 1
fi

sh platform/ios/ios-vision/check-link-imports.sh

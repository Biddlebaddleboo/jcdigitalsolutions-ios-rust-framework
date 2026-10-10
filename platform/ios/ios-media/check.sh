#!/bin/sh
set -eu

cargo fmt --manifest-path crates/framework-media/Cargo.toml -- --check
cargo fmt --manifest-path platform/ios/ios-media/Cargo.toml -- --check
cargo check --locked --no-default-features -p framework-media
cargo clippy --locked --lib --no-default-features -p framework-media -- -D warnings
cargo doc --locked --no-deps -p framework-media
cargo check --locked -p ios-media --features videotoolbox
cargo clippy --locked --lib -p ios-media --features videotoolbox -- -D warnings

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-media --features videotoolbox --target "$target"
    cargo clippy --locked --lib -p ios-media --features videotoolbox --target "$target" -- -D warnings
done

cargo doc --locked --no-deps -p ios-media --features videotoolbox

default_features=$(cargo tree --locked -p ios-media --target aarch64-apple-ios -e features)
if printf '%s\n' "$default_features" | grep -q 'objc2-video-toolbox'; then
    echo "VideoToolbox capability leaked into the default ios-media feature graph" >&2
    exit 1
fi
if ! printf '%s\n' "$default_features" | grep -q 'objc2-core-media feature "CMTime"'; then
    echo "default ios-media features must preserve the CoreMedia time API" >&2
    exit 1
fi
features=$(cargo tree --locked -p ios-media --target aarch64-apple-ios --features videotoolbox -e features)
if printf '%s\n' "$features" | grep -E 'objc2-video-toolbox feature "(default|VTCompressionSession|VTVideoEncoderList)"'; then
    echo "out-of-scope VideoToolbox encode/list or default features are enabled" >&2
    exit 1
fi

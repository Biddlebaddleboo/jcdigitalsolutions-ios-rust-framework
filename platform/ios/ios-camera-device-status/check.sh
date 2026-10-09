#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-camera-device-status/src/lib.rs
manifest=platform/ios/ios-camera-device-status/Cargo.toml
if ! rg -q 'AVCaptureDevice::defaultDeviceWithMediaType\(media_type\)' "$source_file"; then
    echo 'no default video-device query found' >&2
    exit 1
fi
if rg -n 'requestAccess|AVCaptureDeviceInput::|AVCaptureSession::|startRunning|AVCaptureOutput|CMSampleBuffer|AVCapturePhoto' "$source_file"; then
    echo 'out-of-scope capture or authorization API found' >&2
    exit 1
fi
if ! rg -q 'objc2-av-foundation = \{ workspace = true, features = \["AVCaptureDevice", "AVMediaFormat"\] \}' "$manifest"; then
    echo 'unexpected objc2-av-foundation feature set' >&2
    exit 1
fi

cargo fmt --manifest-path "$manifest" -- --check
cargo check --locked -p ios-camera-device-status
cargo clippy --locked --all-targets -p ios-camera-device-status -- -D warnings
cargo doc --locked -p ios-camera-device-status --no-deps
cargo check --locked -p ios-camera-device-status --target aarch64-apple-ios
cargo check --locked -p ios-camera-device-status --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-camera-device-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-camera-device-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-camera-device-status --target aarch64-apple-ios --no-deps
sh platform/ios/ios-camera-device-status/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

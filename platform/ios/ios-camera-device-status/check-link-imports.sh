#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the camera-device link check" >&2
        exit 1
    fi
done

cat > target/ios-camera-device-status-imports-expected.txt <<'IMPORTS'
AVFoundation
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-camera-device-status --example camera_device_link_probe --target "$target"
    binary="target/$target/release/examples/camera_device_link_probe"
    imports="target/ios-camera-device-status-imports-$target.txt"
    libraries="target/ios-camera-device-status-libraries-$target.txt"
    symbols="target/ios-camera-device-status-symbols-$target.txt"
    strings_file="target/ios-camera-device-status-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-camera-device-status-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_getClass' "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    grep -q 'AVMediaTypeVideo' "$symbols"
    strings "$binary" > "$strings_file"
    grep -q 'AVCaptureDevice' "$strings_file"
    grep -q 'defaultDeviceWithMediaType:' "$strings_file"
    if grep -Eqi 'swift_' "$symbols" "$strings_file"; then
        echo "unexpected Swift runtime import in $symbols" >&2
        exit 1
    fi
done

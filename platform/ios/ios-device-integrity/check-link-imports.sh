#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the DeviceCheck link/import check" >&2
        exit 1
    fi
done

cat > target/ios-device-integrity-link-imports-expected.txt <<'IMPORTS'
DeviceCheck
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-device-integrity --example ios_device_integrity_link_probe --target "$target"
    binary="target/$target/release/examples/ios_device_integrity_link_probe"
    imports="target/ios-device-integrity-link-imports-$target.txt"
    libraries="target/ios-device-integrity-link-libraries-$target.txt"
    symbols="target/ios-device-integrity-link-symbols-$target.txt"
    strings_file="target/ios-device-integrity-link-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-device-integrity-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_' "$symbols"; then
        echo "unexpected Swift runtime symbol import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    for symbol in DCDevice DCAppAttestService isSupported; do
        grep -q "$symbol" "$strings_file"
    done
done

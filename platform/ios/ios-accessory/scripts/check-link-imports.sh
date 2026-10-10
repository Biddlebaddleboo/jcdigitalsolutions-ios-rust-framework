#!/bin/sh
set -eu

repo_root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s\n' "$tool is required for the ExternalAccessory link/import check" >&2
        exit 1
    fi
done

cat > target/ios-accessory-link-imports-expected.txt <<'IMPORTS'
ExternalAccessory
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-accessory --example ios_accessory_link_probe --target "$target"
    binary="target/$target/release/examples/ios_accessory_link_probe"
    imports="target/ios-accessory-link-imports-$target.txt"
    libraries="target/ios-accessory-link-libraries-$target.txt"
    symbols="target/ios-accessory-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-accessory-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|EASession|showBluetoothAccessoryPicker|registerForLocalNotifications|inputStream|outputStream' "$symbols"; then
        printf '%s\n' "unexpected Swift runtime or out-of-scope ExternalAccessory symbol import in $symbols" >&2
        exit 1
    fi
done

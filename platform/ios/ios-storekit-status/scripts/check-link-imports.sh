#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the StoreKit link check" >&2
        exit 1
    fi
done

cat > target/ios-storekit-status-imports-expected.txt <<'IMPORTS'
Foundation
StoreKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-storekit-status --example storekit_status_link_probe --target "$target"
    binary="target/$target/release/examples/storekit_status_link_probe"
    imports="target/ios-storekit-status-imports-$target.txt"
    libraries="target/ios-storekit-status-libraries-$target.txt"
    symbols="target/ios-storekit-status-symbols-$target.txt"
    strings_file="target/ios-storekit-status-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-storekit-status-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    strings "$binary" > "$strings_file"
    grep -q 'SKPaymentQueue' "$strings_file"
    grep -q 'canMakePayments' "$strings_file"
    if grep -Eqi 'swift_' "$symbols" "$strings_file"; then
        echo "unexpected Swift runtime import in $symbols" >&2
        exit 1
    fi
done

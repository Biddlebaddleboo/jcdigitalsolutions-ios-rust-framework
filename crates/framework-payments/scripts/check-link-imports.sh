#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the PassKit link check" >&2
        exit 1
    fi
done

cat > target/framework-payments-link-imports-expected.txt <<'IMPORTS'
Foundation
PassKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p framework-payments --example apple_pay_status_link_probe --target "$target"
    binary="target/$target/release/examples/apple_pay_status_link_probe"
    imports="target/framework-payments-link-imports-$target.txt"
    libraries="target/framework-payments-link-libraries-$target.txt"
    symbols="target/framework-payments-link-symbols-$target.txt"
    strings_file="target/framework-payments-link-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/framework-payments-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    strings "$binary" > "$strings_file"
    grep -q 'PKPaymentAuthorizationController' "$strings_file"
    grep -q 'canMakePayments' "$strings_file"
    if grep -Eqi 'swift_' "$symbols" "$strings_file"; then
        echo "unexpected Swift runtime import in $symbols" >&2
        exit 1
    fi
done

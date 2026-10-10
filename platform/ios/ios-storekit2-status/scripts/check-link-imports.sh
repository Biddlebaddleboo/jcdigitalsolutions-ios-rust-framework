#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the StoreKit 2 link check" >&2
        exit 1
    fi
done

cat > target/ios-storekit2-status-imports-expected.txt <<'IMPORTS'
StoreKit
libSystem.B.dylib
IMPORTS

swift_symbol='_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ'
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-storekit2-status --example storekit2_status_link_probe --target "$target"
    binary="target/$target/release/examples/storekit2_status_link_probe"
    imports="target/ios-storekit2-status-imports-$target.txt"
    libraries="target/ios-storekit2-status-libraries-$target.txt"
    symbols="target/ios-storekit2-status-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-storekit2-status-imports-expected.txt "$libraries"
    grep -Eq 'StoreKit\.framework/StoreKit.*weak' "$imports"

    nm -u "$binary" > "$symbols"
    grep -F "$swift_symbol" "$symbols"
    if grep -Eqi 'libswift|_swift_(retain|release|alloc|dealloc)' "$imports" "$symbols"; then
        echo "unexpected direct Swift runtime link in $binary" >&2
        exit 1
    fi
done

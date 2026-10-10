#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo diff grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS CommonCrypto link/import check" >&2
        exit 1
    fi
done

mkdir -p target
cat > target/ios-crypto-link-imports-expected.txt <<'IMPORTS'
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-crypto-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-crypto --example ios_crypto_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_crypto_link_probe"
    imports="target/ios-crypto-link-imports-$target.txt"
    libraries="target/ios-crypto-link-libraries-$target.txt"
    symbols="target/ios-crypto-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-crypto-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if ! grep -Fq '_CC_SHA256' "$symbols"; then
        echo "missing expected CommonCrypto import _CC_SHA256 in $symbols" >&2
        exit 1
    fi

    build_info="target/ios-crypto-link-build-$target.txt"
    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

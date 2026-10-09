#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo diff grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS key-support link/import check" >&2
        exit 1
    fi
done

mkdir -p target
cat > target/ios-key-support-link-imports-expected.txt <<'IMPORTS'
CoreFoundation
Security
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-key-support-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-key-support --example ios_key_support_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_key_support_link_probe"
    imports="target/ios-key-support-link-imports-$target.txt"
    libraries="target/ios-key-support-link-libraries-$target.txt"
    symbols="target/ios-key-support-link-symbols-$target.txt"
    build_info="target/ios-key-support-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-key-support-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -Fq '_SecKeyCreateWithData' "$symbols"
    grep -Fq '_SecKeyIsAlgorithmSupported' "$symbols"
    if grep -Eqi 'swift_|objc_msgSend|SecItem(Add|CopyMatching|Delete|Update)|SecKey(CreateRandomKey|GeneratePair|CreateSignature|CreateEncryptedData|CreateDecryptedData)' "$symbols"; then
        echo "unexpected Swift, Objective-C, keychain, key-generation, or cryptographic-operation import in $symbols" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

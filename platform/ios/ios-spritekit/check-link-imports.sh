#!/bin/sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$repo_root"

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS SpriteKit link/import check" >&2
        exit 1
    fi
done

mkdir -p target
cat > target/ios-spritekit-link-imports-expected.txt <<'IMPORTS'
CoreFoundation
Foundation
SpriteKit
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-spritekit-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" \
    CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-spritekit --example link_check --target "$target"
    binary="$target_dir/$target/release/examples/link_check"
    imports="target/ios-spritekit-link-imports-$target.txt"
    libraries="target/ios-spritekit-link-libraries-$target.txt"
    symbols="target/ios-spritekit-link-symbols-$target.txt"
    string_table="target/ios-spritekit-link-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-spritekit-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -Fq '_objc_msgSend' "$symbols"
    strings "$binary" > "$string_table"
    grep -Fq 'SKNode' "$string_table"
    grep -Fq 'position' "$string_table"
    grep -Fq 'setPosition:' "$string_table"
    if grep -Eiq 'swift|SceneKit|ModelIO|MetalPerformanceShaders|Security|CommonCrypto|_SKView|_SKScene|_SKRenderer' "$symbols"; then
        echo "unexpected Swift runtime or out-of-scope framework/API symbol in $symbols" >&2
        exit 1
    fi
    if grep -Eiq 'SKScene|SKView|SKRenderer|SCN[A-Z]' "$string_table"; then
        echo "unexpected scene, view, renderer, or SceneKit API string in $string_table" >&2
        exit 1
    fi

    build_info="target/ios-spritekit-link-build-$target.txt"
    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

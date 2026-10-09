#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS Photos link/import check" >&2
        exit 1
    fi
done

cat > target/ios-photos-link-imports-expected.txt <<'IMPORTS'
Foundation
Photos
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            sdk_name=iphoneos
            clang_target=arm64-apple-ios14.0
            ;;
        aarch64-apple-ios-sim)
            sdk_name=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            ;;
    esac

    sdk=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    xcrun --sdk "$sdk_name" clang -target "$clang_target" -isysroot "$sdk" -std=gnu11 -fsyntax-only platform/ios/ios-photos/examples/photos_api_floor.m

    target_dir="target/ios-photos-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET=14.0 CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-photos --example ios_photos_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_photos_link_import_probe"
    imports="target/ios-photos-link-imports-$target.txt"
    libraries="target/ios-photos-link-libraries-$target.txt"
    symbols="target/ios-photos-link-symbols-$target.txt"
    build_info="target/ios-photos-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-photos-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if ! grep -q 'objc_getClass' "$symbols" || ! grep -q 'objc_msgSend' "$symbols"; then
        echo "$target probe does not import Objective-C class and message lookup symbols" >&2
        exit 1
    fi
    if grep -Eqi 'swift_|PHAsset|PHImageManager|PHAssetChangeRequest|PHImageRequest|PhotosUI|UIImage|CGImage' "$symbols"; then
        echo "unexpected Swift runtime or out-of-scope Photos symbol import in $symbols" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "14.0" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected 14.0" >&2
        exit 1
    fi
done

if find platform/ios/ios-photos -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-photos" >&2
    exit 1
fi

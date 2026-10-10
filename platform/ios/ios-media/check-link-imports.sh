#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang diff grep nm otool sort vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS media link/import check" >&2
        exit 1
    fi
done

cat > target/ios-media-link-imports-expected.txt <<'IMPORTS'
CoreMedia
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=12.0
            sdk_name=iphoneos
            clang_target=arm64-apple-ios12.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            sdk_name=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            ;;
    esac

    sdk=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    xcrun --sdk "$sdk_name" clang -target "$clang_target" -isysroot "$sdk" -std=c11 -fsyntax-only platform/ios/ios-media/examples/cm_time_layout.c

    target_dir="target/ios-media-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-dead_strip_dylibs" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-media --example ios_media_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_media_link_import_probe"
    imports="target/ios-media-link-imports-$target.txt"
    libraries="target/ios-media-link-libraries-$target.txt"
    symbols="target/ios-media-link-symbols-$target.txt"
    build_info="target/ios-media-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-media-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if ! grep -q 'CMTimeMake' "$symbols"; then
        echo "$target probe does not import CMTimeMake" >&2
        exit 1
    fi
    if grep -Eqi 'swift_|Py[A-Z_]|_OBJC_(CLASS|METACLASS)_\$_|AVFoundation|AVCapture|CMSampleBuffer|CMBlockBuffer|CMTimeMakeWithSeconds|CMTimeGetSeconds|CMTimeCompare' "$symbols"; then
        echo "unexpected runtime or out-of-scope media symbol import in $symbols" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-media -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-media" >&2
    exit 1
fi

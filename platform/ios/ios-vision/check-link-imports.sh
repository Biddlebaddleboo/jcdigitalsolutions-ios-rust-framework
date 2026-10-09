#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS Vision link/import check" >&2
        exit 1
    fi
done

cat > target/ios-vision-link-imports-expected.txt <<'IMPORTS'
Foundation
Vision
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=13.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            ;;
    esac

    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="target/ios-vision-link-$target" cargo build --locked --release -p ios-vision --example ios_vision_link_probe --target "$target"
    binary="target/ios-vision-link-$target/$target/release/examples/ios_vision_link_probe"
    imports="target/ios-vision-link-imports-$target.txt"
    libraries="target/ios-vision-link-libraries-$target.txt"
    symbols="target/ios-vision-link-symbols-$target.txt"
    strings_file="target/ios-vision-strings-$target.txt"
    build_info="target/ios-vision-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-vision-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|VNRequestHandler|VNImageRequestHandler|VNVideoProcessor|CoreML|MLModel|AVCapture|UIApplication|UIView|UIWindow' "$symbols"; then
        echo "unexpected request-handler, image, camera, UI, Core ML, or Swift import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    grep -q 'VNRecognizeTextRequest' "$strings_file"
    grep -q 'supportedRevisions' "$strings_file"
    grep -q 'containsIndex' "$strings_file"
    if grep -Eqi 'VNRequestHandler|VNImageRequestHandler|VNVideoProcessor|CoreML|MLModel|AVCapture|UIApplication|UIView|UIWindow|swift_' "$strings_file"; then
        echo "unexpected request-handler, image, camera, UI, Core ML, or Swift selector in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

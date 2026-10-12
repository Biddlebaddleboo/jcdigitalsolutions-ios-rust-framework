#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
cd "$root"

for tool in awk cargo find grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS PhotosUI picker link check\n' "$tool" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            sdk_name=iphoneos
            clang_target=arm64-apple-ios14.0
            min_flag=-miphoneos-version-min=14.0
            ;;
        aarch64-apple-ios-sim)
            sdk_name=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            min_flag=-mios-simulator-version-min=14.0
            ;;
    esac

    sdk=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    xcrun --sdk "$sdk_name" clang -target "$clang_target" -isysroot "$sdk" \
        -fsyntax-only platform/ios/ios-photo-picker/examples/photo_picker_api_floor.m

    target_dir="target/ios-photo-picker-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET=14.0 RUSTFLAGS="-C link-arg=$min_flag" \
        CARGO_TARGET_DIR="$target_dir" cargo +1.94.1 build --locked --release \
        -p ios-photo-picker --example ios_photo_picker_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_photo_picker_link_probe"
    imports=$(otool -L "$binary")
    actual_libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | LC_ALL=C sort)
    expected_libraries=$(printf '%s\n' Foundation PhotosUI UIKit libSystem.B.dylib libobjc.A.dylib | LC_ALL=C sort)
    if [ "$actual_libraries" != "$expected_libraries" ]; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$actual_libraries" >&2
        exit 1
    fi

    symbols=$(nm -u "$binary")
    for symbol in objc_msgSend objc_getClass NSTemporaryDirectory; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            printf '%s probe does not import %s\n' "$target" "$symbol" >&2
            exit 1
        }
    done
    binary_strings=$(strings "$binary")
    for required in \
        PHPickerViewController \
        'initWithConfiguration:' \
        'picker:didFinishPicking:' \
        'setSelectionLimit:' \
        'setPreferredAssetRepresentationMode:' \
        'imagesFilter' \
        'setFilter:' \
        'loadFileRepresentationForTypeIdentifier:completionHandler:' \
        'presentViewController:animated:completion:' \
        'dismissViewControllerAnimated:completion:'; do
        printf '%s\n' "$binary_strings" | grep -F "$required" >/dev/null || {
            printf '%s probe does not link the public PhotosUI/Foundation selector %s\n' "$target" "$required" >&2
            exit 1
        }
    done
    if printf '%s\n%s\n' "$imports" "$symbols" | grep -E 'libswift|SwiftUI|_swift_|_\$s[0-9]' >/dev/null; then
        printf '%s probe unexpectedly imports Swift or SwiftUI\n' "$target" >&2
        exit 1
    fi

    build=$(vtool -show-build "$binary")
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+14([.]0)?([[:space:]]|$)' >/dev/null || {
        printf '%s probe does not declare the iOS 14.0 deployment minimum\n' "$target" >&2
        exit 1
    }
    printf '%s PhotosUI picker/framework imports and iOS 14.0 link evidence passed; probe not executed\n' "$target"
done

if find platform/ios/ios-photo-picker -type f -name '*.swift' -print -quit | grep -q .; then
    echo 'Swift source found under platform/ios/ios-photo-picker' >&2
    exit 1
fi

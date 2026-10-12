#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
cd "$root"

for tool in awk cargo find grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS ContactsUI picker link check\n' "$tool" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            sdk_name=iphoneos
            minimum=10.0
            clang_target=arm64-apple-ios10.0
            min_flag=-miphoneos-version-min=10.0
            ;;
        aarch64-apple-ios-sim)
            sdk_name=iphonesimulator
            minimum=14.0
            clang_target=arm64-apple-ios14.0-simulator
            min_flag=-mios-simulator-version-min=14.0
            ;;
    esac

    sdk=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    xcrun --sdk "$sdk_name" clang -target "$clang_target" -isysroot "$sdk" \
        -fsyntax-only platform/ios/ios-contact-picker/examples/contact_picker_api_floor.m

    target_dir="target/ios-contact-picker-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$minimum" RUSTFLAGS="-C link-arg=$min_flag" \
        CARGO_TARGET_DIR="$target_dir" cargo +1.94.1 build --locked --release \
        -p ios-contact-picker --example ios_contact_picker_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_contact_picker_link_probe"
    imports=$(otool -L "$binary")
    actual_libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | LC_ALL=C sort)
    expected_libraries=$(printf '%s\n' Contacts ContactsUI Foundation UIKit libSystem.B.dylib libobjc.A.dylib | LC_ALL=C sort)
    if [ "$actual_libraries" != "$expected_libraries" ]; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$actual_libraries" >&2
        exit 1
    fi

    symbols=$(nm -u "$binary")
    for symbol in objc_msgSend objc_getClass; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            printf '%s probe does not import %s\n' "$target" "$symbol" >&2
            exit 1
        }
    done
    binary_strings=$(strings "$binary")
    for required in \
        CNContactPickerViewController \
        'setDisplayedPropertyKeys:' \
        'contactPicker:didSelectContact:' \
        'contactPickerDidCancel:' \
        'presentViewController:animated:completion:' \
        'dismissViewControllerAnimated:completion:'; do
        printf '%s\n' "$binary_strings" | grep -F "$required" >/dev/null || {
            printf '%s probe does not link the public ContactsUI class/selector %s\n' "$target" "$required" >&2
            exit 1
        }
    done
    if printf '%s\n%s\n' "$imports" "$symbols" | grep -E 'libswift|SwiftUI|PhotosUI|_swift_|_\$s[0-9]' >/dev/null; then
        printf '%s probe unexpectedly imports Swift or unrelated UI frameworks\n' "$target" >&2
        exit 1
    fi

    build=$(vtool -show-build "$binary")
    printf '%s\n' "$build" | grep -E "(minos|version)[[:space:]]+$minimum([.]0)?([[:space:]]|$)" >/dev/null || {
        printf '%s probe does not declare the iOS %s deployment minimum\n' "$target" "$minimum" >&2
        exit 1
    }
    printf '%s ContactsUI picker/framework import and iOS %s link evidence passed; probe not executed\n' \
        "$target" "$minimum"
done

if find platform/ios/ios-contact-picker -type f -name '*.swift' -print -quit | grep -q .; then
    echo 'Swift source found under platform/ios/ios-contact-picker' >&2
    exit 1
fi

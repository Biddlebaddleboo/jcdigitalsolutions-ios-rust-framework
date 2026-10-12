#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in cargo grep nm otool strings vtool awk; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the custom render host link check" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-ui-custom-render-host-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-minimal --target "$target"
    binary="$target_dir/$target/release/ios-minimal"
    imports=$(otool -L "$binary")
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    for framework in UIKit CoreGraphics; do
        printf '%s\n' "$imports" | grep -F "/$framework.framework/$framework" >/dev/null || {
            echo "$target custom-render host probe does not link $framework" >&2
            exit 1
        }
    done
    for symbol in objc_getClass objc_msgSend CGImageCreate CGDataProviderCreateWithCFData \
        CGColorSpaceCreateDeviceRGB UIKeyboardWillChangeFrameNotification \
        UIKeyboardWillShowNotification UIKeyboardWillHideNotification \
        UIKeyboardFrameEndUserInfoKey; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target custom-render host probe is missing symbol $symbol" >&2
            exit 1
        }
    done
    for selector in UIImageView UITextView 'imageWithCGImage:' 'setImage:' 'setHidden:' \
        'safeAreaInsets' 'contentScaleFactor' 'applicationState' 'textViewDidChange:' \
        'textViewDidChangeSelection:' 'selectedTextRange' 'setSelectedTextRange:' \
        'markedTextRange' 'unmarkText' 'becomeFirstResponder' 'resignFirstResponder' \
        'keyboardWillChangeFrame:' 'keyboardWillShow:' 'keyboardWillHide:'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            echo "$target custom-render host probe is missing selector or class $selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eq 'swift_|Swift\.framework|libswift|(^|[^[:alnum:]_])Py[A-Z_]|Python\.framework|libpython'; then
        echo "unexpected Swift/Python runtime import in $target custom-render host probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target custom-render host probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s custom-render app UIKit/CoreGraphics imports verified at iOS %s; app not executed\n' \
        "$target" "$deployment_target"
done

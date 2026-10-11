#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo grep nm otool strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS UI text-field link check" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-ui-text-field-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_text_field_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_text_field_link_probe"
    imports=$(otool -L "$binary")
    printf '%s\n' "$imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target text-field probe does not link UIKit" >&2
        exit 1
    }
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    printf '%s\n' "$symbols" | grep -F 'objc_getClass' >/dev/null || {
        echo "$target text-field probe is missing Objective-C runtime class lookup" >&2
        exit 1
    }
    printf '%s\n' "$binary_strings" | grep -Fx 'UITextField' >/dev/null || {
        echo "$target text-field probe is missing the public UITextField runtime class name" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F 'objc_msgSend' >/dev/null || {
        echo "$target text-field probe is missing Objective-C message dispatch" >&2
        exit 1
    }
    for selector in 'setText:' 'text' 'setPlaceholder:' 'placeholder' \
        'becomeFirstResponder' 'resignFirstResponder' 'setSecureTextEntry:' \
        'setKeyboardType:' 'textFieldChanged:' \
        'removeTarget:action:forControlEvents:'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            echo "$target text-field probe is missing the $selector selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eqi 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target text-field probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s UIKit/UITextField selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"
done

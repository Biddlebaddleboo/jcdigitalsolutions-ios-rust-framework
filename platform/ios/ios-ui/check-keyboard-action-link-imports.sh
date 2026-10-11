#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo grep nm otool strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the UIKit responder-action link check\n' "$tool" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-ui-keyboard-action-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo +1.94.1 build --locked --offline --release -p ios-ui \
        --example ios_ui_keyboard_action_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_keyboard_action_link_probe"
    imports=$(otool -L "$binary")
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")

    printf '%s\n' "$imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        printf '%s probe does not link UIKit\n' "$target" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F 'objc_msgSend' >/dev/null || {
        printf '%s probe is missing Objective-C message dispatch\n' "$target" >&2
        exit 1
    }
    printf '%s\n' "$binary_strings" | grep -F 'UIApplication' >/dev/null || {
        printf '%s probe is missing the UIApplication class reference\n' "$target" >&2
        exit 1
    }
    for selector in 'sendAction:to:from:forEvent:' 'resignFirstResponder'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            printf '%s probe is missing the %s selector\n' "$target" "$selector" >&2
            exit 1
        }
    done
    if printf '%s\n%s\n' "$imports" "$symbols" | grep -Eqi 'libswift|SwiftUI|_swift_|_\$s[0-9]|python|Py[A-Z_]'; then
        printf '%s probe has an unexpected Swift or Python runtime import\n' "$target" >&2
        exit 1
    fi

    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        printf '%s probe has deployment target %s; expected %s\n' \
            "$target" "${actual_deployment_target:-unknown}" "$deployment_target" >&2
        exit 1
    fi
    printf '%s UIKit responder-action selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"
done

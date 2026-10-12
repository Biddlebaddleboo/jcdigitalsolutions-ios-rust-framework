#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in cargo grep nm otool strings vtool awk; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the custom-render accessibility link check" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-ui-custom-render-accessibility-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-ui \
            --example ios_ui_custom_render_accessibility_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_custom_render_accessibility_link_probe"
    imports=$(otool -L "$binary")
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    printf '%s\n' "$imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target custom-render accessibility probe does not link UIKit" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F objc_msgSend >/dev/null || {
        echo "$target custom-render accessibility probe is missing objc_msgSend" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F UIAccessibilityPostNotification >/dev/null || {
        echo "$target custom-render accessibility probe is missing UIAccessibilityPostNotification" >&2
        exit 1
    }
    for selector in UIAccessibilityElement 'initWithAccessibilityContainer:' \
        'setAccessibilityFrameInContainerSpace:' 'setAccessibilityElements:' \
        UIAccessibilityCustomAction 'initWithName:target:selector:' \
        'performAccessibilityAction:' 'setAccessibilityCustomActions:'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            echo "$target custom-render accessibility probe is missing selector or class $selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eq 'swift_|Swift\.framework|libswift|(^|[^[:alnum:]_])Py[A-Z_]|Python\.framework|libpython'; then
        echo "unexpected Swift/Python runtime import in $target custom-render accessibility probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target custom-render accessibility probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s custom-render accessibility imports verified at iOS %s; app not executed\n' \
        "$target" "$deployment_target"
done

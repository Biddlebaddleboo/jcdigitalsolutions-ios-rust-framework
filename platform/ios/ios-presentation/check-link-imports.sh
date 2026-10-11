#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo find grep nm otool strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS presentation link check" >&2
        exit 1
    fi
done

if find platform/ios/ios-presentation -type f -name '*.swift' -print -quit | grep -q .; then
    echo "unexpected Swift source in ios-presentation" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-presentation-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-presentation \
        --example destructive_confirmation_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/destructive_confirmation_link_probe"
    imports=$(otool -L "$binary")
    for framework in CoreFoundation Foundation UIKit; do
        printf '%s\n' "$imports" | grep -F "/${framework}.framework/${framework}" >/dev/null || {
            echo "$target presentation probe does not link ${framework}.framework" >&2
            exit 1
        }
    done
    for runtime in libobjc.A.dylib libSystem.B.dylib; do
        printf '%s\n' "$imports" | grep -F "/${runtime}" >/dev/null || {
            echo "$target presentation probe does not link ${runtime}" >&2
            exit 1
        }
    done
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target presentation probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for class in UIAlertController UIAlertAction UIViewController UIView; do
        printf '%s\n' "$binary_strings" | grep -Fx "$class" >/dev/null || {
            echo "$target presentation probe is missing public UIKit class $class" >&2
            exit 1
        }
    done
    for selector in 'actionWithTitle:style:handler:' 'addAction:' 'presentViewController:animated:completion:' \
        'popoverPresentationController' 'setSourceView:' 'setSourceRect:' 'currentDevice' \
        'userInterfaceIdiom' 'isBeingPresented' 'isBeingDismissed' 'presentedViewController' \
        'viewIfLoaded' 'window'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            echo "$target presentation probe is missing the $selector selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eiq 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target presentation probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s UIKit destructive-confirmation selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"
done

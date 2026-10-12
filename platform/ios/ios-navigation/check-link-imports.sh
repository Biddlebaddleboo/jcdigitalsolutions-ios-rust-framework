#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo find grep nm otool strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS navigation link check" >&2
        exit 1
    fi
done

if find platform/ios/ios-navigation -type f \( -name '*.swift' -o -name '*.py' \) -print -quit | grep -q .; then
    echo "unexpected Swift or Python source in ios-navigation" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-navigation-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-navigation \
        --example ios_navigation_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_navigation_link_probe"
    imports=$(otool -L "$binary")
    for framework in CoreFoundation CoreGraphics CoreText Foundation UIKit; do
        printf '%s\n' "$imports" | grep -F "/${framework}.framework/${framework}" >/dev/null || {
            echo "$target navigation probe does not link ${framework}.framework" >&2
            exit 1
        }
    done
    for runtime in libobjc.A.dylib libSystem.B.dylib; do
        printf '%s\n' "$imports" | grep -F "/${runtime}" >/dev/null || {
            echo "$target navigation probe does not link ${runtime}" >&2
            exit 1
        }
    done
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target navigation probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for class in UINavigationController UIBarButtonItem UIAction UIImage UIViewController; do
        printf '%s\n' "$binary_strings" | grep -Fx "$class" >/dev/null || {
            echo "$target navigation probe is missing public UIKit class $class" >&2
            exit 1
        }
    done
    for selector in 'initWithRootViewController:' 'setView:' 'setTitle:' \
        'pushViewController:animated:' 'popViewControllerAnimated:' \
        'navigationItem' 'setRightBarButtonItem:' 'actionWithTitle:image:identifier:handler:' \
        'initWithPrimaryAction:' 'systemImageNamed:'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            echo "$target navigation probe is missing the $selector selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eqi 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target navigation probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s UIKit navigation/action selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"
done

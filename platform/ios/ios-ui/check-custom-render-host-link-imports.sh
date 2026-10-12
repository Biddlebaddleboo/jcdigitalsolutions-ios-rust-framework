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
    if printf '%s\n' "$imports" | grep -F '/Metal.framework/Metal' >/dev/null; then
        echo "$target default CPU custom-render host unexpectedly links Metal" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target custom-render host probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s custom-render app UIKit/CoreGraphics imports verified at iOS %s; app not executed\n' \
        "$target" "$deployment_target"

    metal_target_dir="target/ios-ui-custom-render-metal-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$metal_target_dir" \
        cargo build --locked --offline --release -p ios-minimal --features custom-render-metal-demo --target "$target"
    metal_binary="$metal_target_dir/$target/release/ios-minimal"
    metal_imports=$(otool -L "$metal_binary")
    metal_symbols=$(nm -u "$metal_binary")
    metal_strings=$(strings "$metal_binary")
    for framework in Metal QuartzCore; do
        printf '%s\n' "$metal_imports" | grep -F "/$framework.framework/$framework" >/dev/null || {
            echo "$target optional Metal probe does not link $framework" >&2
            exit 1
        }
    done
    printf '%s\n' "$metal_symbols" | grep -F MTLCreateSystemDefaultDevice >/dev/null || {
        echo "$target optional Metal probe is missing MTLCreateSystemDefaultDevice" >&2
        exit 1
    }
    for selector in CAMetalLayer nextDrawable 'setPixelFormat:' 'setDrawableSize:' \
        'replaceRegion:mipmapLevel:withBytes:bytesPerRow:' 'copyFromTexture:toTexture:' \
        presentDrawable: waitUntilCompleted; do
        printf '%s\n' "$metal_strings" | grep -F "$selector" >/dev/null || {
            echo "$target optional Metal probe is missing selector or class $selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$metal_symbols" "$metal_imports" | grep -Eq 'swift_|Swift\.framework|libswift|(^|[^[:alnum:]_])Py[A-Z_]|Python\.framework|libpython'; then
        echo "unexpected Swift/Python runtime import in $target optional Metal probe" >&2
        exit 1
    fi
    metal_deployment_target=$(vtool -show-build "$metal_binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$metal_deployment_target" != "$deployment_target" ]; then
        echo "$target optional Metal probe has deployment target ${metal_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s optional Metal imports and symbols verified at iOS %s; app not executed\n' \
        "$target" "$deployment_target"

    display_link_target_dir="target/ios-ui-display-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$display_link_target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_display_link_link_probe --target "$target"
    display_link_binary="$display_link_target_dir/$target/release/examples/ios_ui_display_link_link_probe"
    display_link_imports=$(otool -L "$display_link_binary")
    display_link_symbols=$(nm -u "$display_link_binary")
    display_link_strings=$(strings "$display_link_binary")
    for framework in Foundation QuartzCore; do
        printf '%s\n' "$display_link_imports" | grep -F "/$framework.framework/$framework" >/dev/null || {
            echo "$target display-link probe does not link $framework" >&2
            exit 1
        }
    done
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$display_link_symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target display-link probe is missing $symbol" >&2
            exit 1
        }
    done
    for selector in CADisplayLink JcdIosDisplayLinkTarget displayLinkWithTarget:selector: \
        addToRunLoop:forMode: displayLinkDidFire: timestamp targetTimestamp duration \
        invalidate setPaused:; do
        printf '%s\n' "$display_link_strings" | grep -F "$selector" >/dev/null || {
            echo "$target display-link probe is missing selector or class $selector" >&2
            exit 1
        }
    done
    if printf '%s\n' "$display_link_symbols" "$display_link_imports" | grep -Eq 'swift_|Swift\.framework|libswift|(^|[^[:alnum:]_])Py[A-Z_]|Python\.framework|libpython'; then
        echo "unexpected Swift/Python runtime import in $target display-link probe" >&2
        exit 1
    fi
    display_link_deployment_target=$(vtool -show-build "$display_link_binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$display_link_deployment_target" != "$deployment_target" ]; then
        echo "$target display-link probe has deployment target ${display_link_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s CADisplayLink imports and selectors verified at iOS %s; app not executed\n' \
        "$target" "$deployment_target"
done

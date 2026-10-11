#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        deployment_target=10.0
        min_flag=-miphoneos-version-min=10.0
    else
        sdk=iphonesimulator
        deployment_target=14.0
        min_flag=-mios-simulator-version-min=14.0
    fi
    target_dir="$repo_root/target/ios-notification-responses-link-$target"
    SDKROOT="$(xcrun --sdk "$sdk" --show-sdk-path)" \
        IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" RUSTFLAGS="-C link-arg=$min_flag" \
        CARGO_TARGET_DIR="$target_dir" cargo build --locked --release \
            --manifest-path "$repo_root/Cargo.toml" -p ios-notification-responses \
            --example foreground_presentation_link_probe --target "$target"
    artifact="$target_dir/$target/release/examples/foreground_presentation_link_probe"
    imports="$target_dir/imports.txt"
    symbols="$target_dir/symbols.txt"
    otool -L "$artifact" >"$imports"
    nm -u -j "$artifact" >"$symbols"
    for framework in Foundation UserNotifications; do
        if ! grep -Fq "/System/Library/Frameworks/$framework.framework/$framework" "$imports"; then
            printf '%s missing required %s framework import\n' "$target" "$framework" >&2
            cat "$imports" >&2
            exit 1
        fi
    done
    for symbol in _objc_msgSend _objc_getClass _UNNotificationDefaultActionIdentifier _UNNotificationDismissActionIdentifier; do
        if ! grep -Fxq "$symbol" "$symbols"; then
            printf '%s missing required public notification symbol %s\n' "$target" "$symbol" >&2
            cat "$symbols" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|libswift' "$imports" "$symbols"; then
        printf '%s unexpectedly imports Swift runtime symbols\n' "$target" >&2
        exit 1
    fi
    selectors="$target_dir/selectors.txt"
    strings "$artifact" >"$selectors"
    for selector in userInfo allKeys objectAtIndex: objectForKey:; do
        if ! grep -Fxq "$selector" "$selectors"; then
            printf '%s missing notification user-info selector %s\n' "$target" "$selector" >&2
            cat "$selectors" >&2
            exit 1
        fi
    done
    build_info="$target_dir/build.txt"
    vtool -show-build "$artifact" >"$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        printf '%s has deployment target %s; expected %s\n' \
            "$target" "${actual_deployment_target:-unknown}" "$deployment_target" >&2
        exit 1
    fi
    printf '%s link/import/deployment check passed (iOS %s)\n' \
        "$target" "$deployment_target"
done

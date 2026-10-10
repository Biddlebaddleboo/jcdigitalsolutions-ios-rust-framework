#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        min_flag=-miphoneos-version-min=11.3
        minimum=11.3
    else
        sdk=iphonesimulator
        min_flag=-mios-simulator-version-min=14.0
        minimum=14.0
    fi
    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    export IPHONEOS_DEPLOYMENT_TARGET=$minimum
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo +1.94.1 build --locked --release --target "$target" \
        -p ios-homekit-identify-status --example homekit_identify_link_probe

    probe="target/$target/release/examples/homekit_identify_link_probe"
    imports=$(otool -L "$probe")
    printf '%s\n' "$imports" | grep -F '/HomeKit.framework/HomeKit' >/dev/null || {
        echo "$target probe does not import the public HomeKit framework" >&2
        exit 1
    }
    symbols=$(nm -u "$probe")
    printf '%s\n' "$symbols" | grep -E 'objc_msgSend|HMAccessory' >/dev/null || {
        echo "$target probe does not retain the HomeKit accessor import" >&2
        exit 1
    }
    build=$(xcrun vtool -show-build "$probe")
    printf '%s\n' "$build" | grep -E "(minos|version)[[:space:]]+$minimum([.]0)?([[:space:]]|$)" >/dev/null || {
        echo "$target probe does not declare the iOS $minimum deployment floor" >&2
        exit 1
    }
    printf '%s HomeKit framework import and iOS %s minimum verified; link example not executed\n' "$target" "$minimum"
done

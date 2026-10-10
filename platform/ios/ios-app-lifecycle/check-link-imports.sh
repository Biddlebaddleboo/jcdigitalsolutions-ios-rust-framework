#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for tool in awk cargo grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS app-lifecycle link/import check\n' "$tool" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        minimum=10.0
        min_flag=-miphoneos-version-min=10.0
    else
        sdk=iphonesimulator
        minimum=14.0
        min_flag=-mios-simulator-version-min=14.0
    fi

    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    export IPHONEOS_DEPLOYMENT_TARGET=$minimum
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo +1.94.1 build --locked --release --target "$target" \
        -p ios-app-lifecycle --example link_probe

    probe="target/$target/release/examples/link_probe"
    imports=$(otool -L "$probe")
    actual_libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | LC_ALL=C sort)
    expected_libraries=$(printf '%s\n' Foundation UIKit libSystem.B.dylib libobjc.A.dylib | LC_ALL=C sort)
    if [ "$actual_libraries" != "$expected_libraries" ]; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$actual_libraries" >&2
        exit 1
    fi

    symbols=$(nm -u "$probe")
    printf '%s\n' "$symbols" | grep -F '_objc_msgSend' >/dev/null || {
        printf '%s probe does not import objc_msgSend\n' "$target" >&2
        exit 1
    }
    for symbol in \
        UIApplicationDidBecomeActiveNotification \
        UIApplicationWillResignActiveNotification \
        UIApplicationWillEnterForegroundNotification \
        UIApplicationDidEnterBackgroundNotification \
        UIApplicationWillTerminateNotification
    do
        if ! printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null; then
            printf '%s probe does not import %s\n' "$target" "$symbol" >&2
            exit 1
        fi
    done
    binary_strings=$(strings "$probe")
    for selector in applicationState addObserverForName:object:queue:usingBlock:; do
        if ! printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null; then
            printf '%s probe does not reference the %s lifecycle API\n' "$target" "$selector" >&2
            exit 1
        fi
    done
    if printf '%s\n%s\n' "$imports" "$symbols" | grep -E 'libswift|SwiftUI|_swift_|_\$s[0-9]' >/dev/null; then
        printf '%s probe unexpectedly imports Swift runtime symbols\n' "$target" >&2
        exit 1
    fi

    build=$(xcrun vtool -show-build "$probe")
    printf '%s\n' "$build" | grep -E "(minos|version)[[:space:]]+$minimum([.]0)?([[:space:]]|$)" >/dev/null || {
        printf '%s probe does not declare the iOS %s deployment minimum\n' "$target" "$minimum" >&2
        exit 1
    }
    printf '%s public framework/Objective-C imports and iOS %s deployment minimum verified; link probe not executed\n' \
        "$target" "$minimum"
done

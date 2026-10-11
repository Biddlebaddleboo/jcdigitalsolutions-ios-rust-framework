#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for tool in cargo grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS dispatch timer link check\n' "$tool" >&2
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
        -p ios-runtime --example repeating_timer_link_probe --features dispatch-timer

    probe="target/$target/release/examples/repeating_timer_link_probe"
    imports=$(otool -L "$probe")
    symbols=$(nm -u "$probe")
    for symbol in \
        _dispatch_source_type_timer \
        _dispatch_source_create \
        _dispatch_source_set_event_handler \
        _dispatch_source_set_timer \
        _dispatch_source_cancel \
        _dispatch_activate \
        _Block_copy \
        _Block_release
    do
        if ! printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null; then
            printf '%s probe does not import %s\n' "$target" "$symbol" >&2
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
    printf '%s public libdispatch timer and Block imports plus iOS %s deployment minimum verified; link probe not executed\n' \
        "$target" "$minimum"
done

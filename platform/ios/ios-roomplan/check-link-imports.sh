#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        min_flag=-miphoneos-version-min=16.0
    else
        sdk=iphonesimulator
        min_flag=-mios-simulator-version-min=16.0
    fi
    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    export IPHONEOS_DEPLOYMENT_TARGET=16.0
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo build --locked --release --target "$target" -p ios-roomplan \
        --example roomplan_link_probe

    probe="target/$target/release/examples/roomplan_link_probe"
    imports=$(otool -L "$probe")
    printf '%s\n' "$imports" | grep -F '/RoomPlan.framework/RoomPlan' >/dev/null || {
        echo "$target probe does not import RoomPlan.framework" >&2
        exit 1
    }
    printf '%s\n' "$imports" | grep -F '/usr/lib/libSystem.B.dylib' >/dev/null || {
        echo "$target probe lacks the expected libSystem import" >&2
        exit 1
    }
    if printf '%s\n' "$imports" | grep -E 'Swift\.framework|libswift|libobjc' >/dev/null; then
        echo "$target probe unexpectedly imports Swift or Objective-C runtime libraries" >&2
        exit 1
    fi

    symbols=$(nm -u "$probe")
    printf '%s\n' "$symbols" | grep -F '_$s8RoomPlan0A14CaptureSessionCMa' >/dev/null || {
        echo "$target probe lacks the public RoomCaptureSession metadata accessor import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s8RoomPlan0A14CaptureSessionC11isSupportedSbvgZ' >/dev/null || {
        echo "$target probe lacks the public RoomCaptureSession.isSupported import" >&2
        exit 1
    }

    build=$(xcrun vtool -show-build "$probe")
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+16\.0([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$target probe does not declare the iOS 16.0 deployment floor" >&2
        exit 1
    }
    printf '%s RoomPlan Release link imports and iOS 16.0 minimum verified; probe not executed\n' "$target"
done

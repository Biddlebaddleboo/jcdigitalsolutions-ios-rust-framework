#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        min_flag=-miphoneos-version-min=15.4
    else
        sdk=iphonesimulator
        min_flag=-mios-simulator-version-min=15.4
    fi
    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    export IPHONEOS_DEPLOYMENT_TARGET=15.4
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo build --locked --release --target "$target" -p ios-proximity-reader \
        --example proximity_reader_link_probe

    probe="target/$target/release/examples/proximity_reader_link_probe"
    imports=$(otool -L "$probe")
    printf '%s\n' "$imports" | grep -F '/ProximityReader.framework/ProximityReader' >/dev/null || {
        echo "$target probe does not import ProximityReader.framework" >&2
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
    printf '%s\n' "$symbols" | grep -F '_$s15ProximityReader011PaymentCardB0CMa' >/dev/null || {
        echo "$target probe lacks the public PaymentCardReader metadata accessor import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s15ProximityReader011PaymentCardB0C11isSupportedSbvgZ' >/dev/null || {
        echo "$target probe lacks the public PaymentCardReader.isSupported import" >&2
        exit 1
    }

    build=$(xcrun vtool -show-build "$probe")
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+15\.4([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$target probe does not declare the iOS 15.4 deployment floor" >&2
        exit 1
    }
    printf '%s ProximityReader Release link imports and iOS 15.4 minimum verified; probe not executed\n' "$target"
done

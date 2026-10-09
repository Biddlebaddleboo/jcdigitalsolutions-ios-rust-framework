#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        min_flag=-miphoneos-version-min=17.0
    else
        sdk=iphonesimulator
        min_flag=-mios-simulator-version-min=17.0
    fi
    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    export IPHONEOS_DEPLOYMENT_TARGET=17.0
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo +1.94.1 build --locked --release --target "$target" -p ios-photogrammetry-status \
        --example photogrammetry_status_link_probe

    probe="target/$target/release/examples/photogrammetry_status_link_probe"
    imports=$(otool -L "$probe")
    printf '%s\n' "$imports" | grep -F '/RealityFoundation.framework/RealityFoundation' >/dev/null || {
        echo "$target probe does not import RealityFoundation.framework" >&2
        exit 1
    }
    symbols=$(nm -u "$probe")
    printf '%s\n' "$symbols" | grep -F '_$s17RealityFoundation21PhotogrammetrySessionCMa' >/dev/null || {
        echo "$target probe lacks the public PhotogrammetrySession metadata accessor import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s17RealityFoundation21PhotogrammetrySessionC11isSupportedSbvgZ' >/dev/null || {
        echo "$target probe lacks the public PhotogrammetrySession.isSupported import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s17RealityFoundation21PhotogrammetrySessionC6LimitsVMa' >/dev/null || {
        echo "$target probe lacks the public PhotogrammetrySession.Limits metadata accessor import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s17RealityFoundation21PhotogrammetrySessionC6limitsAC6LimitsVvgZ' >/dev/null || {
        echo "$target probe lacks the public PhotogrammetrySession.limits import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s17RealityFoundation21PhotogrammetrySessionC6LimitsV26maximumInputImageDimensionSivg' >/dev/null || {
        echo "$target probe lacks the public maximumInputImageDimension import" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F '_$s17RealityFoundation21PhotogrammetrySessionC6LimitsV26maximumNumberOfInputImagesSivg' >/dev/null || {
        echo "$target probe lacks the public maximumNumberOfInputImages import" >&2
        exit 1
    }

    build=$(xcrun vtool -show-build "$probe")
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+17\.0([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$target probe does not declare the iOS 17.0 deployment floor" >&2
        exit 1
    }
    printf '%s RealityFoundation Release link imports and iOS 17.0 minimum verified; link example not executed\n' "$target"
done

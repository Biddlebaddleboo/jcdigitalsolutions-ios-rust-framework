#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-ad-attribution-status
link_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-ad-attribution-link.XXXXXX")
trap 'rm -rf "$link_dir"' EXIT HUP INT TERM

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        clang_target=arm64-apple-ios17.4
        expected_platform='platform IOS'
    else
        sdk=iphonesimulator
        clang_target=arm64-apple-ios17.4-simulator
        expected_platform='platform IOSSIMULATOR'
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    library="$link_dir/libad_attribution_status_$target.dylib"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -dynamiclib \
        "$package_dir/native/ad_attribution_status.c" \
        -framework AdAttributionKit \
        -Wl,-install_name,@rpath/libad_attribution_status.dylib \
        -o "$library"

    imports=$(xcrun --sdk "$sdk" otool -L "$library")
    printf '%s\n' "$imports" | grep -F '/System/Library/Frameworks/AdAttributionKit.framework/AdAttributionKit' >/dev/null || {
        echo "$target native link lacks AdAttributionKit.framework" >&2
        exit 1
    }
    symbols=$(xcrun --sdk "$sdk" nm -m "$library")
    printf '%s\n' "$symbols" \
        | grep -F '(undefined) weak external _$s16AdAttributionKit13AppImpressionV11isSupportedSbvgZ (from AdAttributionKit)' >/dev/null || {
        echo "$target linked library lacks the weak AppImpression.isSupported import" >&2
        exit 1
    }
    build=$(xcrun --sdk "$sdk" vtool -show-build "$library")
    printf '%s\n' "$build" | grep -F "$expected_platform" >/dev/null || {
        echo "$target linked library has the wrong Apple platform" >&2
        exit 1
    }
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+17\.4([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$target linked library lacks the iOS 17.4 minimum" >&2
        exit 1
    }
    printf '%s AdAttributionKit framework link/import gate passed; library not executed\n' "$target"
done

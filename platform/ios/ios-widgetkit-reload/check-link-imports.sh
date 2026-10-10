#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-widgetkit-reload
link_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-widgetkit-reload-link.XXXXXX")
trap 'rm -rf "$link_dir"' EXIT HUP INT TERM

for sdk in iphoneos iphonesimulator; do
    if [ "$sdk" = iphoneos ]; then
        suffix=device
        clang_target=arm64-apple-ios14.0
        platform_name=IOS
    else
        suffix=simulator
        clang_target=arm64-apple-ios14.0-simulator
        platform_name=IOSSIMULATOR
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    library="$link_dir/libwidgetkit_reload-$suffix.dylib"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -dynamiclib \
        "$package_dir/native/widgetkit_reload.c" -framework WidgetKit \
        -Wl,-install_name,@rpath/libwidgetkit_reload.dylib -o "$library"

    imports=$(xcrun --sdk "$sdk" otool -L "$library")
    printf '%s\n' "$imports" \
        | grep -F '/System/Library/Frameworks/WidgetKit.framework/WidgetKit' >/dev/null || {
        echo "$suffix native link lacks WidgetKit.framework" >&2
        exit 1
    }
    symbols=$(xcrun --sdk "$sdk" nm -m "$library")
    for symbol in \
        '_$s9WidgetKit0A6CenterCMa' \
        '_$s9WidgetKit0A6CenterC6sharedACvgZ' \
        '_$s9WidgetKit0A6CenterC18reloadAllTimelinesyyFTj' \
        '_$s9WidgetKit0A6CenterC38invalidateConfigurationRecommendationsyyFTj'; do
        printf '%s\n' "$symbols" \
            | grep -F "(undefined) weak external $symbol (from WidgetKit)" >/dev/null || {
            echo "$suffix linked library lacks weak import $symbol" >&2
            exit 1
        }
    done
    build=$(xcrun --sdk "$sdk" vtool -show-build "$library")
    printf '%s\n' "$build" | grep -F "platform $platform_name" >/dev/null || {
        echo "$suffix linked library has the wrong Apple platform" >&2
        exit 1
    }
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+14\.0([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$suffix linked library lacks the iOS 14.0 minimum" >&2
        exit 1
    }
done

printf 'device and Simulator WidgetKit framework link/import gates passed; libraries not executed\n'

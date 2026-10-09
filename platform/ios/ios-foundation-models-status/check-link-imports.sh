#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-foundation-models-status
link_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-foundation-models-link.XXXXXX")
trap 'rm -rf "$link_dir"' EXIT HUP INT TERM

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        clang_target=arm64-apple-ios26.0
        expected_platform='platform IOS'
    else
        sdk=iphonesimulator
        clang_target=arm64-apple-ios26.0-simulator
        expected_platform='platform IOSSIMULATOR'
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    library="$link_dir/libfoundation_models_status_$target.dylib"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -dynamiclib \
        "$package_dir/native/foundation_models_status.c" \
        -framework FoundationModels \
        -Wl,-install_name,@rpath/libfoundation_models_status.dylib \
        -o "$library"

    imports=$(xcrun --sdk "$sdk" otool -L "$library")
    printf '%s\n' "$imports" | grep -F '/System/Library/Frameworks/FoundationModels.framework/FoundationModels' >/dev/null || {
        echo "$target native link lacks FoundationModels.framework" >&2
        exit 1
    }
    symbols=$(xcrun --sdk "$sdk" nm -m "$library")
    for symbol in \
        '_$s16FoundationModels19SystemLanguageModelCMa' \
        '_$s16FoundationModels19SystemLanguageModelC7defaultACvgZ' \
        '_$s16FoundationModels19SystemLanguageModelC11isAvailableSbvg'; do
        printf '%s\n' "$symbols" | grep -F "(undefined) weak external $symbol (from FoundationModels)" >/dev/null || {
            echo "$target linked library lacks weak import $symbol" >&2
            exit 1
        }
    done
    build=$(xcrun --sdk "$sdk" vtool -show-build "$library")
    printf '%s\n' "$build" | grep -F "$expected_platform" >/dev/null || {
        echo "$target linked library has the wrong Apple platform" >&2
        exit 1
    }
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+26\.0([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$target linked library lacks the iOS 26.0 minimum" >&2
        exit 1
    }
    printf '%s Foundation Models framework link/import gate passed; library not executed\n' "$target"
done

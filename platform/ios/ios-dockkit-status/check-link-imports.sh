#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-dockkit-status
link_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-dockkit-status-link.XXXXXX")
trap 'rm -rf "$link_dir"' EXIT HUP INT TERM

sdk=iphoneos
clang_target=arm64-apple-ios17.0
sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
library="$link_dir/libdockkit_status.dylib"
xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
    -std=c11 -Wall -Wextra -Werror -dynamiclib \
    "$package_dir/native/dockkit_status.c" \
    -framework DockKit \
    -Wl,-install_name,@rpath/libdockkit_status.dylib \
    -o "$library"

imports=$(xcrun --sdk "$sdk" otool -L "$library")
printf '%s\n' "$imports" | grep -F '/System/Library/Frameworks/DockKit.framework/DockKit' >/dev/null || {
    echo 'device native link lacks DockKit.framework' >&2
    exit 1
}
symbols=$(xcrun --sdk "$sdk" nm -m "$library")
for symbol in \
    '_$s7DockKit0A16AccessoryManagerCMa' \
    '_$s7DockKit0A16AccessoryManagerC6sharedACvgZ' \
    '_$s7DockKit0A16AccessoryManagerC23isSystemTrackingEnabledSbvgTj'; do
    printf '%s\n' "$symbols" | grep -F "(undefined) weak external $symbol (from DockKit)" >/dev/null || {
        echo "device linked library lacks weak import $symbol" >&2
        exit 1
    }
done
build=$(xcrun --sdk "$sdk" vtool -show-build "$library")
printf '%s\n' "$build" | grep -F 'platform IOS' >/dev/null || {
    echo 'device linked library has the wrong Apple platform' >&2
    exit 1
}
printf '%s\n' "$build" | grep -E 'minos[[:space:]]+17\.0([.]0)?([[:space:]]|$)' >/dev/null || {
    echo 'device linked library lacks the iOS 17.0 minimum' >&2
    exit 1
}
printf 'device DockKit framework link/import gate passed; library not executed\n'

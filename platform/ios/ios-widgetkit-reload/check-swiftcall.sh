#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-widgetkit-reload
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-widgetkit-reload-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

metadata_symbol='$s9WidgetKit0A6CenterCMa'
shared_symbol='$s9WidgetKit0A6CenterC6sharedACvgZ'
reload_symbol='$s9WidgetKit0A6CenterC18reloadAllTimelinesyyFTj'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import WidgetKit
public func reloadAllTimelinesOracle() { WidgetCenter.shared.reloadAllTimelines() }
SWIFT

for sdk in iphoneos iphonesimulator; do
    if [ "$sdk" = iphoneos ]; then
        suffix=device
        swift_target=arm64-apple-ios14.0
        clang_target=arm64-apple-ios14.0
    else
        suffix=simulator
        swift_target=arm64-apple-ios14.0-simulator
        clang_target=arm64-apple-ios14.0-simulator
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    oracle_ir="$oracle_dir/oracle-$suffix.ll"
    thunk_ir="$oracle_dir/thunk-$suffix.ll"
    object="$oracle_dir/thunk-$suffix.o"

    xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
        -module-name WidgetKitReloadOracle -parse-as-library -emit-ir \
        -o "$oracle_ir" "$oracle_dir/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -S -emit-llvm "$package_dir/native/widgetkit_reload.c" \
        -o "$thunk_ir"

    grep -F "call swiftcc %swift.metadata_response @\"$metadata_symbol\"(i64 0)" \
        "$oracle_ir" >/dev/null || {
        echo "$suffix Swift oracle lacks the expected WidgetCenter metadata accessor lowering" >&2
        exit 1
    }
    grep -F "call swiftcc ptr @\"$shared_symbol\"(ptr swiftself" \
        "$oracle_ir" >/dev/null || {
        echo "$suffix Swift oracle lacks the expected owned shared WidgetCenter getter lowering" >&2
        exit 1
    }
    grep -F "call swiftcc void @\"$reload_symbol\"(ptr swiftself" \
        "$oracle_ir" >/dev/null || {
        echo "$suffix Swift oracle lacks the expected all-timeline method lowering" >&2
        exit 1
    }
    grep -F "call void @swift_release(ptr " "$oracle_ir" >/dev/null || {
        echo "$suffix Swift oracle lacks the expected WidgetCenter release" >&2
        exit 1
    }
    grep -F 'call swiftcc { ptr, i64 }' "$thunk_ir" \
        | grep -F "$metadata_symbol" | grep -F '(i64 noundef 0)' >/dev/null || {
        echo "$suffix C thunk does not match the metadata accessor lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc ptr' "$thunk_ir" \
        | grep -F "$shared_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$suffix C thunk does not match the shared WidgetCenter getter lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc void' "$thunk_ir" \
        | grep -F "$reload_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$suffix C thunk does not match the all-timeline method lowering" >&2
        exit 1
    }

    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -c "$package_dir/native/widgetkit_reload.c" \
        -o "$object"
    for symbol in "$metadata_symbol" "$shared_symbol" "$reload_symbol"; do
        xcrun --sdk "$sdk" nm -m "$object" \
            | grep -F "(undefined) weak external _$symbol" >/dev/null || {
            echo "$suffix object lacks weak import $symbol" >&2
            exit 1
        }
    done
done

printf 'device and Simulator WidgetKit swiftcall ABI match the compiler oracle\n'

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

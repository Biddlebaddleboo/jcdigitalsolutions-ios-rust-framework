#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-roomplan
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-roomplan-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

metadata_symbol='$s8RoomPlan0A14CaptureSessionCMa'
getter_symbol='$s8RoomPlan0A14CaptureSessionC11isSupportedSbvgZ'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import RoomPlan
public func roomPlanSupportOracle() -> Bool { RoomCaptureSession.isSupported }
SWIFT

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        swift_target=arm64-apple-ios16.0
        clang_target=arm64-apple-ios16.0
    else
        sdk=iphonesimulator
        swift_target=arm64-apple-ios16.0-simulator
        clang_target=arm64-apple-ios16.0-simulator
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
        -module-name RoomPlanSupportOracle -parse-as-library -emit-ir \
        -o "$oracle_dir/oracle-$target.ll" "$oracle_dir/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -S -emit-llvm "$package_dir/native/roomplan_support.c" \
        -o "$oracle_dir/thunk-$target.ll"

    grep -F "call swiftcc %swift.metadata_response @\"$metadata_symbol\"(i64 0)" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected metadata accessor signature" >&2
        exit 1
    }
    grep -F "call swiftcc i1 @\"$getter_symbol\"(ptr swiftself" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected support getter signature" >&2
        exit 1
    }
    grep -F 'call swiftcc { ptr, i64 }' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$metadata_symbol" | grep -F '(i64 noundef 0)' >/dev/null || {
        echo "$target Clang thunk does not match the metadata accessor lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc i1' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$getter_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$target Clang thunk does not match the support getter context lowering" >&2
        exit 1
    }
    grep -F 'define zeroext i8 @framework_roomplan_is_supported' \
        "$oracle_dir/thunk-$target.ll" >/dev/null || {
        echo "$target Clang thunk lacks its fixed-width C result" >&2
        exit 1
    }
    printf '%s RoomPlan swiftcall ABI matches the compiler oracle\n' "$target"
done

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

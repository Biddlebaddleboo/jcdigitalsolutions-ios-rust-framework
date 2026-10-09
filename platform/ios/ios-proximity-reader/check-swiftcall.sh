#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-proximity-reader
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-proximity-reader-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

metadata_symbol='$s15ProximityReader011PaymentCardB0CMa'
getter_symbol='$s15ProximityReader011PaymentCardB0C11isSupportedSbvgZ'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import ProximityReader
public func proximityReaderSupportOracle() -> Bool { PaymentCardReader.isSupported }
SWIFT

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        swift_target=arm64-apple-ios15.4
        clang_target=arm64-apple-ios15.4
    else
        sdk=iphonesimulator
        swift_target=arm64-apple-ios15.4-simulator
        clang_target=arm64-apple-ios15.4-simulator
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
        -module-name ProximityReaderSupportOracle -parse-as-library -emit-ir \
        -o "$oracle_dir/oracle-$target.ll" "$oracle_dir/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -S -emit-llvm "$package_dir/native/proximity_reader_support.c" \
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
    grep -F 'define zeroext i8 @framework_proximity_reader_tap_to_pay_device_model_supported' \
        "$oracle_dir/thunk-$target.ll" >/dev/null || {
        echo "$target Clang thunk lacks its fixed-width C result" >&2
        exit 1
    }
    printf '%s ProximityReader swiftcall ABI matches the compiler oracle\n' "$target"
done

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

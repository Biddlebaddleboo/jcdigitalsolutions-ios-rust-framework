#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-foundation-models-status
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-foundation-models-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

metadata_symbol='$s16FoundationModels19SystemLanguageModelCMa'
default_symbol='$s16FoundationModels19SystemLanguageModelC7defaultACvgZ'
available_symbol='$s16FoundationModels19SystemLanguageModelC11isAvailableSbvg'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import FoundationModels
public func foundationModelsAvailabilityOracle() -> Bool { SystemLanguageModel.default.isAvailable }
SWIFT

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        swift_target=arm64-apple-ios26.0
        clang_target=arm64-apple-ios26.0
    else
        sdk=iphonesimulator
        swift_target=arm64-apple-ios26.0-simulator
        clang_target=arm64-apple-ios26.0-simulator
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
        -module-name FoundationModelsOracle -parse-as-library -emit-ir \
        -o "$oracle_dir/oracle-$target.ll" "$oracle_dir/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -S -emit-llvm "$package_dir/native/foundation_models_status.c" \
        -o "$oracle_dir/thunk-$target.ll"

    grep -F "call swiftcc %swift.metadata_response @\"$metadata_symbol\"(i64 0)" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected metadata accessor lowering" >&2
        exit 1
    }
    grep -F "call swiftcc ptr @\"$default_symbol\"(ptr swiftself" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected owned default getter lowering" >&2
        exit 1
    }
    grep -F "call void @swift_release(ptr " "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected default-model release" >&2
        exit 1
    }
    grep -F "call swiftcc i1 @\"$available_symbol\"(ptr swiftself" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected availability getter lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc { ptr, i64 }' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$metadata_symbol" | grep -F '(i64 noundef 0)' >/dev/null || {
        echo "$target C thunk does not match the metadata accessor lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc ptr' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$default_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$target C thunk does not match the default getter context lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc i1' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$available_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$target C thunk does not match the availability getter lowering" >&2
        exit 1
    }

    object="$oracle_dir/thunk-$target.o"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -c "$package_dir/native/foundation_models_status.c" \
        -o "$object"
    for symbol in "$metadata_symbol" "$default_symbol" "$available_symbol"; do
        xcrun --sdk "$sdk" nm -m "$object" \
            | grep -F "(undefined) weak external _$symbol" >/dev/null || {
            echo "$target object lacks weak import $symbol" >&2
            exit 1
        }
    done
    printf '%s Foundation Models swiftcall ABI matches the compiler oracle\n' "$target"
done

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

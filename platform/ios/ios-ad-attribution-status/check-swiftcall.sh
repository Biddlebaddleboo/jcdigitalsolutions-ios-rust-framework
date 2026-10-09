#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-ad-attribution-status
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-ad-attribution-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

getter_symbol='$s16AdAttributionKit13AppImpressionV11isSupportedSbvgZ'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import AdAttributionKit
public func appImpressionSupportOracle() -> Bool { AppImpression.isSupported }
SWIFT

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        swift_target=arm64-apple-ios18.0
        clang_target=arm64-apple-ios17.4
    else
        sdk=iphonesimulator
        swift_target=arm64-apple-ios18.0-simulator
        clang_target=arm64-apple-ios17.4-simulator
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
        -module-name AdAttributionKitOracle -parse-as-library -emit-ir \
        -o "$oracle_dir/oracle-$target.ll" "$oracle_dir/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -S -emit-llvm "$package_dir/native/ad_attribution_status.c" \
        -o "$oracle_dir/thunk-$target.ll"

    grep -F "call swiftcc i1 @\"$getter_symbol\"()" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected static getter signature" >&2
        exit 1
    }
    grep -F 'call swiftcc i1' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$getter_symbol" | grep -F '()' >/dev/null || {
        echo "$target C thunk does not match the static getter lowering" >&2
        exit 1
    }
    grep -F 'declare extern_weak swiftcc i1' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$getter_symbol" >/dev/null || {
        echo "$target C thunk does not weak-import the support getter" >&2
        exit 1
    }

    object="$oracle_dir/thunk-$target.o"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -c "$package_dir/native/ad_attribution_status.c" \
        -o "$object"
    xcrun --sdk "$sdk" nm -m "$object" \
        | grep -F "(undefined) weak external _$getter_symbol" >/dev/null || {
        echo "$target object lacks the exact weak import" >&2
        exit 1
    }
    printf '%s AdAttributionKit swiftcall ABI matches the compiler oracle\n' "$target"
done

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-dockkit-status
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-dockkit-status-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

metadata_symbol='$s7DockKit0A16AccessoryManagerCMa'
shared_symbol='$s7DockKit0A16AccessoryManagerC6sharedACvgZ'
enabled_symbol='$s7DockKit0A16AccessoryManagerC23isSystemTrackingEnabledSbvgTj'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import DockKit
public func dockKitTrackingSettingOracle() -> Bool { DockAccessoryManager.shared.isSystemTrackingEnabled }
SWIFT

sdk=iphoneos
swift_target=arm64-apple-ios17.0
clang_target=arm64-apple-ios17.0
sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
    -module-name DockKitStatusOracle -parse-as-library -emit-ir \
    -o "$oracle_dir/oracle-device.ll" "$oracle_dir/Oracle.swift"
xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
    -std=c11 -S -emit-llvm "$package_dir/native/dockkit_status.c" \
    -o "$oracle_dir/thunk-device.ll"

grep -F "call swiftcc %swift.metadata_response @\"$metadata_symbol\"(i64 0)" \
    "$oracle_dir/oracle-device.ll" >/dev/null || {
    echo 'device Swift oracle lacks the expected manager metadata accessor lowering' >&2
    exit 1
}
grep -F "call swiftcc ptr @\"$shared_symbol\"(ptr swiftself" \
    "$oracle_dir/oracle-device.ll" >/dev/null || {
    echo 'device Swift oracle lacks the expected owned shared-manager getter lowering' >&2
    exit 1
}
grep -F "call void @swift_release(ptr " "$oracle_dir/oracle-device.ll" >/dev/null || {
    echo 'device Swift oracle lacks the expected manager release' >&2
    exit 1
}
grep -F "call swiftcc i1 @\"$enabled_symbol\"(ptr swiftself" \
    "$oracle_dir/oracle-device.ll" >/dev/null || {
    echo 'device Swift oracle lacks the expected setting getter lowering' >&2
    exit 1
}
grep -F 'call swiftcc { ptr, i64 }' "$oracle_dir/thunk-device.ll" \
    | grep -F "$metadata_symbol" | grep -F '(i64 noundef 0)' >/dev/null || {
    echo 'device C thunk does not match the metadata accessor lowering' >&2
    exit 1
}
grep -F 'call swiftcc ptr' "$oracle_dir/thunk-device.ll" \
    | grep -F "$shared_symbol" | grep -F 'swiftself' >/dev/null || {
    echo 'device C thunk does not match the shared-manager getter lowering' >&2
    exit 1
}
grep -F 'call swiftcc i1' "$oracle_dir/thunk-device.ll" \
    | grep -F "$enabled_symbol" | grep -F 'swiftself' >/dev/null || {
    echo 'device C thunk does not match the setting getter lowering' >&2
    exit 1
}

object="$oracle_dir/thunk-device.o"
xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
    -std=c11 -Wall -Wextra -Werror -c "$package_dir/native/dockkit_status.c" \
    -o "$object"
for symbol in "$metadata_symbol" "$shared_symbol" "$enabled_symbol"; do
    xcrun --sdk "$sdk" nm -m "$object" \
        | grep -F "(undefined) weak external _$symbol" >/dev/null || {
        echo "device object lacks weak import $symbol" >&2
        exit 1
    }
done
printf 'device DockKit swiftcall ABI matches the compiler oracle\n'

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

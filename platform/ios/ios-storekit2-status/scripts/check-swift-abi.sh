#!/bin/sh
set -eu

proof_dir="$(mktemp -d "${TMPDIR:-/tmp}/ios-storekit2-status.XXXXXX")"
trap 'rm -rf "$proof_dir"' EXIT HUP INT TERM

cat > "$proof_dir/Oracle.swift" <<'SWIFT'
import StoreKit
public func oracleAppStoreCanMakePayments() -> Bool {
    AppStore.canMakePayments
}
SWIFT

device_sdk="$(xcrun --sdk iphoneos --show-sdk-path)"
simulator_sdk="$(xcrun --sdk iphonesimulator --show-sdk-path)"
bridge_file=platform/ios/ios-storekit2-status/src/storekit2_bridge.c
swift_symbol='_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ'

xcrun swiftc -parse-as-library -module-name StoreKit2StatusOracle \
    -target arm64-apple-ios15.0 -sdk "$device_sdk" -emit-ir \
    -o "$proof_dir/device.ll" "$proof_dir/Oracle.swift"
xcrun swiftc -parse-as-library -module-name StoreKit2StatusOracle \
    -target arm64-apple-ios15.0-simulator -sdk "$simulator_sdk" -emit-ir \
    -o "$proof_dir/simulator.ll" "$proof_dir/Oracle.swift"
xcrun --sdk iphoneos clang -target arm64-apple-ios15.0 -isysroot "$device_sdk" \
    -S -emit-llvm -o "$proof_dir/device-bridge.ll" "$bridge_file"
xcrun --sdk iphonesimulator clang -target arm64-apple-ios15.0-simulator -isysroot "$simulator_sdk" \
    -S -emit-llvm -o "$proof_dir/simulator-bridge.ll" "$bridge_file"

for target_dir in device simulator; do
    if ! rg -F 'call swiftcc i1 @"$s8StoreKit03AppA0O15canMakePaymentsSbvgZ"()' "$proof_dir/$target_dir.ll"; then
        echo "$target_dir Swift oracle lacks the swiftcall Bool property call" >&2
        exit 1
    fi
    if ! rg -F 'call swiftcc i1 @"\01_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ"()' "$proof_dir/$target_dir-bridge.ll"; then
        echo "$target_dir Clang C IR does not match the Swift Bool call ABI" >&2
        exit 1
    fi
done

for sdk_dir in "$device_sdk" "$simulator_sdk"; do
    tbd="$sdk_dir/System/Library/Frameworks/StoreKit.framework/StoreKit.tbd"
    if ! rg -F "$swift_symbol" "$tbd"; then
        echo "StoreKit SDK lacks the expected public symbol in $tbd" >&2
        exit 1
    fi
done

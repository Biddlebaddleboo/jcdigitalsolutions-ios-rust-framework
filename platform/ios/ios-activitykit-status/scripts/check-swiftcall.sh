#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")/../../.." rev-parse --show-toplevel)"
cd "$repo_root"

scratch="$(mktemp -d "${TMPDIR:-/tmp}/ios-activitykit-status-swiftcall.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

cat > "$scratch/Oracle.swift" <<'SWIFT'
import ActivityKit
public func activityKitEligibilityOracle() -> Bool { ActivityAuthorizationInfo().areActivitiesEnabled }
SWIFT

metadata_symbol='$s11ActivityKit0A17AuthorizationInfoCMa'
init_symbol='$s11ActivityKit0A17AuthorizationInfoCACycfC'
getter_symbol='$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg'
bridge=platform/ios/ios-activitykit-status/native/activitykit_status.c

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        swift_target=arm64-apple-ios16.1
        clang_target=arm64-apple-ios16.1
    else
        sdk=iphonesimulator
        swift_target=arm64-apple-ios16.1-simulator
        clang_target=arm64-apple-ios16.1-simulator
    fi
    sdk_path="$(xcrun --sdk "$sdk" --show-sdk-path)"
    swift_ir="$scratch/$target.swift.ll"
    clang_ir="$scratch/$target.bridge.ll"

    xcrun --sdk "$sdk" swiftc -parse-as-library -module-name ActivityKitStatusOracle \
        -target "$swift_target" -sdk "$sdk_path" -emit-ir -o "$swift_ir" "$scratch/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -std=c11 -Wall -Wextra -Werror -S -emit-llvm -o "$clang_ir" "$bridge"

    grep -Fq "call swiftcc %swift.metadata_response @\"$metadata_symbol\"(i64 0)" "$swift_ir"
    grep -Fq "call swiftcc ptr @\"$init_symbol\"(ptr swiftself" "$swift_ir"
    grep -Fq "call swiftcc i1 @\"$getter_symbol\"(ptr swiftself" "$swift_ir"
    grep -Fq 'call void @swift_release(ptr ' "$swift_ir"
    grep -Fq "call swiftcc { ptr, i64 } @\"\\01_$metadata_symbol\"(i64 noundef 0)" "$clang_ir"
    grep -Fq "call swiftcc ptr @\"\\01_$init_symbol\"(ptr noundef swiftself" "$clang_ir"
    grep -Fq "call swiftcc i1 @\"\\01_$getter_symbol\"(ptr noundef swiftself" "$clang_ir"
    grep -Fq 'extern_weak' "$clang_ir"
    grep -Fq 'framework_activity_authorization_info_create' "$bridge"
    grep -Fq 'framework_activity_authorization_info_are_activities_enabled' "$bridge"
    printf '%s ActivityKit Swift-call ABI matches the compiler oracle\n' "$target"
done

if find platform/ios/ios-activitykit-status -name '*.swift' -print -quit | grep -q .; then
    echo 'Swift source found in the ActivityKit Rust package' >&2
    exit 1
fi

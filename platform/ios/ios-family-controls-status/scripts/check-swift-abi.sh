#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

scratch="$(mktemp -d "${TMPDIR:-/tmp}/family-controls-swift-abi.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

cat > "$scratch/Oracle.swift" <<'SWIFT'
import FamilyControls
@_cdecl("family_controls_status_oracle") public func familyControlsStatusOracle() -> Int { AuthorizationCenter.shared.authorizationStatus.rawValue }
SWIFT

bridge=platform/ios/ios-family-controls-status/src/family_controls_status_bridge.c
for target in arm64-apple-ios15.0 arm64-apple-ios15.0-simulator; do
    if [ "$target" = arm64-apple-ios15.0 ]; then
        sdk=iphoneos
    else
        sdk=iphonesimulator
    fi
    sdk_path="$(xcrun --sdk "$sdk" --show-sdk-path)"
    swift_ir="$scratch/$target.swift.ll"
    clang_ir="$scratch/$target.bridge.ll"

    xcrun --sdk "$sdk" swiftc -parse-as-library -module-name FamilyControlsStatusOracle \
        -target "$target" -sdk "$sdk_path" -emit-ir -o "$swift_ir" "$scratch/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$target" -isysroot "$sdk_path" -std=c11 \
        -Wall -Wextra -Werror -S -emit-llvm "$bridge" -o "$clang_ir"

    grep -Fq '%swift.vwtable = type { ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, i64, i64, i32, i32 }' "$swift_ir"
    grep -Fq 'getelementptr inbounds ptr, ptr %1, i64 -1' "$swift_ir"
    grep -Fq '%struct.SwiftValueWitnessTable = type { [8 x ptr], i64, i64, i32, i32 }' "$clang_ir"
    grep -Fq 'getelementptr inbounds ptr, ptr' "$clang_ir"
    grep -Fq 'i64 -1' "$clang_ir"
    grep -Eq 'call swiftcc .*AuthorizationStatusOMa.*i64 0' "$swift_ir"
    grep -Eq 'call swiftcc .*AuthorizationCenterCMa.*i64 0' "$swift_ir"
    grep -Eq 'call swiftcc \{ ptr, i64 \} .*AuthorizationStatusOMa.*i64 noundef 0' "$clang_ir"
    grep -Eq 'call swiftcc \{ ptr, i64 \} .*AuthorizationCenterCMa.*i64 noundef 0' "$clang_ir"

    for ir in "$swift_ir" "$clang_ir"; do
        grep -Eq 'call swiftcc ptr .*AuthorizationCenterC6sharedACvgZ.*swiftself' "$ir"
        grep -Eq 'call swiftcc void .*AuthorizationCenterC19authorizationStatusAA0cF0OvgTj.*sret.*swiftself' "$ir"
        grep -Eq 'call swiftcc i64 .*AuthorizationStatusO8rawValueSivg.*swiftself' "$ir"
        grep -Eq 'call void @swift_release\(ptr' "$ir"
        grep -Eq 'call void %[^ (]+\(ptr .*ptr' "$ir"
    done

    grep -Fq 'flags & UINT32_C(0xff)' "$bridge"
    grep -Eq 'and i32 .* 255' "$clang_ir"
    grep -Eq 'getelementptr inbounds ptr, ptr .*i32 1' "$swift_ir"
    grep -Eq 'call void %[^ (]+\(ptr noalias .*ptr' "$swift_ir"
    grep -Fq 'vwt->size' "$bridge"
    grep -Fq 'vwt->witnesses[1]' "$bridge"
    grep -Fq '*raw_value = value' "$bridge"
    printf '%s Family Controls Swift ABI matches the compiler oracle\n' "$target"
done

if find platform/ios/ios-family-controls-status -name '*.swift' -print -quit | grep -q .; then
    echo 'Swift source found in the Family Controls Rust package' >&2
    exit 1
fi

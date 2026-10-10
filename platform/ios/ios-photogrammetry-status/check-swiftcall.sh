#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
package_dir=$repo_root/platform/ios/ios-photogrammetry-status
oracle_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-photogrammetry-swiftcall.XXXXXX")
trap 'rm -rf "$oracle_dir"' EXIT HUP INT TERM

metadata_symbol='$s17RealityFoundation21PhotogrammetrySessionCMa'
getter_symbol='$s17RealityFoundation21PhotogrammetrySessionC11isSupportedSbvgZ'
limits_metadata_symbol='$s17RealityFoundation21PhotogrammetrySessionC6LimitsVMa'
limits_getter_symbol='$s17RealityFoundation21PhotogrammetrySessionC6limitsAC6LimitsVvgZ'
maximum_dimension_symbol='$s17RealityFoundation21PhotogrammetrySessionC6LimitsV26maximumInputImageDimensionSivg'
maximum_images_symbol='$s17RealityFoundation21PhotogrammetrySessionC6LimitsV26maximumNumberOfInputImagesSivg'
cat > "$oracle_dir/Oracle.swift" <<'SWIFT'
import RealityKit
public func photogrammetrySupportOracle() -> Bool { PhotogrammetrySession.isSupported }
public func photogrammetryLimitsOracle() -> (Int, Int) { let limits = PhotogrammetrySession.limits; return (limits.maximumInputImageDimension, limits.maximumNumberOfInputImages) }
SWIFT

for target in device simulator; do
    if [ "$target" = device ]; then
        sdk=iphoneos
        swift_target=arm64-apple-ios17.0
        clang_target=arm64-apple-ios17.0
    else
        sdk=iphonesimulator
        swift_target=arm64-apple-ios17.0-simulator
        clang_target=arm64-apple-ios17.0-simulator
    fi
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    xcrun --sdk "$sdk" swiftc -target "$swift_target" -sdk "$sdk_path" \
        -module-name PhotogrammetrySupportOracle -parse-as-library -emit-ir \
        -o "$oracle_dir/oracle-$target.ll" "$oracle_dir/Oracle.swift"
    xcrun --sdk "$sdk" clang -target "$clang_target" -isysroot "$sdk_path" \
        -I "$repo_root/interop/swift-abi-core/include" \
        -std=c11 -S -emit-llvm "$package_dir/native/photogrammetry_status.c" \
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
    grep -F "call swiftcc %swift.metadata_response @\"$limits_metadata_symbol\"(i64 0)" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected Limits metadata accessor signature" >&2
        exit 1
    }
    grep -F "call swiftcc void @\"$limits_getter_symbol\"(ptr noalias sret(%swift.opaque)" \
        "$oracle_dir/oracle-$target.ll" | grep -F 'ptr swiftself' >/dev/null || {
        echo "$target Swift oracle lacks the expected opaque Limits result signature" >&2
        exit 1
    }
    grep -F "call swiftcc i64 @\"$maximum_dimension_symbol\"(ptr noalias swiftself" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected maximum image dimension getter signature" >&2
        exit 1
    }
    grep -F "call swiftcc i64 @\"$maximum_images_symbol\"(ptr noalias swiftself" \
        "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks the expected maximum input image count getter signature" >&2
        exit 1
    }
    grep -F 'getelementptr inbounds nuw %swift.vwtable' "$oracle_dir/oracle-$target.ll" >/dev/null &&
        grep -F '%size = load i64' "$oracle_dir/oracle-$target.ll" >/dev/null &&
        grep -F 'alloca i8, i64 %size' "$oracle_dir/oracle-$target.ll" >/dev/null &&
        grep -F 'call void %Destroy(ptr noalias' "$oracle_dir/oracle-$target.ll" >/dev/null || {
        echo "$target Swift oracle lacks metadata-sized opaque storage and value-witness destruction" >&2
        exit 1
    }
    grep -F 'call swiftcc { ptr, i64 }' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$metadata_symbol" | grep -F '(i64 noundef 0)' >/dev/null || {
        echo "$target C thunk does not match the metadata accessor lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc i1' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$getter_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$target C thunk does not match the support getter context lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc { ptr, i64 }' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$limits_metadata_symbol" | grep -F '(i64 noundef 0)' >/dev/null || {
        echo "$target C thunk does not match the Limits metadata accessor lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc void' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$limits_getter_symbol" | grep -F 'sret(ptr)' | grep -F 'swiftself' >/dev/null || {
        echo "$target C thunk does not match the opaque Limits getter lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc i64' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$maximum_dimension_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$target C thunk does not match the maximum image dimension getter lowering" >&2
        exit 1
    }
    grep -F 'call swiftcc i64' "$oracle_dir/thunk-$target.ll" \
        | grep -F "$maximum_images_symbol" | grep -F 'swiftself' >/dev/null || {
        echo "$target C thunk does not match the maximum input image count getter lowering" >&2
        exit 1
    }
    grep -F 'getelementptr inbounds ptr' "$oracle_dir/thunk-$target.ll" \
        | grep -F 'i64 -1' >/dev/null &&
        grep -F 'posix_memalign' "$oracle_dir/thunk-$target.ll" >/dev/null &&
        grep -F 'call void %' "$oracle_dir/thunk-$target.ll" \
        | grep -F '(ptr' | grep -F 'ptr' >/dev/null || {
        echo "$target C thunk does not use value-witness metadata storage and destruction" >&2
        exit 1
    }
    grep -F 'define zeroext i8 @framework_photogrammetry_session_is_supported' \
        "$oracle_dir/thunk-$target.ll" >/dev/null || {
        echo "$target C thunk lacks its fixed-width C result" >&2
        exit 1
    }
    grep -F 'define zeroext i8 @framework_photogrammetry_session_limits' \
        "$oracle_dir/thunk-$target.ll" >/dev/null || {
        echo "$target C limits thunk lacks its fixed-width C result" >&2
        exit 1
    }
    printf '%s RealityFoundation swiftcall ABI matches the compiler oracles\n' "$target"
done

if rg --files "$package_dir" -g '*.swift' | rg . >/dev/null; then
    echo 'repository package must not contain Swift source' >&2
    exit 1
fi

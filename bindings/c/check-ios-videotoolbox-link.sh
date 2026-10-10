#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F25 device/Simulator link-import gate" >&2
        exit 1
    fi
done

sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for this audit: $llvm_nm" >&2
    exit 1
fi

sh bindings/c/check-ios-videotoolbox.sh

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-videotoolbox-default-tree.txt
if rg -q '(^|[[:space:]])(framework-media|ios-media) v|objc2-video-toolbox v' \
    target/framework-c-ios-videotoolbox-default-tree.txt; then
    echo "VideoToolbox dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-videotoolbox \
    > target/framework-c-ios-videotoolbox-ios-tree.txt
rg -q 'framework-media v' target/framework-c-ios-videotoolbox-ios-tree.txt
rg -q 'ios-media v' target/framework-c-ios-videotoolbox-ios-tree.txt
rg -q 'objc2-video-toolbox feature "VTDecompressionSession"' \
    target/framework-c-ios-videotoolbox-ios-tree.txt
if rg -q 'objc2-video-toolbox feature "default"' \
    target/framework-c-ios-videotoolbox-ios-tree.txt; then
    echo "objc2-video-toolbox default features leaked into F25" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-videotoolbox \
    > target/framework-c-ios-videotoolbox-host-tree.txt
if rg -q '(^|[[:space:]])ios-media v|objc2-video-toolbox v|VideoToolbox.framework' \
    target/framework-c-ios-videotoolbox-host-tree.txt; then
    echo "iOS VideoToolbox dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-videotoolbox
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-videotoolbox -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-videotoolbox

cat > target/framework-c-ios-videotoolbox-c.c <<'FIXTURE_C'
#include <framework_ios_videotoolbox.h>
int main(void) {
    uint8_t supported = 0;
    return (int)framework_ios_videotoolbox_hardware_decode_supported(UINT32_C(0x61766331), &supported);
}
FIXTURE_C
cat > target/framework-c-ios-videotoolbox-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_videotoolbox.h>
int main() {
    uint8_t supported = 0;
    return static_cast<int>(framework_ios_videotoolbox_hardware_decode_supported(UINT32_C(0x61766331), &supported));
}
FIXTURE_CPP

expected_symbols=target/framework-c-ios-videotoolbox-expected-symbols.txt
jq -r '.optional_capabilities.ios_videotoolbox.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-videotoolbox-c.c "$host_archive" \
    -o target/framework-c-ios-videotoolbox-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-videotoolbox-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-videotoolbox-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_videotoolbox_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-videotoolbox-host-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-videotoolbox-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-videotoolbox-host-undefined.txt
if rg -q 'VTIsHardwareDecodeSupported|VideoToolbox|CoreMedia|AVAudioSession|AVFAudio|objc_msgSend|OBJC_CLASS|swift_' \
    target/framework-c-ios-videotoolbox-host-undefined.txt; then
    echo "Apple framework, Objective-C, or Swift import leaked into the host archive" >&2
    exit 1
fi
for language in c cpp; do
    case "$language" in
        c) binary=target/framework-c-ios-videotoolbox-c-host; expected='libSystem.B.dylib' ;;
        cpp) binary=target/framework-c-ios-videotoolbox-cpp-host; expected=$(printf '%s\n' 'libc++.1.dylib' 'libSystem.B.dylib') ;;
    esac
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "target/framework-c-ios-videotoolbox-$language-host-libraries.txt"
    printf '%s\n' "$expected" | LC_ALL=C sort \
        > "target/framework-c-ios-videotoolbox-$language-host-libraries-expected.txt"
    diff -u "target/framework-c-ios-videotoolbox-$language-host-libraries-expected.txt" \
        "target/framework-c-ios-videotoolbox-$language-host-libraries.txt"
    nm -u "$binary" 2>/dev/null > "target/framework-c-ios-videotoolbox-$language-host-undefined.txt"
    if rg -q 'VTIsHardwareDecodeSupported|VideoToolbox|CoreMedia|AVAudioSession|AVFAudio|objc_msgSend|OBJC_CLASS|swift_' \
        "target/framework-c-ios-videotoolbox-$language-host-undefined.txt"; then
        echo "Apple framework, Objective-C, or Swift import leaked into host $language consumer" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=11.0
            sdk=iphoneos
            clang_target=arm64-apple-ios11.0
            rust_min_flag=-miphoneos-version-min=11.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            rust_min_flag=-mios-simulator-version-min=14.0
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-videotoolbox \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-videotoolbox \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-videotoolbox \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-videotoolbox-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-videotoolbox-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-videotoolbox-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework VideoToolbox -Wl,-dead_strip_dylibs \
            -o "$binary"
        libraries="target/framework-c-ios-videotoolbox-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_videotoolbox.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-videotoolbox-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-videotoolbox-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-videotoolbox-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_VTIsHardwareDecodeSupported' "$symbols"
        if rg -q 'VTCompressionSession|VTDecompressionSession|VTDecompressionSessionDecodeFrame|VTCompressionSessionEncodeFrame|AVAudioSession|AVFAudio|swift_|objc_msgSend' \
            "$symbols"; then
            echo "out-of-scope VideoToolbox, audio, Objective-C messaging, or Swift import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-videotoolbox-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s VideoToolbox imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_videotoolbox_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-videotoolbox-$target-archive-symbols.txt"
    diff -u "$expected_symbols" "target/framework-c-ios-videotoolbox-$target-archive-symbols.txt"
done

printf 'F25 device/Simulator link-import gate complete; no tests or consumer/probe binaries were executed\n'

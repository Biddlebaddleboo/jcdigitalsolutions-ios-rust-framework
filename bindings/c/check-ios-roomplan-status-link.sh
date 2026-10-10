#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F31 link/import gate" >&2
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

sh bindings/c/check-ios-roomplan-status.sh

cat > target/framework-c-ios-roomplan-status-link.c <<'FIXTURE_C'
#include "framework_ios_roomplan_status.h"

int main(void) {
    uint8_t supported = 0;
    return (int) framework_ios_roomplan_status_is_supported(&supported);
}
FIXTURE_C
cat > target/framework-c-ios-roomplan-status-link.cpp <<'FIXTURE_CPP'
#include "framework_ios_roomplan_status.h"

int main() {
    uint8_t supported = 0;
    return static_cast<int>(framework_ios_roomplan_status_is_supported(&supported));
}
FIXTURE_CPP

expected_symbols=target/framework-c-ios-roomplan-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_roomplan_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"

cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-roomplan-status
host_archive=target/release/libframework_c_api.a
for language in c cpp; do
    case "$language" in
        c) compiler=clang; source=target/framework-c-ios-roomplan-status-link.c; standard=c11 ;;
        cpp) compiler=clang++; source=target/framework-c-ios-roomplan-status-link.cpp; standard=c++17 ;;
    esac
    binary="target/framework-c-ios-roomplan-status-$language-host"
    "$compiler" -std="$standard" -Wall -Wextra -Werror -pedantic -nostdlib++ \
        -I bindings/c/include "$source" "$host_archive" -o "$binary"
    symbols="target/framework-c-ios-roomplan-status-$language-host-undefined.txt"
    nm -u "$binary" 2>/dev/null > "$symbols"
    if rg -q 'RoomPlan|RoomCaptureSession|swift_|libswift|objc_|OBJC_CLASS' "$symbols"; then
        echo "RoomPlan, Swift, or Objective-C imports leaked into host $language consumer" >&2
        exit 1
    fi
    libraries="target/framework-c-ios-roomplan-status-$language-host-libraries.txt"
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "$libraries"
    printf '%s\n' 'libSystem.B.dylib' > "$libraries.expected"
    diff -u "$libraries.expected" "$libraries"
done
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_roomplan_status_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-roomplan-status-host-archive-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-roomplan-status-host-archive-symbols.txt

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=16.0
            sdk=iphoneos
            clang_target=arm64-apple-ios16.0
            rust_min_flag=-miphoneos-version-min=16.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=16.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios16.0-simulator
            rust_min_flag=-mios-simulator-version-min=16.0
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-roomplan-status --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-roomplan-status \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-roomplan-status --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_roomplan_status_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-roomplan-status-$target-archive-symbols.txt"
    diff -u "$expected_symbols" "target/framework-c-ios-roomplan-status-$target-archive-symbols.txt"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-roomplan-status-link.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-roomplan-status-link.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-roomplan-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -nostdlib++ -I bindings/c/include \
            "$source" "$archive" -framework RoomPlan -Wl,-dead_strip_dylibs -o "$binary"
        imports="target/framework-c-ios-roomplan-status-$language-$target-imports.txt"
        otool -L "$binary" > "$imports"
        libraries="target/framework-c-ios-roomplan-status-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_roomplan_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort > "$libraries.expected"
        diff -u "$libraries.expected" "$libraries"
        symbols="target/framework-c-ios-roomplan-status-$language-$target-undefined.txt"
        nm -m "$binary" 2>/dev/null > "$symbols"
        rg -Fq '_$s8RoomPlan0A14CaptureSessionCMa' "$symbols"
        rg -Fq '_$s8RoomPlan0A14CaptureSessionC11isSupportedSbvgZ' "$symbols"
        if rg -q 'Swift\.framework|libswift|swift_(retain|release|alloc|dealloc|beginAccess|endAccess)|libobjc|objc_msgSend|OBJC_CLASS' \
            "$imports" "$symbols"; then
            echo "Swift or Objective-C runtime import leaked into $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-roomplan-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports RoomPlan and libSystem; minos %s; consumer not executed\n' \
            "$target" "$language" "$actual_deployment_target"
    done
done

printf 'F31 link/import gate complete; no tests, consumers, probes, or RoomPlan calls were executed\n'

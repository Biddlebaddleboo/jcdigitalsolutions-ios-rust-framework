#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

rustc_sysroot=$(rustc --print sysroot)
rustc_host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$rustc_sysroot/lib/rustlib/$rustc_host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for Rust archive scans: $llvm_nm" >&2
    exit 1
fi

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F27 link-import gate" >&2
        exit 1
    fi
done

sh bindings/c/check-ios-core-ml-status.sh

cat > target/framework-c-ios-core-ml-status-c.c <<'FIXTURE_C'
#include <framework_ios_core_ml_status.h>
int main(void) {
    uint8_t available = 0;
    return (int)framework_ios_core_ml_status_has_available_compute_device(&available);
}
FIXTURE_C
cat > target/framework-c-ios-core-ml-status-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_core_ml_status.h>
int main() {
    uint8_t available = 0;
    return static_cast<int>(framework_ios_core_ml_status_has_available_compute_device(&available));
}
FIXTURE_CPP

expected_symbols=target/framework-c-ios-core-ml-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_core_ml_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"

cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-core-ml-status
host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-core-ml-status-c.c "$host_archive" \
    -o target/framework-c-ios-core-ml-status-c-host
clang++ -nostdlib++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-core-ml-status-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-core-ml-status-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_core_ml_status_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-core-ml-status-host-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-core-ml-status-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null > target/framework-c-ios-core-ml-status-host-undefined.txt
if rg -q 'MLModel|availableComputeDevices|CoreML|objc_|OBJC_CLASS|swift_' \
    target/framework-c-ios-core-ml-status-host-undefined.txt; then
    echo "Core ML, Objective-C, or Swift import leaked into the host archive" >&2
    exit 1
fi

for language in c cpp; do
    case "$language" in
        c) binary=target/framework-c-ios-core-ml-status-c-host; expected='libSystem.B.dylib' ;;
        cpp) binary=target/framework-c-ios-core-ml-status-cpp-host; expected='libSystem.B.dylib' ;;
    esac
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "target/framework-c-ios-core-ml-status-$language-host-libraries.txt"
    printf '%s\n' "$expected" | LC_ALL=C sort \
        > "target/framework-c-ios-core-ml-status-$language-host-libraries-expected.txt"
    diff -u "target/framework-c-ios-core-ml-status-$language-host-libraries-expected.txt" \
        "target/framework-c-ios-core-ml-status-$language-host-libraries.txt"
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
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-core-ml-status \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-core-ml-status-c.c; standard=c11; cxx_flag= ;;
            cpp) compiler=clang++; source=target/framework-c-ios-core-ml-status-cpp.cpp; standard=c++17; cxx_flag=-nostdlib++ ;;
        esac
        binary="target/framework-c-ios-core-ml-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" $cxx_flag \
            -framework CoreML -framework Foundation -lobjc -o "$binary"
        libraries="target/framework-c-ios-core-ml-status-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_core_ml_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-core-ml-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-core-ml-status-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-core-ml-status-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        strings_file="target/framework-c-ios-core-ml-status-$language-$target-strings.txt"
        strings "$binary" > "$strings_file"
        rg -q 'MLModel' "$strings_file"
        rg -q 'availableComputeDevices' "$strings_file"
        if rg -qi 'MLModel::(modelWith|load|prediction|predictions)|MLModelConfiguration|MLFeatureProvider|swift_' \
            "$symbols" "$strings_file"; then
            echo "model load/inference or Swift behavior leaked into $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-core-ml-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_core_ml_status_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-core-ml-status-$target-archive-symbols.txt"
    diff -u "$expected_symbols" "target/framework-c-ios-core-ml-status-$target-archive-symbols.txt"
done

printf 'F27 link-import gate passed; consumers/probes are not executed\n'

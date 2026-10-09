#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F24 device/Simulator link-import gate" >&2
        exit 1
    fi
done

sh bindings/c/check-ios-mps-status.sh

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-mps-status-default-tree.txt
if rg -q '(^|[[:space:]])ios-mps-status v|objc2-metal-performance-shaders v|MetalPerformanceShaders.framework' \
    target/framework-c-ios-mps-status-default-tree.txt; then
    echo "MPS dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-mps-status \
    > target/framework-c-ios-mps-status-ios-tree.txt
rg -q 'ios-mps-status v' target/framework-c-ios-mps-status-ios-tree.txt
rg -q 'objc2-metal-performance-shaders feature "MPSCore"' \
    target/framework-c-ios-mps-status-ios-tree.txt
if rg -q 'objc2-metal-performance-shaders feature "default"' \
    target/framework-c-ios-mps-status-ios-tree.txt; then
    echo "objc2-metal-performance-shaders default features leaked into F24" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-mps-status \
    > target/framework-c-ios-mps-status-host-tree.txt
if rg -q '(^|[[:space:]])ios-mps-status v|objc2-metal-performance-shaders v|MetalPerformanceShaders.framework' \
    target/framework-c-ios-mps-status-host-tree.txt; then
    echo "iOS MPS dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-mps-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-mps-status -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-mps-status

cat > target/framework-c-ios-mps-status-c.c <<'FIXTURE_C'
#include <framework_ios_mps_status.h>
int main(void) {
    uint8_t available = 0;
    return (int)framework_ios_mps_status_preferred_device_available(&available);
}
FIXTURE_C
cat > target/framework-c-ios-mps-status-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_mps_status.h>
int main() {
    uint8_t available = 0;
    return static_cast<int>(framework_ios_mps_status_preferred_device_available(&available));
}
FIXTURE_CPP

expected_symbols=target/framework-c-ios-mps-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_mps_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-mps-status-c.c "$host_archive" \
    -o target/framework-c-ios-mps-status-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-mps-status-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-mps-status-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_mps_status_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-mps-status-host-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-mps-status-host-symbols.txt
nm -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-mps-status-host-undefined.txt
if rg -q 'MPSGetPreferredDevice|MetalPerformanceShaders|MTLDevice|objc_msgSend|OBJC_CLASS|swift_' \
    target/framework-c-ios-mps-status-host-undefined.txt; then
    echo "MPS, Objective-C, or Swift import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=12.2
            sdk=iphoneos
            clang_target=arm64-apple-ios12.2
            rust_min_flag=-miphoneos-version-min=12.2
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
        -p framework-c-api --no-default-features --features ios-mps-status \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-mps-status \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-mps-status \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-mps-status-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-mps-status-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-mps-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework Foundation -framework Metal -framework MetalPerformanceShaders \
            -lobjc -o "$binary"
        libraries="target/framework-c-ios-mps-status-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_mps_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-mps-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-mps-status-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-mps-status-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_MPSGetPreferredDevice' "$symbols"
        rg -q '_objc_release' "$symbols"
        if rg -q 'swift|_MTLCreateSystemDefaultDevice|_MTLCommand|_MPSSupportsMTLDevice|_MPS(Graph|Image|Matrix|NDArray)|objc_msgSend' \
            "$symbols"; then
            echo "Swift runtime, Objective-C messaging, or out-of-scope GPU/MPS import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-mps-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s MPS imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_mps_status_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-mps-status-$target-archive-symbols.txt"
    diff -u "$expected_symbols" \
        "target/framework-c-ios-mps-status-$target-archive-symbols.txt"
done

printf 'F24 device/Simulator link-import gate complete; no tests or consumer/probe binaries were executed\n'

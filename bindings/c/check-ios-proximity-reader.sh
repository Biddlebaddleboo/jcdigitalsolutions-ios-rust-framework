#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-proximity-reader.sh
cargo fmt --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_proximity_reader as $reader
    | $reader.cargo_feature == "ios-proximity-reader"
    and $reader.header == "framework_ios_proximity_reader.h"
    and $reader.symbols == ["framework_ios_proximity_reader_tap_to_pay_device_model_supported"]
    and ($reader.api | contains("iOS 15.4"))
    and $reader.link_probe_deployment_minimums["aarch64-apple-ios"] == "15.4"
    and $reader.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "15.4"
    and ($reader.status_mapping.success | contains("0 or 1"))
    and ($reader.ownership.output | contains("aligned writable uint8_t memory"))
    and ($reader.ownership.output | contains("full synchronous call"))
    and ($reader.ownership.output | contains("unsynchronized access"))
    and ($reader.ownership.output | contains("not retained"))
' bindings/c/abi-manifest.json > /dev/null

rg -Fq 'valid, properly aligned writable `uint8_t` memory' \
    bindings/c/src/ios_proximity_reader.rs bindings/c/include/framework_ios_proximity_reader.h
rg -Fq 'full synchronous call' \
    bindings/c/src/ios_proximity_reader.rs bindings/c/include/framework_ios_proximity_reader.h
rg -Fq 'unsynchronized access' \
    bindings/c/src/ios_proximity_reader.rs bindings/c/include/framework_ios_proximity_reader.h
rg -Fq 'does not retain the output address' \
    bindings/c/src/ios_proximity_reader.rs bindings/c/include/framework_ios_proximity_reader.h
rg -Fq 'if out_supported.is_null()' bindings/c/src/ios_proximity_reader.rs
rg -Fq 'null output' bindings/c/include/framework_ios_proximity_reader.h

jq -r '.optional_capabilities.ios_proximity_reader.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-ios-proximity-reader-expected-symbols.txt
rg -o 'framework_ios_proximity_reader_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_proximity_reader.h | sort -u \
    > target/framework-c-ios-proximity-reader-header-symbols.txt
diff -u target/framework-c-ios-proximity-reader-expected-symbols.txt \
    target/framework-c-ios-proximity-reader-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-proximity-reader-default-tree.txt
if rg -q 'ios-proximity-reader v|ProximityReader.framework' \
    target/framework-c-ios-proximity-reader-default-tree.txt; then
    echo "ProximityReader dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-proximity-reader \
    > target/framework-c-ios-proximity-reader-ios-tree.txt
rg -q 'ios-proximity-reader v' target/framework-c-ios-proximity-reader-ios-tree.txt
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-proximity-reader \
    > target/framework-c-ios-proximity-reader-host-tree.txt
if rg -q 'ios-proximity-reader v|ProximityReader.framework' \
    target/framework-c-ios-proximity-reader-host-tree.txt; then
    echo "iOS ProximityReader dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-proximity-reader
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-proximity-reader -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-proximity-reader

cat > target/framework-c-ios-proximity-reader-c.c <<'FIXTURE_C'
#include <framework_ios_proximity_reader.h>
int main(void) {
    FrameworkIosProximityReaderBoolean supported = 1;
    return (int)framework_ios_proximity_reader_tap_to_pay_device_model_supported(&supported);
}
FIXTURE_C
cat > target/framework-c-ios-proximity-reader-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_proximity_reader.h>
int main() {
    FrameworkIosProximityReaderBoolean supported = 1;
    return static_cast<int>(framework_ios_proximity_reader_tap_to_pay_device_model_supported(&supported));
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-proximity-reader-c.c "$host_archive" \
    -o target/framework-c-ios-proximity-reader-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-proximity-reader-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-proximity-reader-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_proximity_reader_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-proximity-reader-host-symbols.txt
diff -u target/framework-c-ios-proximity-reader-expected-symbols.txt \
    target/framework-c-ios-proximity-reader-host-symbols.txt
nm -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-proximity-reader-host-undefined.txt
if rg -q 'PaymentCardReader|ProximityReader|swift_|libobjc|objc_msgSend|OBJC_CLASS|UIKit' \
    target/framework-c-ios-proximity-reader-host-undefined.txt; then
    echo "Apple or Swift runtime import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=15.4
            sdk=iphoneos
            clang_target=arm64-apple-ios15.4
            rust_min_flag=-miphoneos-version-min=15.4
            ;;
        aarch64-apple-ios-sim)
            deployment_target=15.4
            sdk=iphonesimulator
            clang_target=arm64-apple-ios15.4-simulator
            rust_min_flag=-mios-simulator-version-min=15.4
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-proximity-reader \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-proximity-reader \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-proximity-reader \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-proximity-reader-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-proximity-reader-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-proximity-reader-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include \
            "$source" "$archive" -framework ProximityReader -o "$binary"
        imports="target/framework-c-ios-proximity-reader-$language-$target-imports.txt"
        otool -L "$binary" > "$imports"
        libraries="target/framework-c-ios-proximity-reader-$language-$target-libraries.txt"
        awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_proximity_reader.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-proximity-reader-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-proximity-reader-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-proximity-reader-$language-$target-symbols.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_\$s15ProximityReader011PaymentCardB0CMa' "$symbols"
        rg -q '_\$s15ProximityReader011PaymentCardB0C11isSupportedSbvgZ' "$symbols"
        if rg 'Swift\.framework|libswift|libobjc|objc_msgSend|OBJC_CLASS|UIKit|swift_' \
            "$imports" "$symbols"; then
            echo "unexpected Swift/Objective-C runtime or UI import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-proximity-reader-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        if ! rg -q 'minos[[:space:]]+15\.4([.]0)?([[:space:]]|$)|version[[:space:]]+15\.4([.]0)?([[:space:]]|$)' \
            "$build_info"; then
            echo "$binary does not declare the iOS 15.4 deployment floor" >&2
            exit 1
        fi
        printf '%s %s ProximityReader imports and iOS 15.4 minimum verified; probe not executed\n' \
            "$target" "$language"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_proximity_reader_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-proximity-reader-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-proximity-reader-expected-symbols.txt \
        "target/framework-c-ios-proximity-reader-$target-archive-symbols.txt"
done

printf 'F18 checks complete; no tests or consumer binaries were executed\n'

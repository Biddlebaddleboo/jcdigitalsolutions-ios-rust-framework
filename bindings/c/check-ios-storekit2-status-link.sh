#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F32 link/import gate" >&2
        exit 1
    fi
done

sh bindings/c/check-ios-storekit2-status.sh

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-storekit2-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-storekit2-status -- -D warnings
cargo doc --locked --no-deps -p framework-c-api --no-default-features \
    --features ios-storekit2-status
cargo build --locked --release -p framework-c-api --no-default-features
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-storekit2-status

host_archive=target/release/libframework_c_api.a
expected_symbols=target/framework-c-ios-storekit2-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_storekit2_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"
for language in c cpp; do
    case "$language" in
        c) compiler=clang; source=target/framework-c-ios-storekit2-status-c.c; standard=c11 ;;
        cpp) compiler=clang++; source=target/framework-c-ios-storekit2-status-cpp.cpp; standard=c++17 ;;
    esac
    binary="target/framework-c-ios-storekit2-status-$language-host"
    "$compiler" -std="$standard" -Wall -Wextra -Werror -pedantic -nostdlib++ \
        -I bindings/c/include "$source" "$host_archive" -o "$binary"
    nm -u "$binary" 2>/dev/null > "target/framework-c-ios-storekit2-status-$language-host-undefined.txt"
    if rg -q 'StoreKit|AppA0|swift_|objc_|OBJC_CLASS' \
        "target/framework-c-ios-storekit2-status-$language-host-undefined.txt"; then
        echo "StoreKit, Swift, or Objective-C import leaked into host $language consumer" >&2
        exit 1
    fi
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "target/framework-c-ios-storekit2-status-$language-host-libraries.txt"
    printf '%s\n' 'libSystem.B.dylib' \
        > "target/framework-c-ios-storekit2-status-$language-host-libraries-expected.txt"
    diff -u "target/framework-c-ios-storekit2-status-$language-host-libraries-expected.txt" \
        "target/framework-c-ios-storekit2-status-$language-host-libraries.txt"
done
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_storekit2_status_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-storekit2-status-host-archive-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-storekit2-status-host-archive-symbols.txt

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=10.0
            sdk=iphoneos
            clang_target=arm64-apple-ios10.0
            rust_min_flag=-miphoneos-version-min=10.0
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
        -p framework-c-api --no-default-features --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-storekit2-status --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-storekit2-status \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-storekit2-status --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-storekit2-status-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-storekit2-status-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-storekit2-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -nostdlib++ -I bindings/c/include \
            "$source" "$archive" -weak_framework StoreKit -Wl,-dead_strip_dylibs -o "$binary"
        imports="target/framework-c-ios-storekit2-status-$language-$target-imports.txt"
        otool -L "$binary" > "$imports"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "target/framework-c-ios-storekit2-status-$language-$target-libraries.txt"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_storekit2_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-storekit2-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-storekit2-status-$language-imports-expected.txt" \
            "target/framework-c-ios-storekit2-status-$language-$target-libraries.txt"
        rg -q 'StoreKit\.framework/StoreKit.*weak' "$imports"
        symbols="target/framework-c-ios-storekit2-status-$language-$target-undefined.txt"
        nm -m "$binary" 2>/dev/null > "$symbols"
        rg -Fq '_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ' "$symbols"
        rg -F '_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ' "$symbols" \
            | rg -q 'weak import|weak external'
        if rg -q 'SKPaymentQueue|SKPayment|Product|Transaction|purchase|swift_(retain|release|alloc|dealloc)|objc_msgSend|OBJC_CLASS' \
            "$symbols"; then
            echo "out-of-scope StoreKit, Swift runtime, or Objective-C import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-storekit2-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports %s (weak StoreKit symbol); minos %s; consumer not executed\n' \
            "$target" "$language" "$(tr '\n' ' ' < "target/framework-c-ios-storekit2-status-$language-$target-libraries.txt")" "$actual_deployment_target"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_storekit2_status_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-storekit2-status-$target-archive-symbols.txt"
    diff -u "$expected_symbols" "target/framework-c-ios-storekit2-status-$target-archive-symbols.txt"
done

printf 'F32 link/import gate complete; no tests, consumers, probes, or StoreKit calls were executed\n'

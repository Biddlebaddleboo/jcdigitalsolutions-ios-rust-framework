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
        echo "$tool is required for the F23 device/Simulator link-import gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-key-support.sh
sh -n bindings/c/check-ios-key-support-link.sh
rustfmt --check --edition 2024 bindings/c/src/ios_key_support.rs

jq -e '
    .optional_capabilities.ios_key_support as $key
    | $key.cargo_feature == "ios-key-support"
    and $key.header == "framework_ios_key_support.h"
    and $key.symbols == ["framework_ios_key_support_p256_ecdsa_sha256_message_supported"]
    and ($key.api | contains("iOS 10.0"))
    and $key.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $key.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($key.status_mapping.success | contains("0 or 1"))
    and ($key.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($key.status_mapping.panic | contains("PANIC"))
    and ($key.ownership.input | contains("borrowed"))
    and ($key.ownership.output | contains("not retained"))
    and ($key.direct_imports_64_bit_ios.c | sort) == [
        "CoreFoundation", "Security", "libSystem.B.dylib"
    ]
    and ($key.direct_imports_64_bit_ios.cpp | sort) == [
        "CoreFoundation", "Security", "libSystem.B.dylib", "libc++.1.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_key_support_p256_ecdsa_sha256_message_supported
rg -q 'pub unsafe extern "C" fn framework_ios_key_support_p256_ecdsa_sha256_message_supported' \
    bindings/c/src/ios_key_support.rs
rg -q "FrameworkStatus $symbol" bindings/c/include/framework_ios_key_support.h
rg -q '#\[cfg\(not\(target_os = "ios"\)\)\]' bindings/c/src/ios_key_support.rs
rg -q 'FrameworkStatus::UNSUPPORTED' bindings/c/src/ios_key_support.rs

jq -r '.optional_capabilities.ios_key_support.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u \
    > target/framework-c-ios-key-support-expected-symbols.txt
rg -o 'framework_ios_key_support_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_key_support.h | LC_ALL=C sort -u \
    > target/framework-c-ios-key-support-header-symbols.txt
diff -u target/framework-c-ios-key-support-expected-symbols.txt \
    target/framework-c-ios-key-support-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-key-support-default-tree.txt
if rg -q '(^|[[:space:]])(framework-key-support|ios-key-support) v|objc2-security v' \
    target/framework-c-ios-key-support-default-tree.txt; then
    echo "Security key-support dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-key-support \
    > target/framework-c-ios-key-support-ios-tree.txt
rg -q 'framework-key-support v' target/framework-c-ios-key-support-ios-tree.txt
rg -q 'ios-key-support v' target/framework-c-ios-key-support-ios-tree.txt
rg -q 'objc2-security v' target/framework-c-ios-key-support-ios-tree.txt
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-key-support \
    > target/framework-c-ios-key-support-host-tree.txt
if rg -q '(^|[[:space:]])(framework-key-support|ios-key-support) v|objc2-security v' \
    target/framework-c-ios-key-support-host-tree.txt; then
    echo "iOS Security key-support dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-key-support
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-key-support -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-key-support

cat > target/framework-c-ios-key-support-c.c <<'FIXTURE_C'
#include <framework_ios_key_support.h>
int main(void) {
    static const uint8_t key_bytes[65] = {0x04};
    FrameworkSlice key = {key_bytes, sizeof(key_bytes)};
    uint8_t supported = 0;
    return (int)framework_ios_key_support_p256_ecdsa_sha256_message_supported(key, &supported);
}
FIXTURE_C
cat > target/framework-c-ios-key-support-cpp.cpp <<'FIXTURE_CPP'
#include <stddef.h>
#include <framework_ios_key_support.h>
int main() {
    static const uint8_t key_bytes[65] = {0x04};
    FrameworkSlice key{key_bytes, sizeof(key_bytes)};
    uint8_t supported = 0;
    return static_cast<int>(framework_ios_key_support_p256_ecdsa_sha256_message_supported(key, &supported));
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-key-support-c.c "$host_archive" \
    -o target/framework-c-ios-key-support-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-key-support-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-key-support-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_key_support_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-key-support-host-symbols.txt
diff -u target/framework-c-ios-key-support-expected-symbols.txt \
    target/framework-c-ios-key-support-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-key-support-host-undefined.txt
if rg -q 'SecKey|Security|CoreFoundation|objc_msgSend|OBJC_CLASS|swift_' \
    target/framework-c-ios-key-support-host-undefined.txt; then
    echo "Security, Objective-C, or Swift imports leaked into the host archive" >&2
    exit 1
fi

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
        -p framework-c-api --no-default-features --features ios-key-support \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-key-support \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-key-support \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-key-support-c.c; standard=c11; cpp_only_flags= ;;
            cpp) compiler=clang++; source=target/framework-c-ios-key-support-cpp.cpp; standard=c++17; cpp_only_flags=-nostdinc++ ;;
        esac
        binary="target/framework-c-ios-key-support-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            $cpp_only_flags -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework CoreFoundation -framework Security -lSystem -o "$binary"
        libraries="target/framework-c-ios-key-support-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_key_support.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-key-support-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-key-support-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-key-support-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_SecKeyCreateWithData' "$symbols"
        rg -q '_SecKeyIsAlgorithmSupported' "$symbols"
        if rg -q 'SecItem(Add|CopyMatching|Delete|Update)|SecKey(CreateRandomKey|GeneratePair|CreateSignature|CreateEncryptedData|CreateDecryptedData)|objc_msgSend|OBJC_CLASS|swift_' \
            "$symbols"; then
            echo "Keychain, key-generation, cryptographic-operation, Objective-C, or Swift import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-key-support-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s CoreFoundation/Security imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_key_support_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-key-support-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-key-support-expected-symbols.txt \
        "target/framework-c-ios-key-support-$target-archive-symbols.txt"
done

printf 'F23 device/Simulator link-import gate complete; no test or probe binary was executed\n'

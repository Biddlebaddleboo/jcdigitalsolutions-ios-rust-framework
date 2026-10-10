#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for this audit: $llvm_nm" >&2
    exit 1
fi

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-crypto.sh
cargo fmt --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_crypto as $crypto
    | $crypto.cargo_feature == "ios-crypto"
    and $crypto.header == "framework_ios_crypto.h"
    and $crypto.symbols == ["framework_ios_crypto_sha256"]
    and ($crypto.api | contains("iOS 2.0"))
    and $crypto.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $crypto.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($crypto.status_mapping.success | contains("32-byte"))
    and ($crypto.status_mapping.null_output | contains("INVALID_ARGUMENT"))
    and ($crypto.status_mapping.null_output | contains("validating its pointer and input/output disjointness"))
    and ($crypto.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($crypto.ownership.output | contains("writable 32-byte digest region"))
    and ($crypto.ownership.input | contains("full synchronous call"))
    and ($crypto.ownership.input | contains("unsynchronized input access"))
    and ($crypto.ownership.output | contains("full synchronous call"))
    and ($crypto.ownership.output | contains("unsynchronized output access"))
    and ($crypto.ownership.output | contains("initialized to zero after structural and disjointness checks"))
    and ($crypto.ownership.output | contains("caller-owned and not retained"))
    and ($crypto.ownership.output | contains("not retained"))
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_crypto.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-ios-crypto-expected-symbols.txt
rg -o 'framework_ios_crypto_[A-Za-z0-9_]+' bindings/c/include/framework_ios_crypto.h \
    | sort -u > target/framework-c-ios-crypto-header-symbols.txt
diff -u target/framework-c-ios-crypto-expected-symbols.txt \
    target/framework-c-ios-crypto-header-symbols.txt
rg -q '#\[cfg\(not\(target_os = "ios"\)\)\]' bindings/c/src/ios_crypto.rs
rg -q 'FrameworkStatus::UNSUPPORTED' bindings/c/src/ios_crypto.rs
for file in bindings/c/src/ios_crypto.rs bindings/c/include/framework_ios_crypto.h \
    docs/bindings/ios-crypto.md PLAN_BINDINGS_F19.md; do
    rg -F -q 'keep input immutable' "$file"
    rg -F -q 'prevent unsynchronized' "$file"
    rg -F -q 'output access for the full call' "$file"
    rg -F -q 'cannot prove that' "$file"
    rg -F -q 'memory is valid, readable, writable, or live' "$file"
    rg -q 'pointer is retained after return|pointer is retained' "$file"
done
rg -F -q 'input_start.checked_add(input_len)?' bindings/c/src/ios_crypto.rs
rg -F -q 'output_start.checked_add(DIGEST_SIZE)?' bindings/c/src/ios_crypto.rs
rg -F -q 'Some(input_len != 0 && input_start < output_end && output_start < input_end)' \
    bindings/c/src/ios_crypto.rs
rg -F -q 'ptr::write_bytes(out_digest, 0, DIGEST_SIZE)' bindings/c/src/ios_crypto.rs
awk '
    /if out_digest\.is_null\(\)/ { null_output = NR }
    /let Ok\(input_len\) = usize::try_from\(input\.length\(\)\)/ { length_convert = NR }
    /if input_len > isize::MAX as usize/ { slice_limit = NR }
    /if input_len != 0 && input\.data\(\)\.is_null\(\)/ { null_input = NR }
    /input_output_overlap\(input\.data\(\), input_len, out_digest\)/ { overlap_check = NR }
    /ptr::write_bytes\(out_digest, 0, DIGEST_SIZE\)/ { zero_output = NR }
    /if input\.length\(\) > u64::from\(u32::MAX\)/ { common_crypto_limit = NR }
    /catch_unwind_status\(AssertUnwindSafe/ { platform_boundary = NR }
    /input\.as_bytes\(\)/ { input_slice = NR }
    END {
        if (!(null_output < length_convert && length_convert < slice_limit &&
              slice_limit < null_input && null_input < overlap_check &&
              overlap_check < zero_output && zero_output < common_crypto_limit &&
              common_crypto_limit < platform_boundary && platform_boundary < input_slice)) {
            exit 1
        }
    }
' bindings/c/src/ios_crypto.rs || {
    echo "F19 pointer validation, output initialization, and slice order mismatch" >&2
    exit 1
}

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-crypto-default-tree.txt
if rg -q '(^|[[:space:]])ios-crypto v|CC_SHA256|CommonCrypto' \
    target/framework-c-ios-crypto-default-tree.txt; then
    echo "iOS crypto dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-crypto \
    > target/framework-c-ios-crypto-ios-tree.txt
rg -q 'ios-crypto v' target/framework-c-ios-crypto-ios-tree.txt
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-crypto \
    > target/framework-c-ios-crypto-host-tree.txt
if rg -q '(^|[[:space:]])ios-crypto v|CC_SHA256|CommonCrypto' \
    target/framework-c-ios-crypto-host-tree.txt; then
    echo "iOS crypto dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-crypto
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-crypto -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-crypto

cat > target/framework-c-ios-crypto-c.c <<'FIXTURE_C'
#include <framework_ios_crypto.h>
_Static_assert(FRAMEWORK_IOS_CRYPTO_SHA256_DIGEST_SIZE == 32u, "digest size");
int main(void) {
    const uint8_t bytes[1] = {0};
    uint8_t digest[FRAMEWORK_IOS_CRYPTO_SHA256_DIGEST_SIZE] = {0};
    FrameworkSlice input = {bytes, sizeof(bytes)};
    return (int)framework_ios_crypto_sha256(input, digest);
}
FIXTURE_C
cat > target/framework-c-ios-crypto-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_crypto.h>
static_assert(FRAMEWORK_IOS_CRYPTO_SHA256_DIGEST_SIZE == 32u, "digest size");
int main() {
    const uint8_t bytes[1] = {0};
    uint8_t digest[FRAMEWORK_IOS_CRYPTO_SHA256_DIGEST_SIZE] = {0};
    FrameworkSlice input{bytes, sizeof(bytes)};
    return static_cast<int>(framework_ios_crypto_sha256(input, digest));
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-crypto-c.c "$host_archive" -o target/framework-c-ios-crypto-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-crypto-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-crypto-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_crypto_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-crypto-host-symbols.txt
diff -u target/framework-c-ios-crypto-expected-symbols.txt \
    target/framework-c-ios-crypto-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null > target/framework-c-ios-crypto-host-undefined.txt
if rg -q 'CC_SHA256|CommonCrypto|Security|CryptoKit|objc|swift_' \
    target/framework-c-ios-crypto-host-undefined.txt; then
    echo "Apple crypto, Objective-C, or Swift import leaked into the host archive" >&2
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
        -p framework-c-api --no-default-features --features ios-crypto --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-crypto \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-crypto --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-crypto-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-crypto-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-crypto-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -lSystem -o "$binary"
        libraries="target/framework-c-ios-crypto-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_crypto.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-crypto-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-crypto-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-crypto-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_CC_SHA256' "$symbols"
        if rg -q 'CommonCrypto\.framework|Security\.framework|CryptoKit|objc|swift_' \
            "$libraries" "$symbols"; then
            echo "out-of-scope framework or runtime import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-crypto-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s libSystem/_CC_SHA256 imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_crypto_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-crypto-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-crypto-expected-symbols.txt \
        "target/framework-c-ios-crypto-$target-archive-symbols.txt"
done

printf 'F19 checks complete; no tests or consumer binaries were executed\n'

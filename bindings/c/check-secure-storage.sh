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

cargo tree -p framework-c-api --no-default-features > target/framework-c-secure-storage-default-tree.txt
if rg -q 'framework-secure-storage|ios-secure-storage' target/framework-c-secure-storage-default-tree.txt; then
    echo "secure-storage dependencies leaked into the default C ABI build" >&2
    exit 1
fi
cargo tree -p framework-c-api --features secure-storage > target/framework-c-secure-storage-feature-tree.txt
rg -q 'framework-secure-storage' target/framework-c-secure-storage-feature-tree.txt
rg -q 'ios-secure-storage' target/framework-c-secure-storage-feature-tree.txt

cargo test -p framework-c-api --features secure-storage
cargo build --release -p framework-c-api --features secure-storage
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -c bindings/c/tests/secure_storage_header_c.c -o target/framework-c-secure-storage-header-c.o
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -c bindings/c/tests/secure_storage_header_cpp.cpp -o target/framework-c-secure-storage-header-cpp.o

archive=target/release/libframework_c_api.a
"$llvm_nm" -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-c-secure-storage-symbols.txt
jq -r '.c_symbols[], .optional_capabilities.ios_secure_storage.symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-secure-storage-expected-symbols.txt
jq -e '
    .optional_capabilities.ios_secure_storage as $cap |
    $cap.cargo_feature == "secure-storage" and
    $cap.header == "framework_ios_secure_storage.h" and
    ($cap.symbols | sort) == [
        "framework_ios_secure_storage_read",
        "framework_ios_secure_storage_remove",
        "framework_ios_secure_storage_store"
    ] and
    $cap.policy_bits.DEVICE_UNLOCK_REQUIRED == 1 and
    $cap.policy_bits.DEVICE_BOUND == 2 and
    $cap.policy_bits.allowed_mask == 3 and
    ($cap.targets | has("aarch64-apple-ios") and has("aarch64-apple-ios-sim") and has("non_ios")) and
    ($cap.outputs | has("read_found") and has("read_secret") and has("store_effective_policy_flags") and has("remove_removed") and has("native_os_status")) and
    ($cap.status_mapping | has("invalid_inputs") and has("unsupported_policy") and has("non_ios_valid_inputs") and has("backend_error") and has("platform_error") and has("non_platform_error") and has("read_buffer_allocation_failure") and has("panic")) and
    $cap.status_mapping.error_kind_codes.InvalidInput == "FRAMEWORK_STATUS_INVALID_ARGUMENT" and
    $cap.status_mapping.error_kind_codes.Unsupported == "FRAMEWORK_STATUS_UNSUPPORTED" and
    $cap.status_mapping.error_kind_codes.Unavailable == "FRAMEWORK_STATUS_UNAVAILABLE" and
    $cap.status_mapping.error_kind_codes.PermissionDenied == "FRAMEWORK_STATUS_PERMISSION_DENIED" and
    $cap.status_mapping.error_kind_codes.Cancelled == "FRAMEWORK_STATUS_CANCELLED" and
    $cap.status_mapping.error_kind_codes.Timeout == "FRAMEWORK_STATUS_TIMEOUT" and
    $cap.status_mapping.error_kind_codes.NotFound == "FRAMEWORK_STATUS_NOT_FOUND" and
    $cap.status_mapping.error_kind_codes.AlreadyExists == "FRAMEWORK_STATUS_ALREADY_EXISTS" and
    $cap.status_mapping.error_kind_codes.ResourceExhausted == "FRAMEWORK_STATUS_RESOURCE_EXHAUSTED" and
    $cap.status_mapping.error_kind_codes.Platform == "FRAMEWORK_STATUS_PLATFORM_ERROR" and
    $cap.status_mapping.error_kind_codes.UnknownOrInternal == "FRAMEWORK_STATUS_INTERNAL_ERROR" and
    $cap.status_mapping.unsupported_policy == "FRAMEWORK_STATUS_UNSUPPORTED when the iOS backend cannot satisfy the required policy" and
    $cap.status_mapping.non_ios_valid_inputs == "FRAMEWORK_STATUS_UNSUPPORTED after required output initialization and input validation" and
    $cap.status_mapping.platform_error == "FRAMEWORK_STATUS_PLATFORM_ERROR; preserve the exact nonzero OSStatus in the optional native output" and
    ($cap.ownership | has("inputs") and has("read_secret"))
' bindings/c/abi-manifest.json > /dev/null
rg -o 'framework_ios_secure_storage_[a-z_]+' bindings/c/include/framework_ios_secure_storage.h | sort -u > target/framework-c-secure-storage-header-symbols.txt
jq -r '.optional_capabilities.ios_secure_storage.symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-secure-storage-manifest-symbols.txt
diff -u target/framework-c-secure-storage-manifest-symbols.txt target/framework-c-secure-storage-header-symbols.txt
diff -u target/framework-c-secure-storage-expected-symbols.txt target/framework-c-secure-storage-symbols.txt

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include bindings/c/tests/secure_storage_stub.c "$archive" -o target/framework-c-secure-storage-stub
target/framework-c-secure-storage-stub

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo clippy -p framework-c-api --all-targets --features secure-storage --target "$target" -- -D warnings
    cargo build --release -p framework-c-api --features secure-storage --target "$target"
    target_archive="target/$target/release/libframework_c_api.a"
    "$llvm_nm" -u "$target_archive" > "target/framework-c-secure-storage-imports-$target.txt"
    rg -q 'SecItem(Add|CopyMatching|Update|Delete)|kSecAttrAccessible|kSecClassGenericPassword' "target/framework-c-secure-storage-imports-$target.txt"
    rg -q 'CFDictionaryCreate|CFDataCreate|kCFTypeDictionary' "target/framework-c-secure-storage-imports-$target.txt"
    if rg -qi 'swift|objc_msgSend|objc_retain|objc_release|objc_autorelease' "target/framework-c-secure-storage-imports-$target.txt"; then
        echo "unexpected Swift or Objective-C runtime import in $target archive" >&2
        exit 1
    fi

    probe_source=target/framework-c-secure-storage-link-probe.c
    cat > "$probe_source" <<'C'
#include "framework_ios_secure_storage.h"

int main(void) {
    FrameworkStr service = {0};
    FrameworkStr item = {0};
    FrameworkSlice secret = {0};
    FrameworkOwnedBuffer output = {0};
    uint8_t found = 0;
    uint8_t removed = 0;
    uint32_t effective_policy = 0;
    int32_t native_status = 0;

    FrameworkStatus read_status = framework_ios_secure_storage_read(
        service, item, &found, &output, &native_status);
    FrameworkStatus store_status = framework_ios_secure_storage_store(
        service, item, secret, 0, &effective_policy, &native_status);
    FrameworkStatus remove_status = framework_ios_secure_storage_remove(
        service, item, &removed, &native_status);
    framework_owned_buffer_destroy(&output);
    return (int)(read_status | store_status | remove_status);
}
C

    cat > target/framework-c-secure-storage-link-imports-expected.txt <<'IMPORTS'
CoreFoundation
Security
libSystem.B.dylib
IMPORTS

    case "$target" in
        aarch64-apple-ios)
            clang_target=arm64-apple-ios
            sdk_name=iphoneos
            minimum_os_flag=-miphoneos-version-min=10.0
            ;;
        aarch64-apple-ios-sim)
            clang_target=arm64-apple-ios-simulator
            sdk_name=iphonesimulator
            minimum_os_flag=-mios-simulator-version-min=10.0
            ;;
        *)
            echo "unsupported secure-storage probe target: $target" >&2
            exit 1
            ;;
    esac
    sdk_path=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    probe_binary="target/framework-c-secure-storage-link-probe-$target"
    probe_imports="target/framework-c-secure-storage-link-imports-$target.txt"
    probe_libraries="target/framework-c-secure-storage-link-libraries-$target.txt"
    probe_symbols="target/framework-c-secure-storage-link-symbols-$target.txt"
    clang -std=c11 -O0 -Wall -Wextra -Werror -pedantic \
        -target "$clang_target" "$minimum_os_flag" -isysroot "$sdk_path" \
        -I bindings/c/include "$probe_source" "$target_archive" \
        -framework Security -framework CoreFoundation \
        -Wl,-dead_strip -o "$probe_binary"
    otool -L "$probe_binary" > "$probe_imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$probe_imports" | LC_ALL=C sort > "$probe_libraries"
    diff -u target/framework-c-secure-storage-link-imports-expected.txt "$probe_libraries"

    nm -u "$probe_binary" > "$probe_symbols"
    rg -q 'SecItem(Add|CopyMatching|Update|Delete)|kSecAttrAccessible|kSecClassGenericPassword' "$probe_symbols"
    rg -q 'CFDictionaryCreate|CFDataCreate|kCFTypeDictionary' "$probe_symbols"
    if rg -qi 'swift|objc|CFNetwork|NSURLSession|nw_[A-Za-z0-9_]*|SCNetwork' "$probe_symbols"; then
        echo "unexpected Swift, Objective-C, or network import in $probe_symbols" >&2
        exit 1
    fi
done

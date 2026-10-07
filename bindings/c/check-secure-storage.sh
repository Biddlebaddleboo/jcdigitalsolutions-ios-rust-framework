#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

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
nm -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-c-secure-storage-symbols.txt
jq -r '.c_symbols[], .optional_capabilities.ios_secure_storage.symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-secure-storage-expected-symbols.txt
rg -q 'FrameworkStatus framework_ios_secure_storage_read\(' bindings/c/include/framework_ios_secure_storage.h
rg -q 'FrameworkStatus framework_ios_secure_storage_store\(' bindings/c/include/framework_ios_secure_storage.h
rg -q 'FrameworkStatus framework_ios_secure_storage_remove\(' bindings/c/include/framework_ios_secure_storage.h
diff -u target/framework-c-secure-storage-expected-symbols.txt target/framework-c-secure-storage-symbols.txt

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include bindings/c/tests/secure_storage_stub.c "$archive" -o target/framework-c-secure-storage-stub
target/framework-c-secure-storage-stub

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --release -p framework-c-api --features secure-storage --target "$target"
    target_archive="target/$target/release/libframework_c_api.a"
    nm -u "$target_archive" > "target/framework-c-secure-storage-imports-$target.txt"
    rg -q 'SecItem(Add|CopyMatching|Update|Delete)|kSecAttrAccessible|kSecClassGenericPassword' "target/framework-c-secure-storage-imports-$target.txt"
    rg -q 'CFDictionaryCreate|CFDataCreate|kCFTypeDictionary' "target/framework-c-secure-storage-imports-$target.txt"
    if rg -qi 'swift|objc_msgSend|objc_retain|objc_release|objc_autorelease' "target/framework-c-secure-storage-imports-$target.txt"; then
        echo "unexpected Swift or Objective-C runtime import in $target archive" >&2
        exit 1
    fi
done

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

cargo build --release -p framework-c-api
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -c bindings/c/tests/header_c.c -o target/framework-c-header-c.o
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -c bindings/c/tests/header_cpp.cpp -o target/framework-c-header-cpp.o
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include bindings/c/tests/layout.c -o target/framework-c-layout
target/framework-c-layout | jq -S . > target/framework-c-layout-actual.json
jq -S '.targets["64-bit-pointer-and-u64-alignment-8"]' bindings/c/abi-manifest.json > target/framework-c-layout-expected.json
diff -u target/framework-c-layout-expected.json target/framework-c-layout-actual.json

jq -r '.status_codes | to_entries[] | "FRAMEWORK_STATUS_" + .key + " " + (.value | tostring)' bindings/c/abi-manifest.json | LC_ALL=C sort > target/framework-c-api-expected-status-codes.txt
awk '$1 == "#define" && $2 ~ /^FRAMEWORK_STATUS_/ { value = $3; sub(/^UINT32_C\(/, "", value); sub(/\)$/, "", value); print $2, value }' bindings/c/include/framework.h | LC_ALL=C sort > target/framework-c-api-status-codes.txt
diff -u target/framework-c-api-expected-status-codes.txt target/framework-c-api-status-codes.txt

archive=target/release/libframework_c_api.a
nm -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-c-api-symbols.txt
jq -r '.c_symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-api-expected-symbols.txt
rg -q 'uint64_t framework_abi_version\(void\);' bindings/c/include/framework.h
rg -q 'void framework_owned_buffer_destroy\(FrameworkOwnedBuffer \*buffer\);' bindings/c/include/framework.h
diff -u target/framework-c-api-expected-symbols.txt target/framework-c-api-symbols.txt

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include examples/c-minimal/main.c "$archive" -o target/framework-c-minimal
target/framework-c-minimal

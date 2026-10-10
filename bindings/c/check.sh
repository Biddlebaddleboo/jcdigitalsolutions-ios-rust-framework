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

jq -e '
  .abi_version.major == 1 and
  .abi_version.minor == 2 and
  .options_v1.minimum_struct_size == 16 and
  .options_v1.abi_version == 1 and
  .options_v1.reserved_must_be_zero == true and
  .options_v1.validator_symbol == "framework_options_v1_validate" and
  (.options_v1.input_contract | contains("fully initialized")) and
  (.options_v1.future_sized_struct_check | contains("struct_size >= 16")) and
  (.options_v1.future_sized_struct_check | contains("abi_version is 1")) and
  (.options_v1.future_sized_struct_check | contains("ignore all flags and trailing bytes"))
' bindings/c/abi-manifest.json > /dev/null
jq -e '
  .owned_buffer_copy.symbol == "framework_owned_buffer_copy" and
  (.owned_buffer_copy.status_mapping.null_output | contains("INVALID_ARGUMENT")) and
  (.owned_buffer_copy.status_mapping.zero_length | contains("no allocation")) and
  (.owned_buffer_copy.status_mapping.length_overflow_or_exceeds_rust_slice_limit | contains("INVALID_ARGUMENT")) and
  (.owned_buffer_copy.status_mapping.allocation_or_descriptor_capacity_failure | contains("RESOURCE_EXHAUSTED")) and
  .ownership.FrameworkOwnedBuffer.core_creator == "framework_owned_buffer_copy"
' bindings/c/abi-manifest.json > /dev/null
rg -Fq 'pub use options_v1::framework_options_v1_validate' bindings/c/src/lib.rs
rg -Fq 'pub unsafe extern "C" fn framework_options_v1_validate' bindings/c/src/options_v1.rs
rg -Fq 'if options.is_null()' bindings/c/src/options_v1.rs
rg -Fq 'options.struct_size < core::mem::size_of::<FrameworkOptionsV1>() as u32' bindings/c/src/options_v1.rs
rg -Fq 'options.abi_version != ABI_VERSION_MAJOR' bindings/c/src/options_v1.rs
rg -Fq 'options.reserved != 0' bindings/c/src/options_v1.rs
rg -Fq 'return FrameworkStatus::UNSUPPORTED;' bindings/c/src/options_v1.rs
rg -Fq 'FrameworkStatus::OK' bindings/c/src/options_v1.rs
rg -Fq 'framework_options_v1_validate' examples/c-minimal/main.c

jq -r '.status_codes | to_entries[] | "FRAMEWORK_STATUS_" + .key + " " + (.value | tostring)' bindings/c/abi-manifest.json | LC_ALL=C sort > target/framework-c-api-expected-status-codes.txt
awk '$1 == "#define" && $2 ~ /^FRAMEWORK_STATUS_/ { value = $3; sub(/^UINT32_C\(/, "", value); sub(/\)$/, "", value); print $2, value }' bindings/c/include/framework.h | LC_ALL=C sort > target/framework-c-api-status-codes.txt
diff -u target/framework-c-api-expected-status-codes.txt target/framework-c-api-status-codes.txt

archive=target/release/libframework_c_api.a
nm -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-c-api-symbols.txt
jq -r '.c_symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-api-expected-symbols.txt
rg -q 'uint64_t framework_abi_version\(void\);' bindings/c/include/framework.h
rg -Fq 'FrameworkStatus framework_options_v1_validate(const FrameworkOptionsV1 *options);' bindings/c/include/framework.h
rg -Fq 'FrameworkStatus framework_owned_buffer_copy(FrameworkSlice bytes, FrameworkOwnedBuffer *out_buffer);' bindings/c/include/framework.h
rg -q 'void framework_owned_buffer_destroy\(FrameworkOwnedBuffer \*buffer\);' bindings/c/include/framework.h
rg -Fq 'pub unsafe extern "C" fn framework_owned_buffer_copy' crates/framework-abi/src/lib.rs
rg -Fq 'framework_owned_buffer_copy, framework_owned_buffer_destroy' bindings/c/src/lib.rs
diff -u target/framework-c-api-expected-symbols.txt target/framework-c-api-symbols.txt

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include examples/c-minimal/main.c "$archive" -o target/framework-c-minimal
target/framework-c-minimal

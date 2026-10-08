#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

cargo build --release -p framework-c-api

format_tool=$(xcrun -f clang-format)
"$format_tool" --dry-run --Werror bindings/cpp/include/framework.hpp bindings/cpp/tests/consumer.cpp bindings/cpp/tests/owner-consumer.cpp bindings/cpp/tests/ownership.cpp

flags="-std=c++17 -fno-exceptions -fno-rtti -Wall -Wextra -Werror -pedantic -I bindings/c/include -I bindings/cpp/include"
clang++ $flags -c bindings/cpp/tests/consumer.cpp -o target/framework-cpp-consumer.o
nm -u target/framework-cpp-consumer.o | awk '{ name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-cpp-consumer-symbols.txt
cat > target/framework-cpp-consumer-expected-symbols.txt <<'EOF'
framework_abi_version
EOF
diff -u target/framework-cpp-consumer-expected-symbols.txt target/framework-cpp-consumer-symbols.txt
clang++ $flags -nostdlib++ bindings/cpp/tests/consumer.cpp target/release/libframework_c_api.a -o target/framework-cpp-consumer
target/framework-cpp-consumer

clang++ $flags -c bindings/cpp/tests/owner-consumer.cpp -o target/framework-cpp-owner-consumer.o
nm -u target/framework-cpp-owner-consumer.o | awk '{ name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-cpp-owner-consumer-symbols.txt
cat > target/framework-cpp-owner-consumer-expected-symbols.txt <<'EOF'
framework_owned_buffer_destroy
EOF
diff -u target/framework-cpp-owner-consumer-expected-symbols.txt target/framework-cpp-owner-consumer-symbols.txt

clang++ $flags -c bindings/cpp/tests/ownership.cpp -o target/framework-cpp-ownership.o
nm -u target/framework-cpp-ownership.o | awk '{ name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-cpp-ownership-symbols.txt
: > target/framework-cpp-ownership-expected-symbols.txt
diff -u target/framework-cpp-ownership-expected-symbols.txt target/framework-cpp-ownership-symbols.txt
clang++ $flags -nostdlib++ bindings/cpp/tests/ownership.cpp -o target/framework-cpp-ownership
target/framework-cpp-ownership

#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/swift-abi-runtime-layout.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT HUP INT TERM

clang -std=c11 -Wall -Wextra -Werror \
    -I "$repo_root/interop/swift-abi-core/include" \
    "$repo_root/interop/swift-abi-core/tests/runtime-layout.c" \
    -o "$test_dir/runtime-layout"
"$test_dir/runtime-layout"
printf '%s\n' 'shared Swift metadata and value-witness layout fixture passed'

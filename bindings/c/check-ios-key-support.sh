#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

source=bindings/c/src/ios_key_support.rs
header=bindings/c/include/framework_ios_key_support.h

sh -n bindings/c/check-ios-key-support.sh
sh -n bindings/c/check-ios-key-support-link.sh
rustfmt --check --edition 2024 "$source"

source_symbol=$(rg -o 'framework_ios_key_support_p256_ecdsa_sha256_message_supported' "$source" | sort -u)
header_symbol=$(rg -o 'framework_ios_key_support_p256_ecdsa_sha256_message_supported' "$header" | sort -u)
if [ "$source_symbol" != "$header_symbol" ]; then
    echo "F23 Rust export and C header symbol do not match" >&2
    exit 1
fi
for contract_file in "$source" "$header"; do
    rg -q 'valid, properly' "$contract_file"
    rg -q 'aligned writable memory' "$contract_file"
    rg -q 'during this synchronous call' "$contract_file"
    rg -q 'unsynchronized concurrent access' "$contract_file"
    rg -q 'pointer-range arithmetic and overlap' "$contract_file"
    rg -q 'cannot validate' "$contract_file"
    rg -q 'does not retain' "$contract_file"
    rg -q 'output address' "$contract_file"
done

if rg -n '[[:blank:]]+$' \
    PLAN_BINDINGS_F23.md \
    docs/bindings/ios-key-support.md \
    "$source" "$header" bindings/c/check-ios-key-support.sh \
    bindings/c/check-ios-key-support-link.sh; then
    echo "F23 file has trailing whitespace" >&2
    exit 1
fi

printf '#include <framework_ios_key_support.h>\n' | \
    clang -std=c11 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c -fsyntax-only -
printf '#include <stddef.h>\n#include <framework_ios_key_support.h>\n' | \
    clang++ -std=c++17 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c++ -fsyntax-only -

printf 'F23 static checks complete; no Cargo build, link, test, consumer, or probe was run\n'

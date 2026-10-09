#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

source=bindings/c/src/ios_natural_language_status.rs
header=bindings/c/include/framework_ios_natural_language_status.h
guide=docs/bindings/ios-natural-language-status.md
plan=PLAN_BINDINGS_F29.md

for tool in cargo clang clang++ diff jq python3 rg rustfmt sh; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F29 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-natural-language-status.sh
sh -n bindings/c/check-ios-natural-language-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_natural_language_english_contextual_embedding_assets
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'out_status.write\(0\)' "$source"
rg -U -Fq 'unsafe { out_status.write(0) };
    catch_unwind_status' "$source"
rg -q 'ios_natural_language_status::english_contextual_embedding_assets\(\)' "$source"
if rg -n 'requestEmbeddingAssets|loadWithError|\.unload\(|embeddingResultForString|requestAssets|NLLanguageRecognizer|NLTagger' "$source"; then
    echo "Natural Language model operation leaked into F29 source" >&2
    exit 1
fi
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
rg -q 'temporary `NLContextualEmbedding` object' "$header"
rg -q 'full synchronous call' "$source"
rg -q 'prevent unsynchronized concurrent access' "$source"
rg -q 'only nullness' "$source"
rg -q 'does not' "$source"
rg -q 'retain the output pointer' "$source"
rg -q 'caller-owned and not retained' "$header"
rg -Fq 'valid, properly aligned' "$header"
rg -Fq 'writable `uint32_t`' "$header"
rg -Fq 'storage for the full synchronous call' "$header"
rg -q 'caller must prevent' "$header"
rg -q 'unsynchronized' "$header"
rg -U -Fq 'if out_status.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }' "$source"
awk '
    /if out_status\.is_null\(\)/ { null_check = NR }
    /out_status\.write\(0\)/ && !zero_write { zero_write = NR }
    /catch_unwind_status\(AssertUnwindSafe/ { platform_boundary = NR }
    END {
        if (!(null_check < zero_write && zero_write < platform_boundary)) exit 1
    }
' "$source" || {
    echo "F29 output initialization order mismatch" >&2
    exit 1
}
for contract_file in "$source" "$header" "$guide" "$plan"; do
    rg -Fq 'valid' "$contract_file"
    rg -Fq 'properly aligned' "$contract_file"
    rg -Fq 'writable' "$contract_file"
    rg -Fq 'uint32_t' "$contract_file"
    rg -Fq 'full' "$contract_file"
    rg -Fq 'synchronous call' "$contract_file"
    rg -Fq 'caller must prevent' "$contract_file"
    rg -Fq 'unsynchronized' "$contract_file"
    rg -Fq 'concurrent access' "$contract_file"
    rg -Fq 'only nullness' "$contract_file"
    rg -Fq 'does not' "$contract_file"
    rg -Fq 'retain the output pointer' "$contract_file"
    rg -Fq 'output pointer' "$contract_file"
done
for contract_file in "$header" "$guide" "$plan"; do
    rg -Fq 'null output pointer returns' "$contract_file"
    rg -Fq 'FRAMEWORK_STATUS_INVALID_ARGUMENT' "$contract_file"
    rg -Fq 'without a write' "$contract_file"
    rg -Fq 'zero before platform' "$contract_file"
done

jq -e '
    .optional_capabilities.ios_natural_language_status as $natural
    | $natural.cargo_feature == "ios-natural-language-status"
    and $natural.header == "framework_ios_natural_language_status.h"
    and $natural.symbols == ["framework_ios_natural_language_english_contextual_embedding_assets"]
    and ($natural.status_codes | keys | sort) == [
        "FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE",
        "FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE",
        "FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE",
        "FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL",
        "FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE"
    ]
    and ($natural.status_codes | to_entries | map(.value) | sort) == [0, 1, 2, 3, 4]
    and $natural.link_probe_deployment_minimums["aarch64-apple-ios"] == "17.0"
    and $natural.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "17.0"
    and ($natural.api | contains("iOS 17.0 API floor"))
    and ($natural.status_mapping.success | contains("five Rust enum variants"))
    and ($natural.status_mapping.unavailable_api_floor | contains("UNAVAILABLE"))
    and ($natural.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($natural.status_mapping.panic | contains("PANIC"))
    and ($natural.ownership.output | contains("uint32_t"))
    and ($natural.ownership.output | contains("valid properly aligned writable memory for the synchronous call"))
    and ($natural.ownership.output | contains("nullness is checked only"))
    and ($natural.ownership.output | contains("must not be accessed concurrently without synchronization"))
    and ($natural.ownership.output | contains("not retained"))
    and (($natural.direct_imports_64_bit_ios | keys | sort) == ["c", "cpp"])
    and ($natural.direct_imports_64_bit_ios.c | sort) == [
        "Foundation", "NaturalLanguage", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
    and ($natural.direct_imports_64_bit_ios.cpp | sort) == [
        "Foundation", "NaturalLanguage", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

if rg -n '[[:blank:]]+$' \
    PLAN_BINDINGS_F29.md PLAN_VALIDATION_C_ABI_NATURAL_LANGUAGE.md \
    docs/bindings/ios-natural-language-status.md "$source" "$header" \
    bindings/c/check-ios-natural-language-status.sh \
    bindings/c/check-ios-natural-language-status-link.sh; then
    echo "F29 file has trailing whitespace" >&2
    exit 1
fi

cat > target/framework-c-ios-natural-language-status-c.c <<'FIXTURE_C'
#include <framework_ios_natural_language_status.h>
_Static_assert(sizeof(FrameworkIosNaturalLanguageAssetStatus) == sizeof(uint32_t), "status width");
_Static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE == 0, "unavailable code");
_Static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE == 1, "language code");
_Static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL == 2, "no-model code");
_Static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE == 3, "assets-absent code");
_Static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE == 4, "assets-present code");
int main(void) {
    FrameworkIosNaturalLanguageAssetStatus status = UINT32_MAX;
    return (int)framework_ios_natural_language_english_contextual_embedding_assets(&status);
}
FIXTURE_C

cat > target/framework-c-ios-natural-language-status-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_natural_language_status.h>
static_assert(sizeof(FrameworkIosNaturalLanguageAssetStatus) == sizeof(uint32_t), "status width");
static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_UNAVAILABLE == 0, "unavailable code");
static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_LANGUAGE_UNAVAILABLE == 1, "language code");
static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_NO_MODEL == 2, "no-model code");
static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_NOT_AVAILABLE == 3, "assets-absent code");
static_assert(FRAMEWORK_IOS_NATURAL_LANGUAGE_ASSET_STATUS_ASSETS_AVAILABLE == 4, "assets-present code");
int main() {
    FrameworkIosNaturalLanguageAssetStatus status = UINT32_MAX;
    return static_cast<int>(framework_ios_natural_language_english_contextual_embedding_assets(&status));
}
FIXTURE_CPP

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-natural-language-status-c.c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-natural-language-status-cpp.cpp

expected_symbols=target/framework-c-ios-natural-language-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_natural_language_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"
rg -o 'framework_ios_natural_language_[A-Za-z0-9_]+' "$header" | LC_ALL=C sort -u \
    > target/framework-c-ios-natural-language-status-header-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-natural-language-status-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-natural-language-status-default-tree.txt
if rg -q 'ios-natural-language-status v|objc2-natural-language v|NaturalLanguage.framework' \
    target/framework-c-ios-natural-language-status-default-tree.txt; then
    echo "Natural Language dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-natural-language-status \
    > target/framework-c-ios-natural-language-status-ios-tree.txt
rg -q 'ios-natural-language-status v' target/framework-c-ios-natural-language-status-ios-tree.txt
rg -q 'objc2-natural-language feature "NLContextualEmbedding"' \
    target/framework-c-ios-natural-language-status-ios-tree.txt
rg -q 'objc2-natural-language feature "NLLanguage"' \
    target/framework-c-ios-natural-language-status-ios-tree.txt
if rg -q 'objc2-natural-language feature "block2"|objc2-natural-language feature "objc2-core-ml"|objc2-natural-language feature "default"' \
    target/framework-c-ios-natural-language-status-ios-tree.txt; then
    echo "Natural Language block, Core ML, or default features leaked into F29" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-natural-language-status \
    > target/framework-c-ios-natural-language-status-host-tree.txt
if rg -q 'ios-natural-language-status v|objc2-natural-language v|NaturalLanguage.framework' \
    target/framework-c-ios-natural-language-status-host-tree.txt; then
    echo "iOS Natural Language dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-natural-language-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-natural-language-status -- -D warnings
cargo doc --locked -p framework-c-api --no-deps
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-natural-language-status --target aarch64-apple-ios
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-natural-language-status --target aarch64-apple-ios-sim
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-natural-language-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-natural-language-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p framework-c-api --no-deps --target aarch64-apple-ios

printf 'F29 static/build gate passed; no tests, C/C++ consumers, or link probes were built or run\n'

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

source=bindings/c/src/ios_extension_support.rs
header=bindings/c/include/framework_ios_extension_support.h

for tool in cargo clang clang++ diff jq python3 rg rustfmt sh; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F30 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-extension-support.sh
sh -n bindings/c/check-ios-extension-support-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_extension_support_read_extension_point_identifier
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'FrameworkStatus::RESOURCE_EXHAUSTED' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'out_identifier.write\(FrameworkOwnedBuffer::default\(\)\)' "$source"
rg -q 'ios_extension_support::read_extension_point_identifier\(bundle_path\)' "$source"
if rg -n 'bundle\.load|loadAndReturnError|NSExtensionMain|NSExtensionPrincipalClass|ExtensionKit|PlugInKit|requestOpenURL|swift_' "$source"; then
    echo "extension loading, lifecycle, or Swift surface leaked into F30 source" >&2
    exit 1
fi
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
rg -q 'framework_owned_buffer_destroy' "$header"
rg -q 'does not load extension code' "$header"

jq -e '
    .optional_capabilities.ios_extension_support as $extension
    | $extension.cargo_feature == "ios-extension-support"
    and $extension.header == "framework_ios_extension_support.h"
    and $extension.symbols == ["framework_ios_extension_support_read_extension_point_identifier"]
    and ($extension.metadata_error_codes | keys | sort) == [
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER",
        "FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE"
    ]
    and ($extension.metadata_error_codes | to_entries | map(.value) | sort) == [0, 1, 2, 3, 4, 5, 6, 7, 8]
    and $extension.link_probe_deployment_minimums["aarch64-apple-ios"] == "12.0"
    and $extension.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($extension.api | contains("iOS 4.0 API floor"))
    and ($extension.status_mapping.success | contains("FrameworkOwnedBuffer"))
    and ($extension.status_mapping.metadata_error | contains("B77 metadata error code"))
    and ($extension.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($extension.status_mapping.panic | contains("PANIC"))
    and ($extension.ownership.output | contains("disjoint"))
    and (($extension.direct_imports_64_bit_ios | keys | sort) == ["c", "cpp"])
    and ($extension.direct_imports_64_bit_ios.c | sort) == [
        "Foundation", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
    and ($extension.direct_imports_64_bit_ios.cpp | sort) == [
        "Foundation", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

if rg -n '[[:blank:]]+$' \
    PLAN_BINDINGS_F30.md PLAN_VALIDATION_C_ABI_EXTENSION_SUPPORT.md \
    docs/bindings/ios-extension-support.md "$source" "$header" \
    bindings/c/check-ios-extension-support.sh \
    bindings/c/check-ios-extension-support-link.sh; then
    echo "F30 file has trailing whitespace" >&2
    exit 1
fi

cat > target/framework-c-ios-extension-support-c.c <<'FIXTURE_C'
#include <framework_ios_extension_support.h>
#include <stddef.h>
_Static_assert(sizeof(FrameworkIosExtensionMetadataError) == sizeof(uint32_t), "error width");
_Static_assert(sizeof(FrameworkOwnedBuffer) == 24, "owned buffer size");
_Static_assert(offsetof(FrameworkOwnedBuffer, capacity) == 16, "owned buffer capacity offset");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE == 0, "none code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH == 1, "path code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE == 2, "bundle code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE == 3, "info code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY == 4, "missing dictionary code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE == 5, "dictionary type code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER == 6, "missing point code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE == 7, "point type code");
_Static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER == 8, "empty point code");
int main(void) {
    FrameworkStr path = { NULL, 0 };
    FrameworkIosExtensionMetadataError error = FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE;
    FrameworkOwnedBuffer identifier = { NULL, 0, 0 };
    FrameworkStatus status = framework_ios_extension_support_read_extension_point_identifier(
        path, &error, &identifier);
    framework_owned_buffer_destroy(&identifier);
    return (int)status;
}
FIXTURE_C

cat > target/framework-c-ios-extension-support-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_extension_support.h>
#include <cstddef>
static_assert(sizeof(FrameworkIosExtensionMetadataError) == sizeof(uint32_t), "error width");
static_assert(sizeof(FrameworkOwnedBuffer) == 24, "owned buffer size");
static_assert(offsetof(FrameworkOwnedBuffer, capacity) == 16, "owned buffer capacity offset");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE == 0, "none code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_BUNDLE_PATH == 1, "path code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_BUNDLE_UNAVAILABLE == 2, "bundle code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INFO_DICTIONARY_UNAVAILABLE == 3, "info code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_EXTENSION_DICTIONARY == 4, "missing dictionary code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_EXTENSION_DICTIONARY_TYPE == 5, "dictionary type code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_MISSING_POINT_IDENTIFIER == 6, "missing point code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_INVALID_POINT_IDENTIFIER_TYPE == 7, "point type code");
static_assert(FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_EMPTY_POINT_IDENTIFIER == 8, "empty point code");
int main() {
    FrameworkStr path{nullptr, 0};
    FrameworkIosExtensionMetadataError error = FRAMEWORK_IOS_EXTENSION_METADATA_ERROR_NONE;
    FrameworkOwnedBuffer identifier{nullptr, 0, 0};
    FrameworkStatus status = framework_ios_extension_support_read_extension_point_identifier(
        path, &error, &identifier);
    framework_owned_buffer_destroy(&identifier);
    return static_cast<int>(status);
}
FIXTURE_CPP

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-extension-support-c.c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-extension-support-cpp.cpp

expected_symbols=target/framework-c-ios-extension-support-expected-symbols.txt
jq -r '.optional_capabilities.ios_extension_support.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"
rg -o 'framework_ios_extension_support_[A-Za-z0-9_]+' "$header" | LC_ALL=C sort -u \
    > target/framework-c-ios-extension-support-header-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-extension-support-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-extension-support-default-tree.txt
if rg -q 'ios-extension-support v|objc2-foundation v|Foundation.framework' \
    target/framework-c-ios-extension-support-default-tree.txt; then
    echo "extension metadata dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-extension-support \
    > target/framework-c-ios-extension-support-ios-tree.txt
rg -q 'ios-extension-support v' target/framework-c-ios-extension-support-ios-tree.txt
for feature in NSBundle NSDictionary NSString NSURL; do
    rg -q "objc2-foundation feature \"$feature\"" \
        target/framework-c-ios-extension-support-ios-tree.txt
done
if rg -q 'objc2-foundation feature "default"|objc2-foundation feature "block2"' \
    target/framework-c-ios-extension-support-ios-tree.txt; then
    echo "Foundation default or block features leaked into F30" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-extension-support \
    > target/framework-c-ios-extension-support-host-tree.txt
if rg -q 'ios-extension-support v|objc2-foundation v|Foundation.framework' \
    target/framework-c-ios-extension-support-host-tree.txt; then
    echo "iOS extension metadata dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-extension-support
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-extension-support -- -D warnings
cargo doc --locked -p framework-c-api --no-deps
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-extension-support --target aarch64-apple-ios
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-extension-support --target aarch64-apple-ios-sim
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-extension-support --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-extension-support --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p framework-c-api --no-deps --target aarch64-apple-ios

printf 'F30 static/build gate passed; no tests, C/C++ consumers, or link probes were built or run\n'

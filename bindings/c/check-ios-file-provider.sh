#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-file-provider.sh
cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_file_provider as $fp
    | $fp.cargo_feature == "ios-file-provider"
    and $fp.header == "framework_ios_file_provider.h"
    and ($fp.symbols | sort) == [
        "framework_ios_file_provider_operation_destroy",
        "framework_ios_file_provider_operation_poll",
        "framework_ios_file_provider_registered_domain_presence_start"
    ]
    and $fp.link_probe_deployment_minimums["aarch64-apple-ios"] == "11.0"
    and $fp.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and $fp.FrameworkIosFileProviderResultV1.size == 48
    and $fp.FrameworkIosFileProviderResultV1.align == 8
    and ($fp.api | contains("getDomainsWithCompletionHandler:"))
    and ($fp.threading | contains("unspecified callback queue"))
    and ($fp.ownership.native_request | contains("no cancellation path"))
' bindings/c/abi-manifest.json > /dev/null
jq -e '
    .ownership.FrameworkOwnedBuffer.optional_capability_creators
    | index("framework_ios_file_provider_operation_poll") != null
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_file_provider.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-ios-file-provider-expected-symbols.txt
rg -o 'framework_ios_file_provider_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_file_provider.h | sort -u \
    > target/framework-c-ios-file-provider-header-symbols.txt
diff -u target/framework-c-ios-file-provider-expected-symbols.txt \
    target/framework-c-ios-file-provider-header-symbols.txt

{
    cat <<'ASSERT_HEADER'
#if defined(__cplusplus)
#define FRAMEWORK_FP_ASSERT static_assert
#define FRAMEWORK_FP_ALIGNOF alignof
#else
#define FRAMEWORK_FP_ASSERT _Static_assert
#define FRAMEWORK_FP_ALIGNOF _Alignof
#endif
ASSERT_HEADER
    jq -r '
        .optional_capabilities.ios_file_provider.FrameworkIosFileProviderResultV1 as $r
        | "FRAMEWORK_FP_ASSERT(sizeof(FrameworkIosFileProviderResultV1) == "
          + ($r.size | tostring) + ", \"result size\");"
        + "\nFRAMEWORK_FP_ASSERT(FRAMEWORK_FP_ALIGNOF(FrameworkIosFileProviderResultV1) == "
          + ($r.align | tostring) + ", \"result alignment\");"
        + ($r.fields | to_entries | map(
            "\nFRAMEWORK_FP_ASSERT(offsetof(FrameworkIosFileProviderResultV1, "
            + .key + ") == " + (.value | tostring) + ", \"" + .key + " offset\");"
          ) | join(""))
    ' bindings/c/abi-manifest.json
} > target/framework-c-ios-file-provider-manifest-asserts.h

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-file-provider-default-tree.txt
if rg -q '(^|[[:space:]])ios-file-provider v|objc2-file-provider v|FileProvider.framework' \
    target/framework-c-ios-file-provider-default-tree.txt; then
    echo "File Provider dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-file-provider \
    > target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'ios-file-provider v' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-file-provider v' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'block2 v' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-file-provider feature "Extension"' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-file-provider feature "NSFileProviderDomain"' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-file-provider feature "block2"' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-foundation feature "NSArray"' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-foundation feature "NSError"' target/framework-c-ios-file-provider-ios-tree.txt
rg -q 'objc2-foundation feature "NSString"' target/framework-c-ios-file-provider-ios-tree.txt
if rg -q 'objc2-file-provider feature "(default|ReplicatedExtension|NSFileProviderReplicatedExtension)"' \
    target/framework-c-ios-file-provider-ios-tree.txt; then
    echo "out-of-scope File Provider binding feature leaked into F16" >&2
    exit 1
fi
if rg -q 'operation_cancel|NSFileProviderDomain::identifier|displayName|startAccessingSecurityScopedResource|UIDocumentPicker|NSFileCoordinator' \
    bindings/c/src/ios_file_provider.rs; then
    echo "out-of-scope File Provider API leaked into F16" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-file-provider \
    > target/framework-c-ios-file-provider-host-tree.txt
if rg -q '(^|[[:space:]])ios-file-provider v|objc2-file-provider v|FileProvider.framework' \
    target/framework-c-ios-file-provider-host-tree.txt; then
    echo "File Provider dependencies leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-file-provider
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-file-provider -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-file-provider

cat > target/framework-c-ios-file-provider-c.c <<'FIXTURE_C'
#include <stddef.h>
#include <framework_ios_file_provider.h>
#include "framework-c-ios-file-provider-manifest-asserts.h"
_Static_assert(sizeof(FrameworkIosFileProviderReady) == sizeof(void *), "callback width");
static void ready(void *context) { (void)context; }
int main(void) {
    FrameworkIosFileProviderOperation *operation = NULL;
    FrameworkIosFileProviderResultV1 result = {0};
    uint8_t is_ready = 0;
    FrameworkStatus status = framework_ios_file_provider_registered_domain_presence_start(ready, NULL, &operation);
    status |= framework_ios_file_provider_operation_poll(operation, &is_ready, &result);
    framework_owned_buffer_destroy(&result.native_error_domain);
    status |= framework_ios_file_provider_operation_destroy(&operation);
    return (int)status;
}
FIXTURE_C
cat > target/framework-c-ios-file-provider-cpp.cpp <<'FIXTURE_CPP'
#include <cstddef>
#include <framework_ios_file_provider.h>
#include "framework-c-ios-file-provider-manifest-asserts.h"
static_assert(sizeof(FrameworkIosFileProviderReady) == sizeof(void *), "callback width");
static void ready(void *context) { (void)context; }
int main() {
    FrameworkIosFileProviderOperation *operation = nullptr;
    FrameworkIosFileProviderResultV1 result{};
    uint8_t is_ready = 0;
    FrameworkStatus status = framework_ios_file_provider_registered_domain_presence_start(ready, nullptr, &operation);
    status |= framework_ios_file_provider_operation_poll(operation, &is_ready, &result);
    framework_owned_buffer_destroy(&result.native_error_domain);
    status |= framework_ios_file_provider_operation_destroy(&operation);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-ios-file-provider-c.c "$host_archive" \
    -o target/framework-c-ios-file-provider-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-ios-file-provider-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-file-provider-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_file_provider_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-file-provider-host-symbols.txt
diff -u target/framework-c-ios-file-provider-expected-symbols.txt \
    target/framework-c-ios-file-provider-host-symbols.txt
nm -u "$host_archive" 2>/dev/null > target/framework-c-ios-file-provider-host-undefined.txt
if rg -q 'FileProvider|NSFileProvider|objc_msgSend|OBJC_CLASS|UIKit|swift_|Py[A-Z_]' \
    target/framework-c-ios-file-provider-host-undefined.txt; then
    echo "Apple, Swift, or Python import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=11.0; sdk=iphoneos; clang_target=arm64-apple-ios11.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-file-provider --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-file-provider \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-file-provider --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-file-provider-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-file-provider-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-file-provider-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
            "$source" "$archive" -framework FileProvider -framework Foundation -lobjc \
            -o "$binary"
        libraries="target/framework-c-ios-file-provider-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_file_provider.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-file-provider-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-file-provider-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-file-provider-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_getClass' "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        binary_strings="target/framework-c-ios-file-provider-$language-$target-strings.txt"
        strings "$binary" > "$binary_strings"
        rg -q '^NSFileProviderManager$' "$binary_strings"
        rg -q '^getDomainsWithCompletionHandler:$' "$binary_strings"
        if rg -q 'swift_|Py[A-Z_]|UIDocumentPicker|NSFileCoordinator|startAccessingSecurityScopedResource|addDomain:|removeDomain:|NSFileProviderExtension|NSFileProviderReplicatedExtension|NSFileProviderEnumerating|UIKit' \
            "$symbols" "$binary_strings"; then
            echo "out-of-scope File Provider API, Swift, or Python import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-file-provider-$language-$target-build.txt"
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_file_provider_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-file-provider-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-file-provider-expected-symbols.txt \
        "target/framework-c-ios-file-provider-$target-archive-symbols.txt"
done

printf 'F16 checks complete; no tests or consumer binaries were executed\n'

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-media-library-status.sh
cargo fmt --package framework-c-api -- --check

jq -e '
    .optional_capabilities.ios_media_library_status as $media
    | $media.cargo_feature == "ios-media-library-status"
    and $media.header == "framework_ios_media_library_status.h"
    and $media.symbols == ["framework_ios_media_library_authorization_status"]
    and ($media.authorization_raw_values | keys | sort) == [
        "FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_AUTHORIZED",
        "FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_DENIED",
        "FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_NOT_DETERMINED",
        "FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_RESTRICTED"
    ]
    and ($media.authorization_raw_values | to_entries | map(.value) | sort) == [0, 1, 2, 3]
    and (($media.direct_imports_64_bit_ios | keys | sort) == ["c", "cpp"])
    and ($media.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0")
    and ($media.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0")
    and ($media.status_mapping.unknown_native_value | contains("unchanged"))
    and ($media.ownership.output | contains("int64_t"))
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_media_library_status.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-media-library-status-expected-symbols.txt
rg -o 'framework_ios_media_library_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_media_library_status.h | sort -u \
    > target/framework-c-media-library-status-header-symbols.txt
diff -u target/framework-c-media-library-status-expected-symbols.txt \
    target/framework-c-media-library-status-header-symbols.txt
{
    cat <<'ASSERT_HEADER'
#if defined(__cplusplus)
#define FRAMEWORK_MEDIA_LIBRARY_STATIC_ASSERT static_assert
#else
#define FRAMEWORK_MEDIA_LIBRARY_STATIC_ASSERT _Static_assert
#endif
ASSERT_HEADER
    jq -r '
        .optional_capabilities.ios_media_library_status.authorization_raw_values
        | to_entries[]
        | "FRAMEWORK_MEDIA_LIBRARY_STATIC_ASSERT(" + .key + " == " + (.value | tostring)
          + ", \"" + .key + " must match ABI manifest\");"
    ' bindings/c/abi-manifest.json
} > target/framework-c-media-library-status-manifest-asserts.h

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-media-library-status-default-tree.txt
if rg -q 'ios-media-library-status|objc2-media-player|MPMediaLibrary' \
    target/framework-c-media-library-status-default-tree.txt; then
    echo "MediaPlayer status dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-media-library-status \
    > target/framework-c-media-library-status-ios-tree.txt
rg -q 'ios-media-library-status' target/framework-c-media-library-status-ios-tree.txt
rg -q 'objc2-media-player feature "MPMediaLibrary"' \
    target/framework-c-media-library-status-ios-tree.txt
if rg -q 'objc2-media-player feature "block2"|requestAuthorization' \
    target/framework-c-media-library-status-ios-tree.txt; then
    echo "permission-request block support leaked into the MediaPlayer status feature" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-media-library-status \
    > target/framework-c-media-library-status-host-tree.txt
if rg -q 'ios-media-library-status|objc2-media-player|MediaPlayer' \
    target/framework-c-media-library-status-host-tree.txt; then
    echo "iOS MediaPlayer status dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-media-library-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-media-library-status -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-media-library-status

cat > target/framework-c-media-library-status-c.c <<'FIXTURE_C'
#include <framework_ios_media_library_status.h>
#include "framework-c-media-library-status-manifest-asserts.h"
_Static_assert(sizeof(FrameworkIosMediaLibraryAuthorizationStatus) == sizeof(int64_t), "raw status width");
int main(void) {
    FrameworkIosMediaLibraryAuthorizationStatus status = INT64_MIN;
    FrameworkStatus result = framework_ios_media_library_authorization_status(&status);
    return (int)result;
}
FIXTURE_C

cat > target/framework-c-media-library-status-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_media_library_status.h>
#include "framework-c-media-library-status-manifest-asserts.h"
static_assert(sizeof(FrameworkIosMediaLibraryAuthorizationStatus) == sizeof(int64_t), "raw status width");
int main() {
    FrameworkIosMediaLibraryAuthorizationStatus status = INT64_MIN;
    FrameworkStatus result = framework_ios_media_library_authorization_status(&status);
    return static_cast<int>(result);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-media-library-status-c.c "$host_archive" \
    -o target/framework-c-media-library-status-c-host
# These fixtures use only C ABI declarations; avoid libc++ headers at the iOS 10 compile floor.
clang++ -nostdinc++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-media-library-status-cpp.cpp "$host_archive" \
    -o target/framework-c-media-library-status-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_media_library_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-media-library-status-host-symbols.txt
diff -u target/framework-c-media-library-status-expected-symbols.txt \
    target/framework-c-media-library-status-host-symbols.txt
nm -u "$host_archive" 2>/dev/null > target/framework-c-media-library-status-host-undefined.txt
if rg -qi 'MPMediaLibrary|objc_msgSend|OBJC_CLASS|MediaPlayer' \
    target/framework-c-media-library-status-host-undefined.txt; then
    echo "MediaPlayer or Objective-C import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0; sdk=iphoneos; clang_target=arm64-apple-ios10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked -p framework-c-api \
        --no-default-features --features ios-media-library-status --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked -p framework-c-api \
        --no-default-features --features ios-media-library-status --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-media-library-status \
        --target "$target"

    archive="target/$target/release/libframework_c_api.a"
    xcrun --sdk "$sdk" clang -target "$clang_target" -std=c11 -Wall -Wextra -Werror \
        -pedantic -I bindings/c/include -I target \
        target/framework-c-media-library-status-c.c "$archive" \
        -framework Foundation -framework MediaPlayer \
        -o "target/framework-c-media-library-status-c-$target"
    xcrun --sdk "$sdk" clang++ -target "$clang_target" -nostdinc++ -std=c++17 \
        -Wall -Wextra -Werror \
        -pedantic -I bindings/c/include -I target \
        target/framework-c-media-library-status-cpp.cpp "$archive" \
        -framework Foundation -framework MediaPlayer \
        -o "target/framework-c-media-library-status-cpp-$target"

    for language in c cpp; do
        binary="target/framework-c-media-library-status-$language-$target"
        imports="target/framework-c-media-library-status-$language-$target-imports.txt"
        libraries="target/framework-c-media-library-status-$language-$target-libraries.txt"
        symbols="target/framework-c-media-library-status-$language-$target-symbols.txt"
        build_info="target/framework-c-media-library-status-$language-$target-build.txt"
        otool -L "$binary" > "$imports"
        awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_media_library_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > target/framework-c-media-library-status-$language-imports-expected.txt
        diff -u "target/framework-c-media-library-status-$language-imports-expected.txt" \
            "$libraries"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        if rg -qi 'UIKit|Security|Network|StoreKit|Swift|Python|AppKit|WebKit|UserNotifications' \
            "$symbols"; then
            echo "unrelated framework or runtime import in $binary" >&2
            exit 1
        fi
        for api_symbol in MPMediaLibrary authorizationStatus; do
            strings "$binary" | rg -q "$api_symbol"
        done
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
    done
    printf '%s C11/C++17 MediaPlayer status link imports and iOS %s minimum verified; probes not executed\n' \
        "$target" "$deployment_target"
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    archive="target/$target/release/libframework_c_api.a"
    symbols="target/framework-c-media-library-status-$target-archive-symbols.txt"
    imports="target/framework-c-media-library-status-$target-archive-imports.txt"
    nm -g "$archive" 2>/dev/null | rg -o '_framework_ios_media_library_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u > "$symbols"
    diff -u target/framework-c-media-library-status-expected-symbols.txt "$symbols"
    nm -u "$archive" 2>/dev/null > "$imports"
    rg -q '_objc_msgSend' "$imports"
    if rg -qi 'UIKit|Security|Network|StoreKit|Swift|Python|AppKit|WebKit|UserNotifications' \
        "$imports"; then
        echo "unrelated framework or runtime import in $archive" >&2
        exit 1
    fi
done

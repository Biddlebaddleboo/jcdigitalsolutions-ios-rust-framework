#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target
sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for this audit: $llvm_nm" >&2
    exit 1
fi

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-preferences.sh
cargo fmt --package framework-c-api -- --check
cargo xtask docs-check
cargo xtask zero-swift-source

jq -e '
    .optional_capabilities.ios_preferences as $preferences
    | ($preferences.symbols | sort) == [
        "framework_ios_preferences_availability",
        "framework_ios_preferences_create",
        "framework_ios_preferences_destroy",
        "framework_ios_preferences_get",
        "framework_ios_preferences_remove",
        "framework_ios_preferences_set"
    ]
    and ($preferences.availability_tags | keys | sort) == [
        "FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_AVAILABLE",
        "FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_ENTITLEMENT",
        "FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_REQUIRES_PERMISSION",
        "FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_TEMPORARILY_UNAVAILABLE",
        "FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN",
        "FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNSUPPORTED"
    ]
    and ($preferences.update_requirement_tags | keys | sort) == [
        "FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC",
        "FRAMEWORK_IOS_PREFERENCES_REQUIRE_ATOMIC"
    ]
    and ($preferences.update_atomicity_tags | keys | sort) == [
        "FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_ATOMIC",
        "FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_NOT_GUARANTEED",
        "FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN"
    ]
    and all([$preferences.availability_tags[], $preferences.update_requirement_tags[],
        $preferences.update_atomicity_tags[]][]; type == "number" and floor == .)
    and (($preferences.direct_imports_64_bit_ios | keys | sort) == ["c", "cpp"])
    and (($preferences.direct_imports_64_bit_ios.c | sort) == [
        "Foundation", "libSystem.B.dylib", "libobjc.A.dylib"
    ])
    and (($preferences.direct_imports_64_bit_ios.cpp | sort) == [
        "Foundation", "libSystem.B.dylib", "libc++.1.dylib", "libobjc.A.dylib"
    ])
    and ($preferences.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0")
    and ($preferences.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0")
    and ($preferences.ownership.get | contains("framework_owned_buffer_destroy"))
' bindings/c/abi-manifest.json > /dev/null
jq -e '
    .ownership.FrameworkOwnedBuffer.optional_capability_creators
    | index("framework_ios_preferences_get") != null
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_preferences.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-preferences-expected-symbols.txt
rg -o 'framework_ios_preferences_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_preferences.h | sort -u \
    > target/framework-c-preferences-header-symbols.txt
diff -u target/framework-c-preferences-expected-symbols.txt \
    target/framework-c-preferences-header-symbols.txt

{
    cat <<'ASSERT_HEADER'
#if defined(__cplusplus)
#define FRAMEWORK_PREFERENCES_STATIC_ASSERT static_assert
#else
#define FRAMEWORK_PREFERENCES_STATIC_ASSERT _Static_assert
#endif
ASSERT_HEADER
    jq -r '
        .optional_capabilities.ios_preferences
        | [.availability_tags, .update_requirement_tags, .update_atomicity_tags][]
        | to_entries[]
        | "FRAMEWORK_PREFERENCES_STATIC_ASSERT(" + .key + " == " + (.value | tostring)
          + ", \"" + .key + " must match ABI manifest\");"
    ' bindings/c/abi-manifest.json
} > target/framework-c-preferences-manifest-asserts.h

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-preferences-default-tree.txt
if rg -q 'framework-preferences|ios-preferences|objc2-foundation|NSUserDefaults' \
    target/framework-c-preferences-default-tree.txt; then
    echo "preferences dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-preferences \
    > target/framework-c-preferences-ios-tree.txt
for dependency in framework-preferences ios-preferences objc2-foundation; do
    rg -q "$dependency" target/framework-c-preferences-ios-tree.txt
done
cargo tree --locked -e features -i objc2-foundation -p framework-c-api \
    --target aarch64-apple-ios --no-default-features --features ios-preferences \
    > target/framework-c-preferences-ios-foundation-features.txt
for feature in NSData NSString NSUserDefaults; do
    rg -q "objc2-foundation feature \"$feature\"" \
        target/framework-c-preferences-ios-foundation-features.txt
done
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-preferences \
    > target/framework-c-preferences-host-tree.txt
if rg -q 'ios-preferences|objc2-foundation|objc2 v|objc2 feature' \
    target/framework-c-preferences-host-tree.txt; then
    echo "iOS preferences backend or Objective-C dependencies leaked into host graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-preferences
cargo clippy --locked -p framework-c-api --no-default-features --features ios-preferences -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features --features ios-preferences

cat > target/framework-c-preferences-c.c <<'FIXTURE_C'
#include <framework_ios_preferences.h>
#include "framework-c-preferences-manifest-asserts.h"
_Static_assert(sizeof(FrameworkIosPreferencesAvailability) == sizeof(uint32_t), "availability tag size");
_Static_assert(sizeof(FrameworkIosPreferencesUpdateRequirement) == sizeof(uint32_t), "requirement tag size");
_Static_assert(sizeof(FrameworkIosPreferencesUpdateAtomicity) == sizeof(uint32_t), "atomicity tag size");
int main(void) {
    FrameworkIosPreferences *preferences = NULL;
    FrameworkIosPreferencesAvailability availability = FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN;
    FrameworkIosPreferencesUpdateAtomicity atomicity = FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN;
    FrameworkOwnedBuffer value = {0};
    uint8_t found = 0;
    FrameworkStatus status = framework_ios_preferences_create(&preferences);
    status += framework_ios_preferences_availability(preferences, &availability);
    status += framework_ios_preferences_get(preferences, (FrameworkStr){NULL, 0}, &found, &value);
    status += framework_ios_preferences_set(preferences, (FrameworkStr){NULL, 0},
        (FrameworkSlice){NULL, 0}, FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC, &atomicity);
    status += framework_ios_preferences_remove(preferences, (FrameworkStr){NULL, 0}, &found);
    framework_owned_buffer_destroy(&value);
    framework_ios_preferences_destroy(&preferences);
    return (int)status;
}
FIXTURE_C

cat > target/framework-c-preferences-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_preferences.h>
#include "framework-c-preferences-manifest-asserts.h"
static_assert(sizeof(FrameworkIosPreferencesAvailability) == sizeof(uint32_t), "availability tag size");
static_assert(sizeof(FrameworkIosPreferencesUpdateRequirement) == sizeof(uint32_t), "requirement tag size");
static_assert(sizeof(FrameworkIosPreferencesUpdateAtomicity) == sizeof(uint32_t), "atomicity tag size");
int main() {
    FrameworkIosPreferences *preferences = nullptr;
    FrameworkIosPreferencesAvailability availability = FRAMEWORK_IOS_PREFERENCES_AVAILABILITY_UNKNOWN;
    FrameworkIosPreferencesUpdateAtomicity atomicity = FRAMEWORK_IOS_PREFERENCES_UPDATE_ATOMICITY_UNKNOWN;
    FrameworkOwnedBuffer value{};
    uint8_t found = 0;
    FrameworkStatus status = framework_ios_preferences_create(&preferences);
    status += framework_ios_preferences_availability(preferences, &availability);
    status += framework_ios_preferences_get(preferences, FrameworkStr{nullptr, 0}, &found, &value);
    status += framework_ios_preferences_set(preferences, FrameworkStr{nullptr, 0},
        FrameworkSlice{nullptr, 0}, FRAMEWORK_IOS_PREFERENCES_ALLOW_NON_ATOMIC, &atomicity);
    status += framework_ios_preferences_remove(preferences, FrameworkStr{nullptr, 0}, &found);
    framework_owned_buffer_destroy(&value);
    framework_ios_preferences_destroy(&preferences);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-preferences-c.c "$host_archive" -o target/framework-c-preferences-c-host
# The fixtures use only C ABI declarations; avoid libc++ headers at the iOS 10 compile floor.
clang++ -nostdinc++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-preferences-cpp.cpp "$host_archive" \
    -o target/framework-c-preferences-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_preferences_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-preferences-archive-symbols.txt
diff -u target/framework-c-preferences-expected-symbols.txt \
    target/framework-c-preferences-archive-symbols.txt

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0; sdk=iphoneos; clang_target=arm64-apple-ios10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked -p framework-c-api \
        --no-default-features --features ios-preferences --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked -p framework-c-api \
        --no-default-features --features ios-preferences --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-preferences --target "$target"

    archive="target/$target/release/libframework_c_api.a"
    xcrun --sdk "$sdk" clang -target "$clang_target" -std=c11 -Wall -Wextra -Werror \
        -pedantic -I bindings/c/include -I target target/framework-c-preferences-c.c \
        "$archive" -framework Foundation -o "target/framework-c-preferences-c-$target"
    xcrun --sdk "$sdk" clang++ -target "$clang_target" -nostdinc++ -std=c++17 \
        -Wall -Wextra -Werror \
        -pedantic -I bindings/c/include -I target target/framework-c-preferences-cpp.cpp \
        "$archive" -framework Foundation -o "target/framework-c-preferences-cpp-$target"

    for language in c cpp; do
        binary="target/framework-c-preferences-$language-$target"
        imports="target/framework-c-preferences-$language-$target-imports.txt"
        libraries="target/framework-c-preferences-$language-$target-libraries.txt"
        symbols="target/framework-c-preferences-$language-$target-symbols.txt"
        build_info="target/framework-c-preferences-$language-$target-build.txt"
        otool -L "$binary" > "$imports"
        awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_preferences.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > target/framework-c-preferences-$language-imports-expected.txt
        diff -u "target/framework-c-preferences-$language-imports-expected.txt" "$libraries"
        nm -u "$binary" 2>/dev/null > "$symbols"
        if rg -qi 'UIKit|Security|Network|StoreKit|Swift|Python|AppKit|WebKit|UserNotifications' \
            "$symbols"; then
            echo "unrelated framework or runtime import in $binary" >&2
            exit 1
        fi
        for selector in NSUserDefaults standardUserDefaults objectForKey: setObject:forKey: removeObjectForKey:; do
            strings "$binary" | rg -q "$selector"
        done
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
    done
    printf '%s C11/C++17 preferences link imports and iOS %s minimum verified; probes not executed\n' \
        "$target" "$deployment_target"
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    archive="target/$target/release/libframework_c_api.a"
    symbols="target/framework-c-preferences-$target-archive-symbols.txt"
    imports="target/framework-c-preferences-$target-archive-imports.txt"
    "$llvm_nm" -g "$archive" 2>/dev/null | rg -o '_framework_ios_preferences_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u > "$symbols"
    diff -u target/framework-c-preferences-expected-symbols.txt "$symbols"
    "$llvm_nm" -u "$archive" 2>/dev/null > "$imports"
    for selector in NSUserDefaults standardUserDefaults objectForKey: setObject:forKey: removeObjectForKey:; do
        strings "$archive" | rg -q "$selector"
    done
    if rg -qi 'UIKit|Security|Network|StoreKit|Swift|Python|AppKit|WebKit|UserNotifications' \
        "$imports"; then
        echo "unrelated framework or runtime import in $archive" >&2
        exit 1
    fi
done

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
sh -n bindings/c/check-ios-classkit-deep-link.sh
cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_classkit_deep_link as $classkit
    | $classkit.cargo_feature == "ios-classkit-deep-link"
    and $classkit.header == "framework_ios_classkit_deep_link.h"
    and $classkit.symbols == ["framework_ios_classkit_is_deep_link"]
    and $classkit.link_probe_deployment_minimums["aarch64-apple-ios"] == "11.3"
    and $classkit.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($classkit.api | contains("iOS 11.3"))
    and ($classkit.ownership.activity | contains("does not retain"))
    and ($classkit.limits | contains("assignment data"))
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_classkit_deep_link.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-ios-classkit-deep-link-expected-symbols.txt
rg -o 'framework_ios_classkit_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_classkit_deep_link.h | sort -u \
    > target/framework-c-ios-classkit-deep-link-header-symbols.txt
diff -u target/framework-c-ios-classkit-deep-link-expected-symbols.txt \
    target/framework-c-ios-classkit-deep-link-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-classkit-deep-link-default-tree.txt
if rg -q 'ios-system-services|objc2-class-kit|objc2-foundation|NSUserActivity|ClassKit' \
    target/framework-c-ios-classkit-deep-link-default-tree.txt; then
    echo "ClassKit dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-classkit-deep-link \
    > target/framework-c-ios-classkit-deep-link-ios-tree.txt
rg -q 'ios-system-services' target/framework-c-ios-classkit-deep-link-ios-tree.txt
rg -q 'objc2-class-kit feature "NSUserActivity_CLSDeepLinks"' \
    target/framework-c-ios-classkit-deep-link-ios-tree.txt
rg -q 'objc2-foundation feature "NSUserActivity"' \
    target/framework-c-ios-classkit-deep-link-ios-tree.txt
rg -q 'objc2-foundation feature "NSArray"' \
    target/framework-c-ios-classkit-deep-link-ios-tree.txt
rg -q 'objc2-foundation feature "NSString"' \
    target/framework-c-ios-classkit-deep-link-ios-tree.txt
if rg -q 'objc2-class-kit feature "(default|CLSDataStore|CLSContext|CLSActivity|block2|std)"' \
    target/framework-c-ios-classkit-deep-link-ios-tree.txt; then
    echo "ClassKit data-store, activity, or default binding feature leaked into F14" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-classkit-deep-link \
    > target/framework-c-ios-classkit-deep-link-host-tree.txt
if rg -q 'ios-system-services|objc2-class-kit|objc2-foundation|NSUserActivity|ClassKit' \
    target/framework-c-ios-classkit-deep-link-host-tree.txt; then
    echo "ClassKit dependency leaked into the non-iOS feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-classkit-deep-link
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-classkit-deep-link -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-classkit-deep-link

cat > target/framework-c-ios-classkit-deep-link-c.c <<'FIXTURE_C'
#include <framework_ios_classkit_deep_link.h>
int main(void) {
    FrameworkIosClassKitBoolean result = UINT8_MAX;
    FrameworkStatus status = framework_ios_classkit_is_deep_link((const NSUserActivity *)0, &result);
    return (int)status;
}
FIXTURE_C
cat > target/framework-c-ios-classkit-deep-link-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_classkit_deep_link.h>
static_assert(sizeof(FrameworkIosClassKitBoolean) == sizeof(uint8_t), "Boolean ABI width");
int main() {
    FrameworkIosClassKitBoolean result = UINT8_MAX;
    FrameworkStatus status = framework_ios_classkit_is_deep_link(nullptr, &result);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-classkit-deep-link-c.c "$host_archive" \
    -o target/framework-c-ios-classkit-deep-link-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-classkit-deep-link-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-classkit-deep-link-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_classkit_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-classkit-deep-link-host-symbols.txt
diff -u target/framework-c-ios-classkit-deep-link-expected-symbols.txt \
    target/framework-c-ios-classkit-deep-link-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-classkit-deep-link-host-undefined.txt
if rg -qi 'ClassKit|objc_msgSend|OBJC_CLASS|objc2|NSUserActivity' \
    target/framework-c-ios-classkit-deep-link-host-undefined.txt; then
    echo "ClassKit or Objective-C import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=11.3; sdk=iphoneos; clang_target=arm64-apple-ios11.3 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-classkit-deep-link \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-classkit-deep-link \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-classkit-deep-link \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-classkit-deep-link-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-classkit-deep-link-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-classkit-deep-link-$language-$target"
        case "$language" in
            c)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
                    -Wall -Wextra -Werror -pedantic -I bindings/c/include \
                    "$source" "$archive" -framework ClassKit -framework Foundation -lobjc \
                    -o "$binary"
                ;;
            cpp)
                # The C++ fixture uses only C ABI declarations; avoid libc++ headers at Xcode 27 iOS floors.
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -nostdinc++ \
                    -std="$standard" -Wall -Wextra -Werror -pedantic -I bindings/c/include \
                    "$source" "$archive" -framework ClassKit -framework Foundation -lobjc \
                    -o "$binary"
                ;;
        esac
        libraries="target/framework-c-ios-classkit-deep-link-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_classkit_deep_link.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-classkit-deep-link-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-classkit-deep-link-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-classkit-deep-link-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        strings_file="target/framework-c-ios-classkit-deep-link-$language-$target-strings.txt"
        strings "$binary" > "$strings_file"
        rg -q 'isClassKitDeepLink' "$strings_file"
        if rg -qi 'CLSDataStore|CLSContext|CLSActivity|contextIdentifierPath|swift_|Py[A-Z_]' "$symbols" "$strings_file"; then
            echo "out-of-scope ClassKit data, path, Swift, or Python symbol/string in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-classkit-deep-link-$language-$target-build.txt"
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_classkit_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-classkit-deep-link-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-classkit-deep-link-expected-symbols.txt \
        "target/framework-c-ios-classkit-deep-link-$target-archive-symbols.txt"
done

printf 'F14 checks complete; no tests or consumer binaries were executed\n'

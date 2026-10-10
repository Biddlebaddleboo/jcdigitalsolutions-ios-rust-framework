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
sh -n bindings/c/check-ios-vision.sh
cargo fmt --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_vision as $vision
    | $vision.cargo_feature == "ios-vision"
    and $vision.header == "framework_ios_vision.h"
    and $vision.symbols == ["framework_ios_vision_text_recognition_revision_is_supported"]
    and ($vision.api | contains("iOS 13.0"))
    and $vision.link_probe_deployment_minimums["aarch64-apple-ios"] == "13.0"
    and $vision.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($vision.status_mapping.success | contains("0 or 1"))
    and ($vision.ownership.output | contains("aligned writable uint8_t memory"))
    and ($vision.ownership.output | contains("full synchronous call"))
    and ($vision.ownership.output | contains("unsynchronized access"))
    and ($vision.ownership.output | contains("not retained"))
' bindings/c/abi-manifest.json > /dev/null

rg -Fq 'valid, properly aligned writable `uint8_t` memory' \
    bindings/c/src/ios_vision.rs bindings/c/include/framework_ios_vision.h
rg -Fq 'full synchronous call' \
    bindings/c/src/ios_vision.rs bindings/c/include/framework_ios_vision.h
rg -Fq 'unsynchronized access' \
    bindings/c/src/ios_vision.rs bindings/c/include/framework_ios_vision.h
rg -Fq 'does not retain the output address' \
    bindings/c/src/ios_vision.rs bindings/c/include/framework_ios_vision.h
rg -Fq 'if out_supported.is_null()' bindings/c/src/ios_vision.rs
rg -Fq 'null output' bindings/c/include/framework_ios_vision.h

jq -r '.optional_capabilities.ios_vision.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-ios-vision-expected-symbols.txt
rg -o 'framework_ios_vision_[A-Za-z0-9_]+' bindings/c/include/framework_ios_vision.h \
    | sort -u > target/framework-c-ios-vision-header-symbols.txt
diff -u target/framework-c-ios-vision-expected-symbols.txt \
    target/framework-c-ios-vision-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-vision-default-tree.txt
if rg -q 'ios-vision|objc2-vision|Vision.framework' \
    target/framework-c-ios-vision-default-tree.txt; then
    echo "Vision dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-vision \
    > target/framework-c-ios-vision-ios-tree.txt
rg -q 'ios-vision v' target/framework-c-ios-vision-ios-tree.txt
rg -q 'objc2-vision feature "VNRequest"' target/framework-c-ios-vision-ios-tree.txt
rg -q 'objc2-vision feature "VNRecognizeTextRequest"' target/framework-c-ios-vision-ios-tree.txt
rg -q 'objc2-foundation feature "NSIndexSet"' target/framework-c-ios-vision-ios-tree.txt
if rg -q 'objc2-vision feature "(default|VNImageRequestHandler|VNRecognizedTextObservation)"' \
    target/framework-c-ios-vision-ios-tree.txt; then
    echo "out-of-scope Vision feature leaked into F17" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-vision \
    > target/framework-c-ios-vision-host-tree.txt
if rg -q 'ios-vision v|objc2-vision v|Vision.framework' \
    target/framework-c-ios-vision-host-tree.txt; then
    echo "iOS Vision dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-vision
cargo clippy --locked -p framework-c-api --no-default-features --features ios-vision -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features --features ios-vision

cat > target/framework-c-ios-vision-c.c <<'FIXTURE_C'
#include <framework_ios_vision.h>
int main(void) {
    uint8_t supported = 1;
    return (int)framework_ios_vision_text_recognition_revision_is_supported(1, &supported);
}
FIXTURE_C
cat > target/framework-c-ios-vision-cpp.cpp <<'FIXTURE_CPP'
#include <stddef.h>
#include <framework_ios_vision.h>
int main() {
    uint8_t supported = 1;
    return static_cast<int>(framework_ios_vision_text_recognition_revision_is_supported(1, &supported));
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-vision-c.c "$host_archive" -o target/framework-c-ios-vision-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-vision-cpp.cpp "$host_archive" -o target/framework-c-ios-vision-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_vision_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-vision-host-symbols.txt
diff -u target/framework-c-ios-vision-expected-symbols.txt \
    target/framework-c-ios-vision-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null > target/framework-c-ios-vision-host-undefined.txt
if rg -q 'VNRecognizeTextRequest|supportedRevisions|objc_msgSend|OBJC_CLASS|Vision' \
    target/framework-c-ios-vision-host-undefined.txt; then
    echo "Vision or Objective-C import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=13.0; sdk=iphoneos; clang_target=arm64-apple-ios13.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked -p framework-c-api \
        --no-default-features --features ios-vision --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked -p framework-c-api \
        --no-default-features --features ios-vision --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-vision --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-vision-c.c; standard=c11; cpp_only_flags= ;;
            cpp) compiler=clang++; source=target/framework-c-ios-vision-cpp.cpp; standard=c++17; cpp_only_flags=-nostdinc++ ;;
        esac
        binary="target/framework-c-ios-vision-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            $cpp_only_flags -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework Vision -framework Foundation -lobjc -o "$binary"
        libraries="target/framework-c-ios-vision-$language-$target-libraries.txt"
        otool -L "$binary" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_vision.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-vision-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-vision-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-vision-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        binary_strings="target/framework-c-ios-vision-$language-$target-strings.txt"
        strings "$binary" > "$binary_strings"
        rg -q '^VNRecognizeTextRequest$' "$binary_strings"
        rg -q '^supportedRevisions$' "$binary_strings"
        rg -q '^containsIndex:$' "$binary_strings"
        if rg -q 'VNImageRequestHandler|VNRecognizedTextObservation|UIKit|VisionKit|CoreML|AVFoundation|swift_|Py[A-Z_]' \
            "$symbols" "$binary_strings"; then
            echo "out-of-scope Vision API or runtime import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-vision-$language-$target-build.txt"
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null | rg -o '_framework_ios_vision_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u > "target/framework-c-ios-vision-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-vision-expected-symbols.txt \
        "target/framework-c-ios-vision-$target-archive-symbols.txt"
done

printf 'F17 Vision checks complete; no tests or consumer binaries were executed\n'

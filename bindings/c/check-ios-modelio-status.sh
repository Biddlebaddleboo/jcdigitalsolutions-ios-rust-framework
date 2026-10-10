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
sh -n bindings/c/check-ios-modelio-status.sh
cargo fmt --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_modelio_status as $modelio
    | $modelio.cargo_feature == "ios-modelio-status"
    and $modelio.header == "framework_ios_modelio_status.h"
    and $modelio.symbols == ["framework_ios_modelio_can_import_file_extension"]
    and ($modelio.api | contains("iOS 9.0"))
    and $modelio.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $modelio.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($modelio.status_mapping.success | contains("0 or 1"))
    and ($modelio.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($modelio.ownership.input | contains("FrameworkStr"))
    and ($modelio.ownership.output | contains("aligned writable"))
    and ($modelio.ownership.output | contains("full synchronous call"))
    and ($modelio.ownership.output | contains("unsynchronized access"))
    and ($modelio.ownership.output | contains("range arithmetic"))
    and ($modelio.ownership.output | contains("input/output overlap"))
    and ($modelio.ownership.output | contains("cannot prove that memory"))
    and ($modelio.ownership.output | contains("neither pointer is retained"))
    and ($modelio.direct_imports_64_bit_ios.c | sort) == [
        "Foundation", "ModelIO", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
    and ($modelio.direct_imports_64_bit_ios.cpp | sort) == [
        "Foundation", "ModelIO", "libSystem.B.dylib", "libc++.1.dylib", "libobjc.A.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

rg -Fq 'valid, properly aligned writable memory' \
    bindings/c/src/ios_modelio_status.rs
rg -Fq 'if out_supported.is_null()' bindings/c/src/ios_modelio_status.rs
rg -Fq 'A null' bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'output, malformed span metadata' bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'input_output_overlap' bindings/c/src/ios_modelio_status.rs
rg -Fq 'Some(false)' bindings/c/src/ios_modelio_status.rs
rg -Fq 'valid, properly aligned' bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'writable memory for one byte' bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'full synchronous call' \
    bindings/c/src/ios_modelio_status.rs bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'unsynchronized access' \
    bindings/c/src/ios_modelio_status.rs bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'range arithmetic' \
    bindings/c/src/ios_modelio_status.rs bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'input/output' \
    bindings/c/src/ios_modelio_status.rs bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'overlap' \
    bindings/c/src/ios_modelio_status.rs bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'cannot prove that memory' \
    bindings/c/src/ios_modelio_status.rs bindings/c/include/framework_ios_modelio_status.h
rg -Fq 'Neither pointer is' bindings/c/src/ios_modelio_status.rs
rg -Fq 'retained.' bindings/c/src/ios_modelio_status.rs
rg -Fq 'retains neither pointer' bindings/c/include/framework_ios_modelio_status.h

jq -r '.optional_capabilities.ios_modelio_status.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-ios-modelio-status-expected-symbols.txt
rg -o 'framework_ios_modelio_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_modelio_status.h | sort -u \
    > target/framework-c-ios-modelio-status-header-symbols.txt
diff -u target/framework-c-ios-modelio-status-expected-symbols.txt \
    target/framework-c-ios-modelio-status-header-symbols.txt
rg -q '#\[cfg\(not\(target_os = "ios"\)\)\]' \
    bindings/c/src/ios_modelio_status.rs
rg -q 'FrameworkStatus::UNSUPPORTED' bindings/c/src/ios_modelio_status.rs

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-modelio-status-default-tree.txt
if rg -q '(^|[[:space:]])ios-modelio-status v|objc2-model-io v|ModelIO.framework' \
    target/framework-c-ios-modelio-status-default-tree.txt; then
    echo "ModelIO dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-modelio-status \
    > target/framework-c-ios-modelio-status-ios-tree.txt
rg -q 'ios-modelio-status v' target/framework-c-ios-modelio-status-ios-tree.txt
rg -q 'objc2-model-io feature "MDLAsset"' \
    target/framework-c-ios-modelio-status-ios-tree.txt
rg -q 'objc2-foundation feature "NSString"' \
    target/framework-c-ios-modelio-status-ios-tree.txt
if rg -q 'objc2-model-io feature "default"|objc2-model-io feature "MDLMesh"' \
    target/framework-c-ios-modelio-status-ios-tree.txt; then
    echo "out-of-scope ModelIO binding feature leaked into F20" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-modelio-status \
    > target/framework-c-ios-modelio-status-host-tree.txt
if rg -q '(^|[[:space:]])ios-modelio-status v|objc2-model-io v|objc2-foundation v|ModelIO.framework' \
    target/framework-c-ios-modelio-status-host-tree.txt; then
    echo "iOS ModelIO or Foundation dependency leaked into the host graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-modelio-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-modelio-status -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-modelio-status

cat > target/framework-c-ios-modelio-status-c.c <<'FIXTURE_C'
#include <framework_ios_modelio_status.h>
int main(void) {
    static const uint8_t extension_bytes[] = {'u', 's', 'd', 'z'};
    FrameworkStr extension = {extension_bytes, sizeof(extension_bytes)};
    uint8_t supported = 0;
    return (int)framework_ios_modelio_can_import_file_extension(extension, &supported);
}
FIXTURE_C
cat > target/framework-c-ios-modelio-status-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_modelio_status.h>
int main() {
    static const uint8_t extension_bytes[] = {'u', 's', 'd', 'z'};
    FrameworkStr extension{extension_bytes, sizeof(extension_bytes)};
    uint8_t supported = 0;
    return static_cast<int>(framework_ios_modelio_can_import_file_extension(extension, &supported));
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-modelio-status-c.c "$host_archive" \
    -o target/framework-c-ios-modelio-status-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-modelio-status-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-modelio-status-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_modelio_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-modelio-status-host-symbols.txt
diff -u target/framework-c-ios-modelio-status-expected-symbols.txt \
    target/framework-c-ios-modelio-status-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-modelio-status-host-undefined.txt
if rg -q 'MDLAsset|ModelIO|objc_msgSend|OBJC_CLASS|libobjc|swift_' \
    target/framework-c-ios-modelio-status-host-undefined.txt; then
    echo "ModelIO, Objective-C, or Swift import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=10.0
            sdk=iphoneos
            clang_target=arm64-apple-ios10.0
            rust_min_flag=-miphoneos-version-min=10.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            rust_min_flag=-mios-simulator-version-min=14.0
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-modelio-status \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-modelio-status \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-modelio-status \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-modelio-status-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-modelio-status-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-modelio-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework Foundation -framework ModelIO -lobjc -o "$binary"
        libraries="target/framework-c-ios-modelio-status-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_modelio_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-modelio-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-modelio-status-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-modelio-status-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        binary_strings="target/framework-c-ios-modelio-status-$language-$target-strings.txt"
        strings "$binary" > "$binary_strings"
        rg -q '^canImportFileExtension:$' "$binary_strings"
        if rg -q 'swift_|Py[A-Z_]|Security|Metal|SceneKit|MDLMesh|MDLTexture|MTLDevice' \
            "$symbols" "$binary_strings"; then
            echo "out-of-scope framework, asset, or runtime import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-modelio-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s ModelIO imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_modelio_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-modelio-status-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-modelio-status-expected-symbols.txt \
        "target/framework-c-ios-modelio-status-$target-archive-symbols.txt"
done

printf 'F20 checks complete; no tests or consumer binaries were executed\n'

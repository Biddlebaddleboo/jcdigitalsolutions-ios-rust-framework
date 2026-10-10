#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

rustc_sysroot=$(rustc --print sysroot)
rustc_host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$rustc_sysroot/lib/rustlib/$rustc_host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for Rust archive scans: $llvm_nm" >&2
    exit 1
fi

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-accelerate.sh
cargo fmt --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_accelerate as $accelerate
    | $accelerate.cargo_feature == "ios-accelerate"
    and $accelerate.header == "framework_ios_accelerate.h"
    and $accelerate.symbols == ["framework_ios_accelerate_vector_add"]
    and ($accelerate.api | contains("iOS 4.0"))
    and $accelerate.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $accelerate.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($accelerate.status_mapping.length_mismatch | contains("INVALID_ARGUMENT"))
    and ($accelerate.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($accelerate.ownership.inputs | contains("ranges may overlap"))
    and ($accelerate.ownership.inputs | contains("neither pointer is retained"))
    and ($accelerate.ownership.inputs | contains("full synchronous call"))
    and ($accelerate.ownership.inputs | contains("unsynchronized input access"))
    and ($accelerate.ownership.output | contains("borrowed writable f32 array"))
    and ($accelerate.ownership.output | contains("full synchronous call"))
    and ($accelerate.ownership.output | contains("unsynchronized output access"))
    and ($accelerate.ownership.output | contains("caller-owned and not retained"))
    and ($accelerate.ownership.output | contains("disjoint from both inputs"))
    and ($accelerate.ownership.output | contains("overwritten"))
    and ($accelerate.direct_imports_64_bit_ios.c | sort) == [
        "Accelerate", "libSystem.B.dylib"
    ]
    and ($accelerate.direct_imports_64_bit_ios.cpp | sort) == [
        "Accelerate", "libSystem.B.dylib", "libc++.1.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_accelerate.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-ios-accelerate-expected-symbols.txt
rg -o 'framework_ios_accelerate_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_accelerate.h | sort -u \
    > target/framework-c-ios-accelerate-header-symbols.txt
diff -u target/framework-c-ios-accelerate-expected-symbols.txt \
    target/framework-c-ios-accelerate-header-symbols.txt
rg -q '#\[cfg\(not\(target_os = "ios"\)\)\]' \
    bindings/c/src/ios_accelerate.rs
rg -q 'FrameworkStatus::UNSUPPORTED' bindings/c/src/ios_accelerate.rs
for file in bindings/c/src/ios_accelerate.rs \
    bindings/c/include/framework_ios_accelerate.h \
    docs/bindings/ios-accelerate.md; do
    rg -F -q 'keep both inputs immutable' "$file"
    rg -F -q 'unsynchronized output access' "$file"
    rg -F -q 'for the full call' "$file"
    rg -F -q 'cannot prove that' "$file"
    rg -F -q 'memory is valid, readable, writable, or live' "$file"
    rg -F -q 'pointer is retained after return' "$file"
done
rg -F -q 'length.checked_mul(size_of::<f32>())' bindings/c/src/ios_accelerate.rs
rg -F -q 'byte_length > isize::MAX as usize' bindings/c/src/ios_accelerate.rs
rg -F -q 'start.checked_add(byte_length)?' bindings/c/src/ios_accelerate.rs
rg -F -q 'ranges_overlap(output_range, a_range) || ranges_overlap(output_range, b_range)' \
    bindings/c/src/ios_accelerate.rs
rg -F -q 'if length != 0 && (pointer.is_null() || start % align_of::<f32>() != 0)' \
    bindings/c/src/ios_accelerate.rs
if rg -q 'ranges_overlap\((a_range, b_range|b_range, a_range)\)' \
    bindings/c/src/ios_accelerate.rs; then
    echo "F22 input span overlap is no longer accepted" >&2
    exit 1
fi
awk '
    /if a_length != b_length \|\| a_length != output_length/ { count_check = NR }
    /length\.checked_mul\(size_of::<f32>\(\)\)/ { byte_length = NR }
    /if byte_length > isize::MAX as usize/ { slice_limit = NR }
    /let Some\(a_range\) = span_range\(a, length, byte_length\)/ { a_range = NR }
    /let Some\(b_range\) = span_range\(b, length, byte_length\)/ { b_range = NR }
    /let Some\(output_range\) = span_range\(output\.cast_const\(\), length, byte_length\)/ { output_range = NR }
    /if ranges_overlap\(output_range, a_range\)/ { output_overlap = NR }
    /let a = unsafe \{ slice::from_raw_parts\(a, length\) \}/ { input_slice = NR }
    END {
        if (!(count_check < byte_length && byte_length < slice_limit &&
              slice_limit < a_range && a_range < b_range && b_range < output_range &&
              output_range < output_overlap && output_overlap < input_slice)) {
            exit 1
        }
    }
' bindings/c/src/ios_accelerate.rs || {
    echo "F22 span validation and alias check must precede Rust slice creation" >&2
    exit 1
}

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-accelerate-default-tree.txt
if rg -q '(^|[[:space:]])ios-accelerate v|Accelerate.framework' \
    target/framework-c-ios-accelerate-default-tree.txt; then
    echo "Accelerate dependency leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-accelerate \
    > target/framework-c-ios-accelerate-ios-tree.txt
rg -q 'ios-accelerate v' target/framework-c-ios-accelerate-ios-tree.txt
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-accelerate \
    > target/framework-c-ios-accelerate-host-tree.txt
if rg -q '(^|[[:space:]])ios-accelerate v|Accelerate.framework' \
    target/framework-c-ios-accelerate-host-tree.txt; then
    echo "iOS Accelerate dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-accelerate
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-accelerate -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-accelerate

cat > target/framework-c-ios-accelerate-c.c <<'FIXTURE_C'
#include <framework_ios_accelerate.h>
_Static_assert(sizeof(float) == sizeof(uint32_t), "f32 C ABI");
int main(void) {
    const float a[2] = {1.0f, 2.0f};
    const float b[2] = {3.0f, 4.0f};
    float output[2] = {0.0f, 0.0f};
    return (int)framework_ios_accelerate_vector_add(a, 2, b, 2, output, 2);
}
FIXTURE_C
cat > target/framework-c-ios-accelerate-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_accelerate.h>
static_assert(sizeof(float) == sizeof(uint32_t), "f32 C ABI");
int main() {
    const float a[2] = {1.0f, 2.0f};
    const float b[2] = {3.0f, 4.0f};
    float output[2] = {0.0f, 0.0f};
    return static_cast<int>(framework_ios_accelerate_vector_add(a, 2, b, 2, output, 2));
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-accelerate-c.c "$host_archive" \
    -o target/framework-c-ios-accelerate-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-accelerate-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-accelerate-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_accelerate_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-accelerate-host-symbols.txt
diff -u target/framework-c-ios-accelerate-expected-symbols.txt \
    target/framework-c-ios-accelerate-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-accelerate-host-undefined.txt
if rg -q 'vDSP_vadd|Accelerate|UIKit|Metal|objc|swift_' \
    target/framework-c-ios-accelerate-host-undefined.txt; then
    echo "Accelerate, Apple framework, Objective-C, or Swift import leaked into host archive" >&2
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
        -p framework-c-api --no-default-features --features ios-accelerate \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-accelerate \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-accelerate --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-accelerate-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-accelerate-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-accelerate-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework Accelerate -o "$binary"
        libraries="target/framework-c-ios-accelerate-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_accelerate.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-accelerate-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-accelerate-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-accelerate-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_vDSP_vadd' "$symbols"
        if rg -q 'Foundation|UIKit|Metal|objc|swift_' "$libraries" "$symbols"; then
            echo "out-of-scope framework or runtime import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-accelerate-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s Accelerate/_vDSP_vadd imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_accelerate_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-accelerate-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-accelerate-expected-symbols.txt \
        "target/framework-c-ios-accelerate-$target-archive-symbols.txt"
done

printf 'F22 checks complete; no tests or consumer binaries were executed\n'

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
sh -n bindings/c/check-ios-spritekit.sh
cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_spritekit as $spritekit
    | $spritekit.cargo_feature == "ios-spritekit"
    and $spritekit.header == "framework_ios_spritekit.h"
    and ($spritekit.symbols | sort) == [
        "framework_ios_spritekit_node_create",
        "framework_ios_spritekit_node_destroy",
        "framework_ios_spritekit_node_get_position",
        "framework_ios_spritekit_node_set_position"
    ]
    and ($spritekit.direct_imports_64_bit_ios.c | sort) == [
        "CoreFoundation", "Foundation", "SpriteKit", "UIKit", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
    and ($spritekit.direct_imports_64_bit_ios.cpp | sort) == [
        "CoreFoundation", "Foundation", "SpriteKit", "UIKit", "libSystem.B.dylib", "libc++.1.dylib", "libobjc.A.dylib"
    ]
    and $spritekit.link_probe_deployment_minimums["aarch64-apple-ios"] == "12.0"
    and $spritekit.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($spritekit.status_mapping.off_main_thread | contains("does not read or change"))
    and ($spritekit.status_mapping.non_ios | contains("destroy does not inspect or change its slot"))
    and ($spritekit.ownership.handle | contains("unique"))
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_spritekit.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-spritekit-expected-symbols.txt
rg -o 'framework_ios_spritekit_node_[A-Za-z0-9_]+' bindings/c/include/framework_ios_spritekit.h \
    | sort -u > target/framework-c-spritekit-header-symbols.txt
diff -u target/framework-c-spritekit-expected-symbols.txt target/framework-c-spritekit-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-spritekit-default-tree.txt
if rg -q 'framework-spritekit|ios-spritekit|ios-runtime' target/framework-c-spritekit-default-tree.txt; then
    echo "SpriteKit dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-spritekit > target/framework-c-spritekit-ios-tree.txt
for dependency in framework-spritekit ios-spritekit ios-runtime; do
    rg -q "$dependency" target/framework-c-spritekit-ios-tree.txt
done
if rg -q 'framework-sharing|ios-sharing|ios-transfer|ios-preferences|ios-media-library-status' \
    target/framework-c-spritekit-ios-tree.txt; then
    echo "unrelated capability dependency leaked into the SpriteKit feature graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-spritekit > target/framework-c-spritekit-host-tree.txt
if rg -q 'framework-spritekit|ios-spritekit|ios-runtime|SpriteKit|UIKit|objc2' \
    target/framework-c-spritekit-host-tree.txt; then
    echo "iOS SpriteKit dependency leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-spritekit
cargo clippy --locked -p framework-c-api --no-default-features --features ios-spritekit -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features --features ios-spritekit

cat > target/framework-c-spritekit-c.c <<'FIXTURE_C'
#include <framework_ios_spritekit.h>
_Static_assert(sizeof(double) == 8, "SpriteKit coordinates use 64-bit double");
int main(void) {
    FrameworkIosSpriteKitNode *node = NULL;
    double x = 0.0;
    double y = 0.0;
    FrameworkStatus status = framework_ios_spritekit_node_create(1.0, -2.0, &node);
    status += framework_ios_spritekit_node_get_position(node, &x, &y);
    status += framework_ios_spritekit_node_set_position(node, x, y);
    status += framework_ios_spritekit_node_destroy(&node);
    return (int)status;
}
FIXTURE_C
cat > target/framework-c-spritekit-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_spritekit.h>
static_assert(sizeof(double) == 8, "SpriteKit coordinates use 64-bit double");
int main() {
    FrameworkIosSpriteKitNode *node = nullptr;
    double x = 0.0;
    double y = 0.0;
    FrameworkStatus status = framework_ios_spritekit_node_create(1.0, -2.0, &node);
    status += framework_ios_spritekit_node_get_position(node, &x, &y);
    status += framework_ios_spritekit_node_set_position(node, x, y);
    status += framework_ios_spritekit_node_destroy(&node);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-spritekit-c.c "$host_archive" -o target/framework-c-spritekit-c-host
# These fixtures use only C ABI declarations; avoid libc++ headers at the iOS 12 compile floor.
clang++ -nostdinc++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-spritekit-cpp.cpp "$host_archive" -o target/framework-c-spritekit-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_spritekit_node_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-spritekit-host-symbols.txt
diff -u target/framework-c-spritekit-expected-symbols.txt target/framework-c-spritekit-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null > target/framework-c-spritekit-host-undefined.txt
if rg -qi 'SpriteKit|UIKit|SKNode|objc_msgSend|OBJC_CLASS|objc2' target/framework-c-spritekit-host-undefined.txt; then
    echo "SpriteKit or Objective-C import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0; sdk=iphoneos; clang_target=arm64-apple-ios12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-spritekit --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-spritekit --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-spritekit --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-spritekit-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-spritekit-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-spritekit-$language-$target"
        case "$language" in
            c)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
                    -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
                    -framework CoreFoundation -framework Foundation -framework SpriteKit \
                    -framework UIKit -lobjc -o "$binary"
                ;;
            cpp)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -nostdinc++ \
                    -std="$standard" -Wall -Wextra -Werror -pedantic -I bindings/c/include \
                    "$source" "$archive" -framework CoreFoundation -framework Foundation \
                    -framework SpriteKit -framework UIKit -lobjc -o "$binary"
                ;;
        esac
        libraries="target/framework-c-spritekit-$language-$target-libraries.txt"
        otool -L "$binary" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_spritekit.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-spritekit-$language-imports-expected.txt"
        diff -u "target/framework-c-spritekit-$language-imports-expected.txt" "$libraries"
        nm -u "$binary" 2>/dev/null > "target/framework-c-spritekit-$language-$target-undefined.txt"
        rg -q '_objc_msgSend' "target/framework-c-spritekit-$language-$target-undefined.txt"
        strings "$binary" > "target/framework-c-spritekit-$language-$target-strings.txt"
        for selector in SKNode position setPosition:; do
            rg -q "$selector" "target/framework-c-spritekit-$language-$target-strings.txt"
        done
        if rg -qi 'swift|SceneKit|ModelIO|MetalPerformanceShaders|Security|CommonCrypto|SKScene|SKView|SKRenderer' \
            "target/framework-c-spritekit-$language-$target-undefined.txt" \
            "target/framework-c-spritekit-$language-$target-strings.txt"; then
            echo "unexpected runtime, framework, scene, view, or renderer import in $binary" >&2
            exit 1
        fi
        vtool -show-build "$binary" > "target/framework-c-spritekit-$language-$target-build.txt"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' \
            "target/framework-c-spritekit-$language-$target-build.txt")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_spritekit_node_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
        > "target/framework-c-spritekit-$target-archive-symbols.txt"
    diff -u target/framework-c-spritekit-expected-symbols.txt \
        "target/framework-c-spritekit-$target-archive-symbols.txt"
done

printf 'F11 checks complete; no tests or consumer binaries were executed\n'

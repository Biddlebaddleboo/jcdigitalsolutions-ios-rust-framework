#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

jq -e '
    .optional_capabilities.ios_clipboard.availability_tags as $tags
    | ($tags | keys | sort) == [
        "FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_AVAILABLE",
        "FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_ENTITLEMENT",
        "FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_PERMISSION",
        "FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_TEMPORARILY_UNAVAILABLE",
        "FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN",
        "FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED"
    ]
    and all($tags[]; type == "number" and floor == .)
' bindings/c/abi-manifest.json > /dev/null
jq -e '
    .optional_capabilities.ios_clipboard.ownership as $clipboard
    | ($clipboard | has("client") and has("read") and has("inputs"))
    and all([$clipboard.client, $clipboard.read, $clipboard.inputs][];
        type == "string" and length > 0)
    and .ownership.FrameworkOwnedBuffer.creator_in_core_f1 == true
    and .ownership.FrameworkOwnedBuffer.core_creator == "framework_owned_buffer_copy"
    and .ownership.FrameworkOwnedBuffer.destroyer == "framework_owned_buffer_destroy"
    and (.ownership.FrameworkOwnedBuffer.rule | type == "string" and length > 0)
' bindings/c/abi-manifest.json > /dev/null
{
    cat <<'TAG_HEADER'
#if defined(__cplusplus)
#define FRAMEWORK_CLIPBOARD_STATIC_ASSERT static_assert
#else
#define FRAMEWORK_CLIPBOARD_STATIC_ASSERT _Static_assert
#endif
TAG_HEADER
    jq -r '
        .optional_capabilities.ios_clipboard.availability_tags
        | to_entries[]
        | "#define FRAMEWORK_C_MANIFEST_" + .key + " " + (.value | tostring)
          + "\nFRAMEWORK_CLIPBOARD_STATIC_ASSERT(" + .key
          + " == FRAMEWORK_C_MANIFEST_" + .key
          + ", \"" + .key + " must match ABI manifest\");"
    ' bindings/c/abi-manifest.json
} > target/framework-c-clipboard-tags.h

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios --no-default-features \
    > target/framework-c-clipboard-default-tree.txt
if rg -q 'framework-sharing|ios-sharing|objc2|block2|UIActivityViewController' \
    target/framework-c-clipboard-default-tree.txt; then
    echo "clipboard dependencies leaked into the default C ABI build" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-clipboard > target/framework-c-clipboard-ios-tree.txt
for dependency in framework-sharing ios-sharing ios-runtime; do
    rg -q "$dependency" target/framework-c-clipboard-ios-tree.txt
done
cargo tree --locked -e features -i ios-sharing -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-clipboard > target/framework-c-clipboard-ios-features.txt
rg -q 'ios-sharing feature "clipboard"' target/framework-c-clipboard-ios-features.txt
if rg -q 'ios-sharing feature "share"|block2|UIActivityViewController|objc2-core-foundation' \
    target/framework-c-clipboard-ios-features.txt; then
    echo "share-only dependencies leaked into the C clipboard feature graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-clipboard > target/framework-c-clipboard-host-tree.txt
if rg -q 'ios-sharing|objc2-ui-kit|block2|UIActivityViewController' \
    target/framework-c-clipboard-host-tree.txt; then
    echo "iOS clipboard dependencies leaked into a non-iOS feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-clipboard
cargo build --locked --release -p framework-c-api --no-default-features --features ios-clipboard

cat > target/framework-c-clipboard-c.c <<'FIXTURE_C'
#include <framework_ios_clipboard.h>
#include "framework-c-clipboard-tags.h"
_Static_assert(sizeof(FrameworkIosClipboardAvailability) == sizeof(uint32_t), "availability tag size");
int main(void) {
    FrameworkIosClipboard *clipboard = NULL;
    FrameworkIosClipboardAvailability availability = FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN;
    uint8_t has_value = 0;
    FrameworkOwnedBuffer text = {0};
    int32_t native_code = 0;
    FrameworkStatus status = framework_ios_clipboard_create(&clipboard);
    status += framework_ios_clipboard_availability(clipboard, &availability);
    status += framework_ios_clipboard_read(clipboard, &has_value, &text, &native_code);
    status += framework_ios_clipboard_write(clipboard, (FrameworkStr){0}, &native_code);
    status += framework_ios_clipboard_clear(clipboard, &native_code);
    framework_owned_buffer_destroy(&text);
    framework_ios_clipboard_destroy(&clipboard);
    return (int)status;
}
FIXTURE_C

cat > target/framework-c-clipboard-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_clipboard.h>
#include "framework-c-clipboard-tags.h"
static_assert(sizeof(FrameworkIosClipboardAvailability) == sizeof(uint32_t), "availability tag size");
int main() {
    FrameworkIosClipboard *clipboard = nullptr;
    FrameworkIosClipboardAvailability availability = FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN;
    uint8_t has_value = 0;
    FrameworkOwnedBuffer text{};
    int32_t native_code = 0;
    FrameworkStatus status = framework_ios_clipboard_create(&clipboard);
    status += framework_ios_clipboard_availability(clipboard, &availability);
    status += framework_ios_clipboard_read(clipboard, &has_value, &text, &native_code);
    status += framework_ios_clipboard_write(clipboard, FrameworkStr{nullptr, 0}, &native_code);
    status += framework_ios_clipboard_clear(clipboard, &native_code);
    framework_owned_buffer_destroy(&text);
    framework_ios_clipboard_destroy(&clipboard);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -I target \
    target/framework-c-clipboard-c.c "$host_archive" -o target/framework-c-clipboard-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -I target \
    target/framework-c-clipboard-cpp.cpp "$host_archive" -o target/framework-c-clipboard-cpp-host
nm -gU "$host_archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u \
    > target/framework-c-clipboard-symbols.txt
jq -r '.c_symbols[], .optional_capabilities.ios_clipboard.symbols[]' \
    bindings/c/abi-manifest.json | sort -u > target/framework-c-clipboard-expected-symbols.txt
diff -u target/framework-c-clipboard-expected-symbols.txt target/framework-c-clipboard-symbols.txt

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p framework-c-api --no-default-features --features ios-clipboard --target "$target"
    cargo clippy --locked -p framework-c-api --no-default-features --features ios-clipboard \
        --all-targets --target "$target" -- -D warnings
    cargo build --locked --release -p framework-c-api --no-default-features \
        --features ios-clipboard --target "$target"
done

device_archive=target/aarch64-apple-ios/release/libframework_c_api.a
simulator_archive=target/aarch64-apple-ios-sim/release/libframework_c_api.a
if ! command -v xcrun >/dev/null 2>&1; then
    echo "xcrun is required for the iOS C link checks" >&2
    exit 1
fi
xcrun --sdk iphoneos clang -arch arm64 -miphoneos-version-min=17.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-clipboard-c.c \
    "$device_archive" -framework UIKit -framework Foundation -o target/framework-c-clipboard-c-device
xcrun --sdk iphoneos clang++ -arch arm64 -miphoneos-version-min=17.0 -std=c++17 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-clipboard-cpp.cpp \
    "$device_archive" -framework UIKit -framework Foundation -o target/framework-c-clipboard-cpp-device
xcrun --sdk iphonesimulator clang -arch arm64 -mios-simulator-version-min=17.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-clipboard-c.c \
    "$simulator_archive" -framework UIKit -framework Foundation -o target/framework-c-clipboard-c-simulator
xcrun --sdk iphonesimulator clang++ -arch arm64 -mios-simulator-version-min=17.0 -std=c++17 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-clipboard-cpp.cpp \
    "$simulator_archive" -framework UIKit -framework Foundation -o target/framework-c-clipboard-cpp-simulator

if ! command -v otool >/dev/null 2>&1; then
    echo "otool is required for the iOS clipboard link/import check" >&2
    exit 1
fi
cat > target/framework-c-clipboard-c-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS
cat > target/framework-c-clipboard-cpp-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
libSystem.B.dylib
libc++.1.dylib
libobjc.A.dylib
IMPORTS
check_probe_imports() {
    binary=$1
    expected=$2
    label=$3
    imports="target/framework-c-clipboard-$label-imports.txt"
    symbols="target/framework-c-clipboard-$label-symbols.txt"
    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "target/framework-c-clipboard-$label-libraries.txt"
    diff -u "$expected" "target/framework-c-clipboard-$label-libraries.txt"
    nm -u "$binary" 2>/dev/null > "$symbols"
    if rg -qi 'UIActivityViewController|block2|swift|python|AppKit|WebKit|UserNotifications|CoreLocation|CoreMotion|StoreKit|SecItem|nw_[A-Za-z0-9_]*|SCNetwork' "$symbols"; then
        echo "unexpected share, runtime, or unrelated capability symbol in $symbols" >&2
        exit 1
    fi
}
check_probe_imports target/framework-c-clipboard-c-device \
    target/framework-c-clipboard-c-imports-expected.txt ios-c-device
check_probe_imports target/framework-c-clipboard-c-simulator \
    target/framework-c-clipboard-c-imports-expected.txt ios-c-simulator
check_probe_imports target/framework-c-clipboard-cpp-device \
    target/framework-c-clipboard-cpp-imports-expected.txt ios-cpp-device
check_probe_imports target/framework-c-clipboard-cpp-simulator \
    target/framework-c-clipboard-cpp-imports-expected.txt ios-cpp-simulator

for archive in "$device_archive" "$simulator_archive"; do
    imports=target/framework-c-clipboard-imports.txt
    nm -u "$archive" 2>/dev/null > "$imports"
    if rg -qi 'UIActivityViewController|block2|swift|python|AppKit|WebKit|UserNotifications|CoreLocation|CoreMotion|StoreKit' "$imports"; then
        echo "unexpected share, runtime, or unrelated framework import in $archive" >&2
        exit 1
    fi
done
for symbol in framework_ios_clipboard_create framework_ios_clipboard_destroy \
    framework_ios_clipboard_availability framework_ios_clipboard_read \
    framework_ios_clipboard_write framework_ios_clipboard_clear; do
    rg -q "$symbol" bindings/c/include/framework_ios_clipboard.h
done

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
sh -n bindings/c/check-ios-share.sh
cargo fmt --all -- --check
cargo xtask docs-check
cargo xtask zero-swift-source

jq -e '
    .optional_capabilities.ios_share as $share
    | ($share.symbols | sort) == [
        "framework_ios_share_cancel",
        "framework_ios_share_session_availability",
        "framework_ios_share_session_create",
        "framework_ios_share_session_destroy",
        "framework_ios_share_start"
    ]
    and ($share.item_tags | keys | sort) == [
        "FRAMEWORK_IOS_SHARE_ITEM_TEXT",
        "FRAMEWORK_IOS_SHARE_ITEM_URL"
    ]
    and ($share.outcome_tags | keys | sort) == [
        "FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED",
        "FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED"
    ]
    and ($share.availability_tags | keys | sort) == [
        "FRAMEWORK_IOS_SHARE_AVAILABILITY_AVAILABLE",
        "FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_ENTITLEMENT",
        "FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_PERMISSION",
        "FRAMEWORK_IOS_SHARE_AVAILABILITY_TEMPORARILY_UNAVAILABLE",
        "FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN",
        "FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED"
    ]
    and all([$share.item_tags[], $share.outcome_tags[], $share.availability_tags[]][];
        type == "number" and floor == .)
    and ($share.callback | has("success") and has("error") and has("lifetime") and has("rules"))
    and all([
        $share.callback.success,
        $share.callback.error,
        $share.callback.lifetime,
        $share.callback.rules
    ][]; type == "string" and length > 0)
    and ($share.status_mapping | has("accepted_start") and has("already_active") and has("cancel_inactive")
        and has("preflight_error") and has("share_error")
        and has("unknown_non_exhaustive_value") and has("off_main_thread")
        and has("unknown_availability") and has("non_ios"))
    and all([
        $share.status_mapping.accepted_start,
        $share.status_mapping.already_active,
        $share.status_mapping.cancel_inactive,
        $share.status_mapping.preflight_error,
        $share.status_mapping.share_error,
        $share.status_mapping.unknown_non_exhaustive_value,
        $share.status_mapping.off_main_thread,
        $share.status_mapping.unknown_availability,
        $share.status_mapping.non_ios
    ][]; type == "string" and length > 0)
    and ($share.ownership | has("session") and has("context") and has("request")
        and has("cancel") and has("destroy"))
    and all([
        $share.ownership.session,
        $share.ownership.context,
        $share.ownership.request,
        $share.ownership.cancel,
        $share.ownership.destroy
    ][]; type == "string" and length > 0)
    and (($share.direct_imports_64_bit_ios | keys | sort) == ["c", "cpp"])
    and (($share.direct_imports_64_bit_ios.c | sort) == [
        "Foundation", "UIKit", "libSystem.B.dylib", "libobjc.A.dylib"
    ])
    and (($share.direct_imports_64_bit_ios.cpp | sort) == [
        "Foundation", "UIKit", "libSystem.B.dylib", "libc++.1.dylib", "libobjc.A.dylib"
    ])
' bindings/c/abi-manifest.json > /dev/null
jq -e '
    .optional_capabilities.ios_share.layouts_64_bit_ios as $layouts
    | ($layouts | keys | sort) == [
        "FrameworkIosShareAnchorV1",
        "FrameworkIosShareItemV1",
        "FrameworkIosShareRequestV1"
    ]
    and ($layouts.FrameworkIosShareItemV1.fields | keys | sort) == ["kind", "reserved", "text"]
    and ($layouts.FrameworkIosShareRequestV1.fields | keys | sort) == [
        "abi_version", "item_count", "items", "struct_size"
    ]
    and ($layouts.FrameworkIosShareAnchorV1.fields | keys | sort) == [
        "abi_version", "height", "struct_size", "width", "x", "y"
    ]
    and all([
        $layouts.FrameworkIosShareItemV1.size,
        $layouts.FrameworkIosShareItemV1.align,
        $layouts.FrameworkIosShareItemV1.fields.kind,
        $layouts.FrameworkIosShareItemV1.fields.reserved,
        $layouts.FrameworkIosShareItemV1.fields.text,
        $layouts.FrameworkIosShareRequestV1.size,
        $layouts.FrameworkIosShareRequestV1.align,
        $layouts.FrameworkIosShareRequestV1.fields.struct_size,
        $layouts.FrameworkIosShareRequestV1.fields.abi_version,
        $layouts.FrameworkIosShareRequestV1.fields.items,
        $layouts.FrameworkIosShareRequestV1.fields.item_count,
        $layouts.FrameworkIosShareAnchorV1.size,
        $layouts.FrameworkIosShareAnchorV1.align,
        $layouts.FrameworkIosShareAnchorV1.fields.struct_size,
        $layouts.FrameworkIosShareAnchorV1.fields.abi_version,
        $layouts.FrameworkIosShareAnchorV1.fields.x,
        $layouts.FrameworkIosShareAnchorV1.fields.y,
        $layouts.FrameworkIosShareAnchorV1.fields.width,
        $layouts.FrameworkIosShareAnchorV1.fields.height
    ][]; type == "number" and floor == .)
' bindings/c/abi-manifest.json > /dev/null
jq -r '.optional_capabilities.ios_share.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-share-expected-symbols.txt
for probe_kind in c cpp; do
    jq -r --arg kind "$probe_kind" \
        '.optional_capabilities.ios_share.direct_imports_64_bit_ios[$kind][]' \
        bindings/c/abi-manifest.json | LC_ALL=C sort \
        > "target/framework-c-share-$probe_kind-imports-expected.txt"
done
{
    cat <<'ASSERT_HEADER'
#include <stddef.h>
#if defined(__cplusplus)
#define FRAMEWORK_SHARE_STATIC_ASSERT static_assert
#define FRAMEWORK_SHARE_ALIGNOF alignof
#else
#define FRAMEWORK_SHARE_STATIC_ASSERT _Static_assert
#define FRAMEWORK_SHARE_ALIGNOF _Alignof
#endif
ASSERT_HEADER
    jq -r '
        .optional_capabilities.ios_share
        | [.item_tags, .outcome_tags, .availability_tags][]
        | to_entries[]
        | "#define FRAMEWORK_C_MANIFEST_" + .key + " " + (.value | tostring)
          + "\nFRAMEWORK_SHARE_STATIC_ASSERT(" + .key
          + " == FRAMEWORK_C_MANIFEST_" + .key
          + ", \"" + .key + " must match ABI manifest\");"
    ' bindings/c/abi-manifest.json
    jq -r '
        .optional_capabilities.ios_share.layouts_64_bit_ios
        | to_entries[]
        | .key as $type
        | .value as $layout
        | "FRAMEWORK_SHARE_STATIC_ASSERT(sizeof(" + $type + ") == "
          + ($layout.size | tostring) + ", \"" + $type + " size mismatch\");",
          "FRAMEWORK_SHARE_STATIC_ASSERT(FRAMEWORK_SHARE_ALIGNOF(" + $type + ") == "
          + ($layout.align | tostring) + ", \"" + $type + " alignment mismatch\");"
    ' bindings/c/abi-manifest.json
    jq -r '
        .optional_capabilities.ios_share.layouts_64_bit_ios
        | to_entries[] as $record
        | $record.key as $type
        | $record.value.fields
        | to_entries[]
        | .key as $field
        | .value as $offset
        | "FRAMEWORK_SHARE_STATIC_ASSERT(offsetof(" + $type + ", " + $field + ") == "
          + ($offset | tostring) + ", \"" + $type + "." + $field + " offset mismatch\");"
    ' bindings/c/abi-manifest.json
} > target/framework-c-share-manifest-asserts.h
rg -o 'framework_ios_share_[A-Za-z0-9_]+' bindings/c/include/framework_ios_share.h \
    | sort -u > target/framework-c-share-header-symbols.txt
diff -u target/framework-c-share-expected-symbols.txt target/framework-c-share-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-share-default-tree.txt
if rg -q 'framework-sharing|ios-sharing|objc2-ui-kit|block2|dispatch2|UIActivityViewController' \
    target/framework-c-share-default-tree.txt; then
    echo "share dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-share > target/framework-c-share-ios-tree.txt
for dependency in framework-sharing ios-sharing ios-runtime objc2-core-foundation objc2-ui-kit block2 dispatch2; do
    rg -q "$dependency" target/framework-c-share-ios-tree.txt
done
cargo tree --locked -e features -i ios-sharing -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-share > target/framework-c-share-ios-features.txt
rg -q 'ios-sharing feature "share"' target/framework-c-share-ios-features.txt
if rg -q 'ios-sharing feature "clipboard"|UIPasteboard' \
    target/framework-c-share-ios-features.txt; then
    echo "clipboard features leaked into the C share graph" >&2
    exit 1
fi
cargo tree --locked -e features -i objc2-ui-kit -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-share > target/framework-c-share-ui-kit-features.txt
rg -q 'objc2-ui-kit feature "UIActivityViewController"' \
    target/framework-c-share-ui-kit-features.txt
if rg -q 'objc2-ui-kit feature "UIPasteboard"' target/framework-c-share-ui-kit-features.txt; then
    echo "UIPasteboard feature leaked into the C share graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-share > target/framework-c-share-host-tree.txt
if rg -q 'ios-sharing|objc2-ui-kit|objc2-core-foundation|block2|dispatch2|UIActivityViewController' \
    target/framework-c-share-host-tree.txt; then
    echo "iOS share dependencies leaked into a non-iOS feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features --features ios-share
cargo clippy --locked -p framework-c-api --no-default-features --features ios-share -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features --features ios-share

cat > target/framework-c-share-c.c <<'FIXTURE_C'
#include <stddef.h>
#include <framework_ios_share.h>
#include "framework-c-share-manifest-asserts.h"
static void completed(void *context, FrameworkStatus status, FrameworkIosShareOutcome outcome, int32_t native_code) {
    (void)context; (void)status; (void)outcome; (void)native_code;
}
int main(void) {
    FrameworkIosShareAnchorV1 anchor = {sizeof(FrameworkIosShareAnchorV1), 1, 0, 0, 0, 0};
    FrameworkIosShareItemV1 item = {FRAMEWORK_IOS_SHARE_ITEM_TEXT, 0, {NULL, 0}};
    FrameworkIosShareRequestV1 request = {sizeof(FrameworkIosShareRequestV1), 1, &item, 1};
    FrameworkIosShareSession *session = NULL;
    FrameworkIosShareAvailability availability = FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN;
    FrameworkStatus status = framework_ios_share_session_create(NULL, NULL, anchor, &session);
    status += framework_ios_share_session_availability(session, &availability);
    status += framework_ios_share_start(session, &request, completed, NULL);
    status += framework_ios_share_cancel(session);
    status += framework_ios_share_session_destroy(&session);
    return (int)status;
}
FIXTURE_C
cat > target/framework-c-share-cpp.cpp <<'FIXTURE_CPP'
#include <cstddef>
#include <framework_ios_share.h>
#include "framework-c-share-manifest-asserts.h"
static void completed(void *context, FrameworkStatus status, FrameworkIosShareOutcome outcome, int32_t native_code) {
    (void)context; (void)status; (void)outcome; (void)native_code;
}
int main() {
    FrameworkIosShareAnchorV1 anchor{sizeof(FrameworkIosShareAnchorV1), 1, 0, 0, 0, 0};
    FrameworkIosShareItemV1 item{FRAMEWORK_IOS_SHARE_ITEM_URL, 0, {nullptr, 0}};
    FrameworkIosShareRequestV1 request{sizeof(FrameworkIosShareRequestV1), 1, &item, 1};
    FrameworkIosShareSession *session = nullptr;
    FrameworkIosShareAvailability availability = FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN;
    FrameworkStatus status = framework_ios_share_session_create(nullptr, nullptr, anchor, &session);
    status += framework_ios_share_session_availability(session, &availability);
    status += framework_ios_share_start(session, &request, completed, nullptr);
    status += framework_ios_share_cancel(session);
    status += framework_ios_share_session_destroy(&session);
    return static_cast<int>(status);
}
FIXTURE_CPP

cc -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-share-c.c target/release/libframework_c_api.a \
    -o target/framework-c-share-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-share-cpp.cpp target/release/libframework_c_api.a \
    -o target/framework-c-share-cpp-host

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p framework-c-api --no-default-features --features ios-share --target "$target"
    cargo clippy --locked -p framework-c-api --no-default-features --features ios-share \
        --target "$target" -- -D warnings
    cargo build --locked --release -p framework-c-api --no-default-features \
        --features ios-share --target "$target"
done

if ! command -v xcrun > /dev/null 2>&1; then
    echo "xcrun is required for the iOS C link checks" >&2
    exit 1
fi
device_archive=target/aarch64-apple-ios/release/libframework_c_api.a
simulator_archive=target/aarch64-apple-ios-sim/release/libframework_c_api.a
xcrun --sdk iphoneos clang -arch arm64 -miphoneos-version-min=17.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-share-c.c \
    "$device_archive" -framework UIKit -framework Foundation -o target/framework-c-share-c-device
xcrun --sdk iphoneos clang++ -arch arm64 -miphoneos-version-min=17.0 -std=c++17 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-share-cpp.cpp \
    "$device_archive" -framework UIKit -framework Foundation -o target/framework-c-share-cpp-device
xcrun --sdk iphonesimulator clang -arch arm64 -mios-simulator-version-min=17.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-share-c.c \
    "$simulator_archive" -framework UIKit -framework Foundation -o target/framework-c-share-c-simulator
xcrun --sdk iphonesimulator clang++ -arch arm64 -mios-simulator-version-min=17.0 -std=c++17 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target target/framework-c-share-cpp.cpp \
    "$simulator_archive" -framework UIKit -framework Foundation \
    -o target/framework-c-share-cpp-simulator

if ! command -v otool > /dev/null 2>&1; then
    echo "otool is required for the iOS share link/import check" >&2
    exit 1
fi
check_probe() {
    binary=$1
    expected=$2
    label=$3
    imports="target/framework-c-share-$label-imports.txt"
    libraries="target/framework-c-share-$label-libraries.txt"
    symbols="target/framework-c-share-$label-symbols.txt"
    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "$libraries"
    diff -u "$expected" "$libraries"
    nm -u "$binary" 2> /dev/null > "$symbols"
    if rg -qi 'UIPasteboard|swift|python|AppKit|WebKit|UserNotifications|CoreLocation|CoreMotion|StoreKit|Security' "$symbols"; then
        echo "clipboard, runtime, or unrelated framework symbol in $symbols" >&2
        exit 1
    fi
}
check_probe target/framework-c-share-c-device \
    target/framework-c-share-c-imports-expected.txt c-device
check_probe target/framework-c-share-c-simulator \
    target/framework-c-share-c-imports-expected.txt c-simulator
check_probe target/framework-c-share-cpp-device \
    target/framework-c-share-cpp-imports-expected.txt cpp-device
check_probe target/framework-c-share-cpp-simulator \
    target/framework-c-share-cpp-imports-expected.txt cpp-simulator

for archive in "$device_archive" "$simulator_archive"; do
    imports=target/framework-c-share-imports.txt
    symbols=target/framework-c-share-symbols.txt
    "$llvm_nm" -u "$archive" 2> /dev/null > "$imports"
    "$llvm_nm" -g "$archive" 2> /dev/null > "$symbols"
    rg -o 'framework_ios_share_[A-Za-z0-9_]+' "$symbols" | sort -u \
        > target/framework-c-share-archive-symbols.txt
    diff -u target/framework-c-share-expected-symbols.txt target/framework-c-share-archive-symbols.txt
    if rg -qi 'UIPasteboard|swift|python|AppKit|WebKit|UserNotifications|CoreLocation|CoreMotion|StoreKit|Security' "$imports"; then
        echo "clipboard, runtime, or unrelated framework import in $archive" >&2
        exit 1
    fi
    rg -qi 'UIActivityViewController' "$imports"
done

diff -u target/framework-c-share-expected-symbols.txt target/framework-c-share-header-symbols.txt

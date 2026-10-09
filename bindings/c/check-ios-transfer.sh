#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

if ! command -v otool >/dev/null 2>&1; then
    echo "otool is required for the iOS transfer link/import check" >&2
    exit 1
fi

cargo tree --locked -p framework-c-api --target aarch64-apple-ios --no-default-features \
    > target/framework-c-transfer-default-tree.txt
if rg -q 'framework-transfer|ios-transfer|objc2|framework-network|framework-files' \
    target/framework-c-transfer-default-tree.txt; then
    echo "transfer dependencies leaked into the default C ABI build" >&2
    exit 1
fi

cargo tree --locked -p framework-c-api --target aarch64-apple-ios --features ios-transfer \
    > target/framework-c-transfer-ios-tree.txt
for dependency in framework-files framework-network framework-transfer ios-transfer objc2; do
    rg -q "$dependency" target/framework-c-transfer-ios-tree.txt
done
cargo tree --locked -p framework-c-api --target x86_64-apple-darwin --features ios-transfer \
    > target/framework-c-transfer-host-tree.txt
if rg -q 'ios-transfer|objc2|framework-network|framework-files|framework-transfer' \
    target/framework-c-transfer-host-tree.txt; then
    echo "iOS-only transfer dependencies leaked into a non-iOS feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --features ios-transfer
cargo build --locked --release -p framework-c-api --features ios-transfer

jq -e '
    .optional_capabilities.ios_transfer as $transfer
    | ($transfer.availability_tags | type == "object" and length > 0 and all(.[]; type == "number" and floor == .))
    and ($transfer.directory_tags | type == "object" and length > 0 and all(.[]; type == "number" and floor == .))
    and ($transfer.state_tags | type == "object" and length > 0 and all(.[]; type == "number" and floor == .))
    and ($transfer.error_kind_tags | type == "object" and length > 0 and all(.[]; type == "number" and floor == .))
    and ($transfer.layouts_64_bit_ios | type == "object"
        and has("FrameworkTransferIdV1")
        and has("FrameworkTransferHeaderV1")
        and has("FrameworkTransferRequestV1")
        and has("FrameworkTransferSnapshotViewV1")
        and has("FrameworkTransferHeaderViewV1")
        and all(.[]; (.size | type == "number" and floor == .)
            and (.align | type == "number" and floor == .)
            and (.fields | type == "object" and length > 0
                and all(.[]; type == "number" and floor == .))))
    and ($transfer.ownership as $ownership
        | ($ownership.client | type == "string" and length > 0)
        and ($ownership.snapshot | type == "string" and length > 0)
        and ($ownership.events | type == "string" and length > 0)
        and ($ownership.ordinary_launch | type == "string" and length > 0))
' bindings/c/abi-manifest.json > /dev/null

# Check ownership field structure; compare each claim with the plan, source, header, and guide by hand
cat > target/framework-c-transfer-manifest-asserts.h <<'ASSERTS'
#include <stddef.h>
#include <stdint.h>
#include <framework_ios_transfer.h>

#ifdef __cplusplus
#define FRAMEWORK_C_TRANSFER_MANIFEST_ASSERT(condition, message) static_assert(condition, message)
#define FRAMEWORK_C_TRANSFER_MANIFEST_ALIGNOF(type) alignof(type)
#else
#define FRAMEWORK_C_TRANSFER_MANIFEST_ASSERT(condition, message) _Static_assert(condition, message)
#define FRAMEWORK_C_TRANSFER_MANIFEST_ALIGNOF(type) _Alignof(type)
#endif
ASSERTS
jq -r '
    .optional_capabilities.ios_transfer as $transfer
    | (
        ([$transfer.availability_tags, $transfer.directory_tags, $transfer.state_tags, $transfer.error_kind_tags]
            | add | to_entries[]
            | "FRAMEWORK_C_TRANSFER_MANIFEST_ASSERT(\(.key) == UINT32_C(\(.value)), \"manifest tag \(.key)\");"),
        ($transfer.layouts_64_bit_ios
            | to_entries[] as $record
            | "FRAMEWORK_C_TRANSFER_MANIFEST_ASSERT(sizeof(\($record.key)) == \($record.value.size), \"manifest size \($record.key)\");",
              "FRAMEWORK_C_TRANSFER_MANIFEST_ASSERT(FRAMEWORK_C_TRANSFER_MANIFEST_ALIGNOF(\($record.key)) == \($record.value.align), \"manifest alignment \($record.key)\");",
              ($record.value.fields | to_entries[] as $field
                | "FRAMEWORK_C_TRANSFER_MANIFEST_ASSERT(offsetof(\($record.key), \($field.key)) == \($field.value), \"manifest offset \($record.key).\($field.key)\");"))
      )
' bindings/c/abi-manifest.json >> target/framework-c-transfer-manifest-asserts.h

cat > target/framework-c-transfer-c.c <<'FIXTURE_C'
#include <stddef.h>
#include <framework_ios_transfer.h>
#include "framework-c-transfer-manifest-asserts.h"

_Static_assert(sizeof(FrameworkTransferIdV1) == 16, "transfer ID size");
_Static_assert(_Alignof(FrameworkTransferIdV1) == 8, "transfer ID alignment");
_Static_assert(offsetof(FrameworkTransferIdV1, high) == 0, "transfer ID high offset");
_Static_assert(offsetof(FrameworkTransferIdV1, low) == 8, "transfer ID low offset");
_Static_assert(sizeof(FrameworkTransferHeaderV1) == 32, "request header size");
_Static_assert(_Alignof(FrameworkTransferHeaderV1) == 8, "request header alignment");
_Static_assert(offsetof(FrameworkTransferHeaderV1, name) == 0, "request header name offset");
_Static_assert(offsetof(FrameworkTransferHeaderV1, value) == 16, "request header value offset");
_Static_assert(sizeof(FrameworkTransferRequestV1) == 80, "request size");
_Static_assert(_Alignof(FrameworkTransferRequestV1) == 8, "request alignment");
_Static_assert(offsetof(FrameworkTransferRequestV1, struct_size) == 0, "request size field offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, abi_version) == 4, "request ABI version offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, id) == 8, "request ID offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, url) == 24, "request URL offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, headers) == 40, "request headers offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, header_count) == 48, "request header count offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, directory) == 56, "request directory offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, reserved) == 60, "request reserved offset");
_Static_assert(offsetof(FrameworkTransferRequestV1, relative_path) == 64, "request path offset");
_Static_assert(sizeof(FrameworkTransferSnapshotViewV1) == 56, "snapshot view size");
_Static_assert(_Alignof(FrameworkTransferSnapshotViewV1) == 8, "snapshot view alignment");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, struct_size) == 0, "view size field offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, abi_version) == 4, "view ABI version offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, id) == 8, "view ID offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, state) == 24, "view state offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, failure_kind) == 28, "view failure kind offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, native_code) == 32, "view native code offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, http_status) == 36, "view HTTP status offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, reserved) == 40, "view reserved offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, reserved2) == 44, "view reserved2 offset");
_Static_assert(offsetof(FrameworkTransferSnapshotViewV1, response_header_count) == 48, "header count offset");
_Static_assert(sizeof(FrameworkTransferHeaderViewV1) == 32, "response header view size");
_Static_assert(_Alignof(FrameworkTransferHeaderViewV1) == 8, "response header view alignment");
_Static_assert(offsetof(FrameworkTransferHeaderViewV1, name) == 0, "response header name offset");
_Static_assert(offsetof(FrameworkTransferHeaderViewV1, value) == 16, "response header value offset");

int main(void) {
    return (int)framework_ios_transfer_finish_launch_without_background_events(NULL, NULL);
}
FIXTURE_C

cat > target/framework-c-transfer-cpp.cpp <<'FIXTURE_CPP'
#include <cstddef>
#include <framework_ios_transfer.h>
#include "framework-c-transfer-manifest-asserts.h"

static_assert(sizeof(FrameworkTransferIdV1) == 16, "transfer ID size");
static_assert(alignof(FrameworkTransferIdV1) == 8, "transfer ID alignment");
static_assert(offsetof(FrameworkTransferIdV1, high) == 0, "transfer ID high offset");
static_assert(offsetof(FrameworkTransferIdV1, low) == 8, "transfer ID low offset");
static_assert(sizeof(FrameworkTransferHeaderV1) == 32, "request header size");
static_assert(alignof(FrameworkTransferHeaderV1) == 8, "request header alignment");
static_assert(offsetof(FrameworkTransferHeaderV1, name) == 0, "request header name offset");
static_assert(offsetof(FrameworkTransferHeaderV1, value) == 16, "request header value offset");
static_assert(sizeof(FrameworkTransferRequestV1) == 80, "request size");
static_assert(alignof(FrameworkTransferRequestV1) == 8, "request alignment");
static_assert(offsetof(FrameworkTransferRequestV1, struct_size) == 0, "request size field offset");
static_assert(offsetof(FrameworkTransferRequestV1, abi_version) == 4, "request ABI version offset");
static_assert(offsetof(FrameworkTransferRequestV1, id) == 8, "request ID offset");
static_assert(offsetof(FrameworkTransferRequestV1, url) == 24, "request URL offset");
static_assert(offsetof(FrameworkTransferRequestV1, headers) == 40, "request headers offset");
static_assert(offsetof(FrameworkTransferRequestV1, header_count) == 48, "request header count offset");
static_assert(offsetof(FrameworkTransferRequestV1, directory) == 56, "request directory offset");
static_assert(offsetof(FrameworkTransferRequestV1, reserved) == 60, "request reserved offset");
static_assert(offsetof(FrameworkTransferRequestV1, relative_path) == 64, "request path offset");
static_assert(sizeof(FrameworkTransferSnapshotViewV1) == 56, "snapshot view size");
static_assert(alignof(FrameworkTransferSnapshotViewV1) == 8, "snapshot view alignment");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, struct_size) == 0, "view size field offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, abi_version) == 4, "view ABI version offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, id) == 8, "view ID offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, state) == 24, "view state offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, failure_kind) == 28, "view failure kind offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, native_code) == 32, "view native code offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, http_status) == 36, "view HTTP status offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, reserved) == 40, "view reserved offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, reserved2) == 44, "view reserved2 offset");
static_assert(offsetof(FrameworkTransferSnapshotViewV1, response_header_count) == 48, "header count offset");
static_assert(sizeof(FrameworkTransferHeaderViewV1) == 32, "response header view size");
static_assert(alignof(FrameworkTransferHeaderViewV1) == 8, "response header view alignment");
static_assert(offsetof(FrameworkTransferHeaderViewV1, name) == 0, "response header name offset");
static_assert(offsetof(FrameworkTransferHeaderViewV1, value) == 16, "response header value offset");

int main() {
    return static_cast<int>(framework_ios_transfer_finish_launch_without_background_events(nullptr, nullptr));
}
FIXTURE_CPP

cat > target/framework-c-transfer-link-probe.c <<'PROBE_C'
#include <framework_ios_transfer.h>

static void transfer_events_complete(void *context) {
    (void)context;
}

int main(void) {
    int32_t native_code = 0;
    FrameworkIosTransferClient *client = NULL;
    FrameworkStr no_session = {NULL, 0};
    FrameworkStatus create_status = framework_ios_transfer_client_create(
        no_session, &client, &native_code);
    FrameworkStatus event_status = framework_ios_transfer_forward_background_events(
        client, no_session, transfer_events_complete, NULL, &native_code);
    FrameworkStatus launch_status = framework_ios_transfer_finish_launch_without_background_events(
        client, &native_code);
    return (int)(create_status | event_status | launch_status);
}
PROBE_C

archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-transfer-c.c "$archive" -o target/framework-c-transfer-c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-transfer-cpp.cpp "$archive" -o target/framework-c-transfer-cpp

nm -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u \
    > target/framework-c-transfer-symbols.txt
jq -r '.c_symbols[], .optional_capabilities.ios_transfer.symbols[]' \
    bindings/c/abi-manifest.json | sort -u > target/framework-c-transfer-expected-symbols.txt
diff -u target/framework-c-transfer-expected-symbols.txt target/framework-c-transfer-symbols.txt

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p framework-c-api --features ios-transfer --target "$target"
    cargo clippy --locked -p framework-c-api --features ios-transfer \
        --target "$target" -- -D warnings
    cargo build --locked --release -p framework-c-api --features ios-transfer --target "$target"
done

device_archive=target/aarch64-apple-ios/release/libframework_c_api.a
simulator_archive=target/aarch64-apple-ios-sim/release/libframework_c_api.a
if ! command -v xcrun >/dev/null 2>&1; then
    echo "xcrun is required for the iOS C link checks" >&2
    exit 1
fi
xcrun --sdk iphoneos clang -arch arm64 -miphoneos-version-min=17.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include target/framework-c-transfer-c.c \
    "$device_archive" -framework Foundation \
    -o target/framework-c-transfer-c-device
xcrun --sdk iphoneos clang++ -arch arm64 -miphoneos-version-min=17.0 -std=c++17 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include target/framework-c-transfer-cpp.cpp \
    "$device_archive" -framework Foundation \
    -o target/framework-c-transfer-cpp-device
xcrun --sdk iphonesimulator clang -arch arm64 -mios-simulator-version-min=17.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include target/framework-c-transfer-c.c \
    "$simulator_archive" -framework Foundation \
    -o target/framework-c-transfer-c-simulator
xcrun --sdk iphonesimulator clang++ -arch arm64 -mios-simulator-version-min=17.0 -std=c++17 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include target/framework-c-transfer-cpp.cpp \
    "$simulator_archive" -framework Foundation \
    -o target/framework-c-transfer-cpp-simulator

xcrun --sdk iphoneos clang -arch arm64 -miphoneos-version-min=10.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include target/framework-c-transfer-link-probe.c \
    "$device_archive" -framework Foundation -Wl,-dead_strip \
    -o target/framework-c-transfer-link-probe-device
xcrun --sdk iphonesimulator clang -arch arm64 -mios-simulator-version-min=10.0 -std=c11 \
    -Wall -Wextra -Werror -pedantic -I bindings/c/include target/framework-c-transfer-link-probe.c \
    "$simulator_archive" -framework Foundation -Wl,-dead_strip \
    -o target/framework-c-transfer-link-probe-simulator

cat > target/framework-c-transfer-c-imports-expected.txt <<'IMPORTS'
CoreFoundation
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS
cat > target/framework-c-transfer-cpp-imports-expected.txt <<'IMPORTS'
CoreFoundation
Foundation
libSystem.B.dylib
libc++.1.dylib
libobjc.A.dylib
IMPORTS

check_probe_imports() {
    binary=$1
    expected=$2
    label=$3
    imports="target/framework-c-transfer-$label-imports.txt"
    libraries="target/framework-c-transfer-$label-libraries.txt"
    symbols="target/framework-c-transfer-$label-symbols.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u "$expected" "$libraries"

    nm -u "$binary" > "$symbols"
    rg -q 'dispatch_async' "$symbols"
    rg -q 'dispatch_main_q' "$symbols"
    rg -q 'objc_msgSend' "$symbols"
    if rg -qi 'swift|python|UIKit|AppKit|WebKit|UserNotifications|CoreLocation|Security|CFNetwork|nw_[A-Za-z0-9_]*|SCNetwork|SecItem|StoreKit|AVFoundation|CoreData|MapKit' "$symbols"; then
        echo "unexpected Swift, Python, or unrelated import in $symbols" >&2
        exit 1
    fi
}

check_probe_imports target/framework-c-transfer-link-probe-device \
    target/framework-c-transfer-c-imports-expected.txt ios-transfer-c-device
check_probe_imports target/framework-c-transfer-link-probe-simulator \
    target/framework-c-transfer-c-imports-expected.txt ios-transfer-c-simulator
check_probe_imports target/framework-c-transfer-c-device \
    target/framework-c-transfer-c-imports-expected.txt ios-transfer-c-fixture-device
check_probe_imports target/framework-c-transfer-c-simulator \
    target/framework-c-transfer-c-imports-expected.txt ios-transfer-c-fixture-simulator
check_probe_imports target/framework-c-transfer-cpp-device \
    target/framework-c-transfer-cpp-imports-expected.txt ios-transfer-cpp-device
check_probe_imports target/framework-c-transfer-cpp-simulator \
    target/framework-c-transfer-cpp-imports-expected.txt ios-transfer-cpp-simulator

jq -r '.optional_capabilities.ios_transfer.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-transfer-header-expected-symbols.txt
rg -o 'framework_ios_transfer_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_transfer.h | sort -u \
    > target/framework-c-transfer-header-symbols.txt
diff -u target/framework-c-transfer-header-expected-symbols.txt \
    target/framework-c-transfer-header-symbols.txt

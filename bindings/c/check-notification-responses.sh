#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

jq -e '
    .optional_capabilities.notification_responses as $response
    | ($response.symbols | sort) == [
        "framework_notification_response_create",
        "framework_notification_response_destroy",
        "framework_notification_response_get_view"
    ]
    and ($response.response_kinds | keys | sort) == [
        "FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION",
        "FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT",
        "FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS",
        "FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT"
    ]
    and ($response.response_kinds.FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT == 0)
    and ($response.response_kinds.FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS == 1)
    and ($response.response_kinds.FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION == 2)
    and ($response.response_kinds.FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT == 3)
    and all($response.response_kinds[]; type == "number" and floor == .)
    and ($response.FrameworkNotificationResponseViewV1.size == 56)
    and ($response.FrameworkNotificationResponseViewV1.align == 8)
    and (($response.FrameworkNotificationResponseViewV1.fields | keys | sort) == [
        "action_id", "kind", "notification_id", "reserved", "user_text"
    ])
    and ($response.FrameworkNotificationResponseViewV1.fields.kind == 0)
    and ($response.FrameworkNotificationResponseViewV1.fields.reserved == 4)
    and ($response.FrameworkNotificationResponseViewV1.fields.notification_id == 8)
    and ($response.FrameworkNotificationResponseViewV1.fields.action_id == 24)
    and ($response.FrameworkNotificationResponseViewV1.fields.user_text == 40)
    and all([$response.FrameworkNotificationResponseViewV1.size,
        $response.FrameworkNotificationResponseViewV1.align,
        $response.FrameworkNotificationResponseViewV1.fields[]][];
        type == "number" and floor == .)
    and (($response.view_fields | keys | sort) == ["action_id", "notification_id", "user_text"])
    and ($response.view_fields == {
        "notification_id": "always present",
        "action_id": "present only for CustomAction and TextInput",
        "user_text": "present only for TextInput; empty text remains present by kind"
    })
    and all($response.view_fields[]; type == "string" and length > 0)
    and (($response.status_mapping | keys | sort) == [
        "FRAMEWORK_STATUS_INTERNAL_ERROR",
        "FRAMEWORK_STATUS_INVALID_ARGUMENT",
        "FRAMEWORK_STATUS_PANIC",
        "FRAMEWORK_STATUS_RESOURCE_EXHAUSTED"
    ])
    and ($response.status_mapping == {
        "FRAMEWORK_STATUS_INVALID_ARGUMENT": "Malformed spans, invalid UTF-8 or IDs, unknown tags, non-empty unused fields, or null required outputs",
        "FRAMEWORK_STATUS_INTERNAL_ERROR": "A future D9 response kind has no C tag",
        "FRAMEWORK_STATUS_RESOURCE_EXHAUSTED": "String reserve failure or view span length that does not fit in u64",
        "FRAMEWORK_STATUS_PANIC": "Panic caught by create or get_view"
    })
    and all($response.status_mapping[]; type == "string" and length > 0)
    and (($response.ownership | keys | sort) == ["handle", "inputs", "output_slots", "view"])
    and ($response.ownership == {
        "inputs": "FrameworkStr spans are borrowed through create and copied into the owned response; no input span is retained",
        "output_slots": "create: out_response points to aligned storage with write access, is distinct from every input span and live handle slot, and must not hold a live handle on entry; set to NULL before input checks. get_view: out_view points to aligned storage with write access, is distinct from response, and is zeroed before response validation. destroy: a non-NULL response slot points to aligned storage with write access; clear the live original handle before drop",
        "handle": "Destroy the original handle once through its writable pointer slot; accept a null pointer-to-pointer or null handle as a no-op; clear a live original slot before drop; do not copy or destroy an alias",
        "view": "View spans borrow from the handle and expire at destroy"
    })
    and all($response.ownership[]; type == "string" and length > 0)
    and ($response.limits == "Creates, views, and destroys values only; no delivery, delegate, callback, event queue, polling API, executor, native response object, or B12 lifecycle semantics")
' bindings/c/abi-manifest.json > /dev/null

{
    cat <<'ASSERT_HEADER'
#include <stddef.h>
#if defined(__cplusplus)
#define FRAMEWORK_NOTIFICATION_STATIC_ASSERT static_assert
#define FRAMEWORK_NOTIFICATION_ALIGNOF alignof
#else
#define FRAMEWORK_NOTIFICATION_STATIC_ASSERT _Static_assert
#define FRAMEWORK_NOTIFICATION_ALIGNOF _Alignof
#endif
ASSERT_HEADER
    jq -r '
        .optional_capabilities.notification_responses.response_kinds
        | to_entries[]
        | "#define FRAMEWORK_C_MANIFEST_" + .key + " " + (.value | tostring)
          + "\nFRAMEWORK_NOTIFICATION_STATIC_ASSERT(" + .key
          + " == FRAMEWORK_C_MANIFEST_" + .key
          + ", \"" + .key + " must match ABI manifest\");"
    ' bindings/c/abi-manifest.json
    jq -r '
        .optional_capabilities.notification_responses.FrameworkNotificationResponseViewV1 as $layout
        | "FRAMEWORK_NOTIFICATION_STATIC_ASSERT(sizeof(FrameworkNotificationResponseViewV1) == "
          + ($layout.size | tostring) + ", \"view size mismatch\");",
          "FRAMEWORK_NOTIFICATION_STATIC_ASSERT(FRAMEWORK_NOTIFICATION_ALIGNOF(FrameworkNotificationResponseViewV1) == "
          + ($layout.align | tostring) + ", \"view alignment mismatch\");"
    ' bindings/c/abi-manifest.json
    jq -r '
        .optional_capabilities.notification_responses.FrameworkNotificationResponseViewV1.fields
        | to_entries[]
        | "FRAMEWORK_NOTIFICATION_STATIC_ASSERT(offsetof(FrameworkNotificationResponseViewV1, "
          + .key + ") == " + (.value | tostring)
          + ", \"view field offset mismatch\");"
    ' bindings/c/abi-manifest.json
} > target/framework-c-notification-response-manifest-asserts.h

cargo tree -p framework-c-api --no-default-features > target/framework-c-notification-responses-default-tree.txt
if rg -q 'framework-notifications' target/framework-c-notification-responses-default-tree.txt; then
    echo "notification response dependency leaked into the default C ABI build" >&2
    exit 1
fi
cargo tree -p framework-c-api --features notification-responses > target/framework-c-notification-responses-feature-tree.txt
rg -q 'framework-notifications' target/framework-c-notification-responses-feature-tree.txt

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --features notification-responses
cargo build --locked --release -p framework-c-api --features notification-responses

cat > target/framework-c-notification-responses-c.c <<'EOF'
#include <stddef.h>
#include <framework_notification_responses.h>
#include "framework-c-notification-response-manifest-asserts.h"

_Static_assert(sizeof(FrameworkNotificationResponseKind) == sizeof(uint32_t), "kind width");

int main(void) {
    static const uint8_t notification[] = "example";
    static const uint8_t action[] = "open";
    FrameworkStr notification_id = {notification, sizeof(notification) - 1};
    FrameworkStr action_id = {action, sizeof(action) - 1};
    FrameworkStr unused = {NULL, 0};
    FrameworkNotificationResponse *response = NULL;
    FrameworkNotificationResponseViewV1 view = {0};
    FrameworkStatus status = framework_notification_response_create(
        notification_id,
        FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION,
        action_id,
        unused,
        &response);
    status |= framework_notification_response_get_view(response, &view);
    framework_notification_response_destroy(&response);
    return (int)status;
}
EOF

cat > target/framework-c-notification-responses-cpp.cpp <<'EOF'
#include <cstddef>
#include <framework_notification_responses.h>
#include "framework-c-notification-response-manifest-asserts.h"

static_assert(sizeof(FrameworkNotificationResponseKind) == sizeof(uint32_t), "kind width");

int main() {
    static const uint8_t notification[] = "example";
    FrameworkStr notification_id = {notification, sizeof(notification) - 1};
    static const uint8_t action[] = "input";
    FrameworkStr action_id = {action, sizeof(action) - 1};
    FrameworkStr empty_text = {nullptr, 0};
    FrameworkNotificationResponse *response = nullptr;
    FrameworkNotificationResponseViewV1 view = {};
    FrameworkStatus status = framework_notification_response_create(
        notification_id,
        FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT,
        action_id,
        empty_text,
        &response);
    status |= framework_notification_response_get_view(response, &view);
    framework_notification_response_destroy(&response);
    return static_cast<int>(status) + static_cast<int>(empty_text.length != 0);
}
EOF

archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-notification-responses-c.c "$archive" \
    -o target/framework-c-notification-responses-c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-notification-responses-cpp.cpp "$archive" \
    -o target/framework-c-notification-responses-cpp

nm -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-c-notification-responses-symbols.txt
jq -r '.c_symbols[], .optional_capabilities.notification_responses.symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-notification-responses-expected-symbols.txt
jq -r '.optional_capabilities.notification_responses.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-notification-responses-header-expected-symbols.txt
rg -o 'framework_notification_response_[A-Za-z0-9_]+' \
    bindings/c/include/framework_notification_responses.h | sort -u \
    > target/framework-c-notification-responses-header-symbols.txt
diff -u target/framework-c-notification-responses-header-expected-symbols.txt \
    target/framework-c-notification-responses-header-symbols.txt
diff -u target/framework-c-notification-responses-expected-symbols.txt target/framework-c-notification-responses-symbols.txt

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

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

_Static_assert(sizeof(FrameworkNotificationResponseViewV1) == 56, "view size");
_Static_assert(offsetof(FrameworkNotificationResponseViewV1, kind) == 0, "kind offset");
_Static_assert(offsetof(FrameworkNotificationResponseViewV1, reserved) == 4, "reserved offset");
_Static_assert(offsetof(FrameworkNotificationResponseViewV1, notification_id) == 8, "notification ID offset");
_Static_assert(offsetof(FrameworkNotificationResponseViewV1, action_id) == 24, "action ID offset");
_Static_assert(offsetof(FrameworkNotificationResponseViewV1, user_text) == 40, "user text offset");

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

static_assert(sizeof(FrameworkNotificationResponseViewV1) == 56, "view size");
static_assert(offsetof(FrameworkNotificationResponseViewV1, notification_id) == 8, "notification ID offset");
static_assert(offsetof(FrameworkNotificationResponseViewV1, action_id) == 24, "action ID offset");
static_assert(offsetof(FrameworkNotificationResponseViewV1, user_text) == 40, "user text offset");

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
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-notification-responses-c.c "$archive" \
    -o target/framework-c-notification-responses-c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-notification-responses-cpp.cpp "$archive" \
    -o target/framework-c-notification-responses-cpp

nm -gU "$archive" 2>/dev/null | awk '$NF ~ /^_framework_[A-Za-z0-9_]+$/ { name=$NF; sub(/^_/, "", name); print name }' | sort -u > target/framework-c-notification-responses-symbols.txt
jq -r '.c_symbols[], .optional_capabilities.notification_responses.symbols[]' bindings/c/abi-manifest.json | sort -u > target/framework-c-notification-responses-expected-symbols.txt
rg -q 'FrameworkStatus framework_notification_response_create\(' bindings/c/include/framework_notification_responses.h
rg -q 'FrameworkStatus framework_notification_response_get_view\(' bindings/c/include/framework_notification_responses.h
rg -q 'void framework_notification_response_destroy\(' bindings/c/include/framework_notification_responses.h
diff -u target/framework-c-notification-responses-expected-symbols.txt target/framework-c-notification-responses-symbols.txt

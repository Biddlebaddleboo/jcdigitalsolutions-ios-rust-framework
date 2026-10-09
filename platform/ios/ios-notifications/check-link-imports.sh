#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS notifications link/import check" >&2
        exit 1
    fi
done

probe_source=platform/ios/ios-notifications/examples/ios_notifications_link_import_probe.rs
if ! grep -Fq 'backend.pending_request_count()' "$probe_source"; then
    echo "iOS notifications link probe must retain the pending-request count API call" >&2
    exit 1
fi
if ! grep -Fq 'backend.authorization_status_raw_value()' "$probe_source"; then
    echo "iOS notifications link probe must retain the raw authorization-status API call" >&2
    exit 1
fi
if ! grep -Fq 'backend.notification_setting_raw_values()' "$probe_source"; then
    echo "iOS notifications link probe must retain the raw alert/sound/badge API call" >&2
    exit 1
fi
if ! grep -Fq 'backend.notification_settings_extended_raw_values()' "$probe_source"; then
    echo "iOS notifications link probe must retain the extended raw settings API call" >&2
    exit 1
fi
if ! grep -Fq 'backend.notification_settings_surface_raw_values()' "$probe_source"; then
    echo "iOS notifications link probe must retain the notification-surface API call" >&2
    exit 1
fi
settings_source=platform/ios/ios-notifications/src/platform.rs
for selector in criticalAlertSetting timeSensitiveSetting scheduledDeliverySetting announcementSetting; do
    if ! grep -Fq "respondsToSelector(sel!($selector))" "$settings_source"; then
        echo "extended raw settings must availability-check $selector" >&2
        exit 1
    fi
done

cat > target/ios-notifications-link-imports-expected.txt <<'IMPORTS'
Foundation
UserNotifications
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo +1.94.1 build --locked --release -p ios-notifications --example ios_notifications_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_notifications_link_import_probe"
    imports="target/ios-notifications-link-imports-$target.txt"
    libraries="target/ios-notifications-link-libraries-$target.txt"
    symbols="target/ios-notifications-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-notifications-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|\$s[0-9]|(_OBJC_(CLASS|METACLASS)_\$_)(UIApplication|UIView|CLLocationManager|PKPushRegistry|UNNotificationResponse|UNUserNotificationCenterDelegate)|UNPushNotificationTrigger' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated capability symbol import in $symbols" >&2
        exit 1
    fi
done

if find platform/ios/ios-notifications -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-notifications" >&2
    exit 1
fi

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
if ! grep -Fq '.with_system_default_sound()' "$probe_source"; then
    echo "iOS notifications link probe must retain the system-default sound API call" >&2
    exit 1
fi
if ! grep -Fq '.with_badge_count(3)' "$probe_source"; then
    echo "iOS notifications link probe must retain the badge-count API call" >&2
    exit 1
fi
if ! grep -Fq 'backend.pending_request_count()' "$probe_source"; then
    echo "iOS notifications link probe must retain the pending-request count API call" >&2
    exit 1
fi
if ! grep -Fq 'backend.request_authorization_with_options(' "$probe_source"; then
    echo "iOS notifications link probe must retain the explicit authorization-options API call" >&2
    exit 1
fi
for expression in 'backend.register_categories(' 'IosNotificationAction::button(' 'IosNotificationAction::text_input(' '.with_category_identifier(category_identifier)'; do
    if ! grep -Fq "$expression" "$probe_source"; then
        echo "iOS notifications link probe must retain the category/action API call: $expression" >&2
        exit 1
    fi
done
authorization_source=platform/ios/ios-notifications/src/platform.rs
for option in Alert Badge Sound; do
    if ! grep -Fq "native_options |= UNAuthorizationOptions::$option;" "$authorization_source"; then
        echo "iOS notification authorization must map the $option option" >&2
        exit 1
    fi
done
if ! grep -Fq 'requestAuthorizationWithOptions_completionHandler(native_options, &handler);' "$authorization_source"; then
    echo "iOS notification authorization must pass selected options to UserNotifications" >&2
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
for expression in 'content.setCategoryIdentifier(&NSString::from_str(category_identifier));' 'self.center.setNotificationCategories(&native_categories);' 'CATEGORY_REGISTRATION_LOCK' 'UNNotificationAction::actionWithIdentifier_title_options(' 'UNTextInputNotificationAction::actionWithIdentifier_title_options_textInputButtonTitle_textInputPlaceholder(' 'UNNotificationCategory::categoryWithIdentifier_actions_intentIdentifiers_options('; do
    if ! grep -Fq "$expression" "$settings_source"; then
        echo "iOS notification action/category mapping must retain: $expression" >&2
        exit 1
    fi
done
for option in AuthenticationRequired Destructive Foreground; do
    if ! grep -Fq "UNNotificationActionOptions::$option" "$settings_source"; then
        echo "iOS notification action must map the $option option" >&2
        exit 1
    fi
done
for regression in duplicate_actions_are_rejected_without_reordering_existing_actions duplicate_categories_are_rejected_and_empty_set_is_valid_cleanup text_input_action_owns_exact_prompt_and_action_values; do
    if ! grep -Fq "fn $regression()" platform/ios/ios-notifications/src/actions.rs; then
        echo "iOS notification category regression is required: $regression" >&2
        exit 1
    fi
done
for regression in category_identifiers_reject_empty_and_nul_values_without_normalization category_identifier_is_absent_by_default_and_preserves_exact_text; do
    if ! grep -Fq "fn $regression()" crates/framework-notifications/src/lib.rs; then
        echo "portable notification category regression is required: $regression" >&2
        exit 1
    fi
done
if ! grep -Fq 'NotificationCategoryId::new' crates/framework-notifications/src/lib.rs; then
    echo "portable notification category identifiers must keep their validated constructor" >&2
    exit 1
fi
if ! grep -Fq 'notification.content().badge_count()' "$settings_source" || \
    ! grep -Fq 'content.setBadge(Some(&badge));' "$settings_source"; then
    echo "iOS notification requests must map a present badge count to UserNotifications" >&2
    exit 1
fi
if ! grep -Fq 'if notification.content().uses_system_default_sound() {' "$settings_source"; then
    echo "iOS notification requests must set sound only when the portable opt-in is true" >&2
    exit 1
fi
if ! grep -Fq 'let sound = UNNotificationSound::defaultSound();' "$settings_source" || \
    ! grep -Fq 'content.setSound(Some(&sound));' "$settings_source"; then
    echo "iOS notification requests must use public UNNotificationSound.defaultSound" >&2
    exit 1
fi
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
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
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

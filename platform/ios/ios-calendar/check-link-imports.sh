#!/bin/sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/ios-calendar-imports.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

cat >"$scratch/Cargo.toml" <<EOF
[package]
name = "ios-calendar-import-probe"
version = "0.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
framework-calendar = { path = "$repo_root/crates/framework-calendar" }
ios-calendar = { path = "$repo_root/platform/ios/ios-calendar" }
EOF

mkdir -p "$scratch/src"
cat >"$scratch/src/lib.rs" <<'EOF'
use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use framework_calendar::{
    Calendar, CalendarAuthorizationBackend, CalendarAuthorizationStatus, CalendarEvent,
    CalendarEventDraft, CalendarEventRange, CalendarEventTime,
    CalendarReminderAuthorizationBackend, CalendarReminderDraft, ReminderAuthorizationStatus,
    WritableEventCalendar,
};
use ios_calendar::IosCalendarBackend;
use std::string::String;
use std::sync::Arc;
use std::task::Wake;

struct NoopWake;

impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_authorization_link_probe() -> u8 {
    let backend = IosCalendarBackend::new();
    let calendar = Calendar::new(backend);
    match calendar.authorization_status() {
        CalendarAuthorizationStatus::NotDetermined => 1,
        CalendarAuthorizationStatus::Restricted => 2,
        CalendarAuthorizationStatus::Denied => 3,
        CalendarAuthorizationStatus::WriteOnly => 4,
        CalendarAuthorizationStatus::FullAccess => 5,
        CalendarAuthorizationStatus::Unknown => 0,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_request_link_probe() -> bool {
    let mut backend = IosCalendarBackend::new();
    let mut future = pin!(backend.request_full_access());
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    matches!(future.as_mut().poll(&mut context), Poll::Ready(_))
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_reminder_status_link_probe() -> u8 {
    match Calendar::new(IosCalendarBackend::new()).reminder_authorization_status() {
        ReminderAuthorizationStatus::NotDetermined => 1,
        ReminderAuthorizationStatus::Restricted => 2,
        ReminderAuthorizationStatus::Denied => 3,
        ReminderAuthorizationStatus::FullAccess => 4,
        ReminderAuthorizationStatus::Unknown => 0,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_reminder_request_link_probe() -> bool {
    let mut backend = IosCalendarBackend::new();
    let mut future = pin!(backend.request_full_reminder_access());
    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    matches!(future.as_mut().poll(&mut context), Poll::Ready(_))
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_reminder_create_link_probe() -> u8 {
    let draft = CalendarReminderDraft::new(String::from("Link probe")).unwrap();
    match Calendar::new(IosCalendarBackend::new()).create_reminder(&draft) {
        Ok(()) => 1,
        Err(error) => error.kind() as u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_event_range_link_probe(start: f64, end: f64) -> u8 {
    let Ok(start) = CalendarEventTime::from_unix_seconds(start) else {
        return 0;
    };
    let Ok(end) = CalendarEventTime::from_unix_seconds(end) else {
        return 0;
    };
    let Ok(range) = CalendarEventRange::new(start, end) else {
        return 0;
    };
    match Calendar::new(IosCalendarBackend::new()).events_in_range(range) {
        Ok(events) => events.len().min(u8::MAX as usize) as u8,
        Err(error) => error.kind() as u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_event_create_link_probe() -> u8 {
    let calendar = Calendar::new(IosCalendarBackend::new());
    let mut calendars = match calendar.writable_event_calendars() {
        Ok(calendars) => calendars,
        Err(error) => return error.kind() as u8,
    };
    if calendars.is_empty() {
        return 0;
    }
    let selected = calendars.remove(0);
    let start = CalendarEventTime::from_unix_seconds(1_700_000_000.0).unwrap();
    let end = CalendarEventTime::from_unix_seconds(1_700_000_060.0).unwrap();
    let draft = CalendarEventDraft::new(
        String::from("Link probe"),
        start,
        end,
        false,
        Some(String::from("Probe location")),
        Some(String::from("Probe notes")),
    )
    .unwrap();
    match calendar.create_event(&selected, &draft) {
        Ok(()) => 1,
        Err(error) => error.kind() as u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_event_delete_link_probe() -> u8 {
    let start = CalendarEventTime::from_unix_seconds(1_700_000_000.0).unwrap();
    let end = CalendarEventTime::from_unix_seconds(1_700_000_060.0).unwrap();
    let event = CalendarEvent::new(
        Some(String::from("link-probe-event")),
        String::from("Link probe"),
        start,
        end,
        false,
        None,
        None,
    )
    .unwrap();
    match Calendar::new(IosCalendarBackend::new()).delete_event(&event) {
        Ok(()) => 1,
        Err(error) => error.kind() as u8,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calendar_event_update_link_probe() -> u8 {
    let start = CalendarEventTime::from_unix_seconds(1_700_000_000.0).unwrap();
    let end = CalendarEventTime::from_unix_seconds(1_700_000_060.0).unwrap();
    let event = CalendarEvent::new(
        Some(String::from("link-probe-event")),
        String::from("Link probe"),
        start,
        end,
        false,
        None,
        None,
    )
    .unwrap();
    let replacement = CalendarEventDraft::new(
        String::from("Updated link probe"),
        CalendarEventTime::from_unix_seconds(1_700_000_100.0).unwrap(),
        CalendarEventTime::from_unix_seconds(1_700_000_160.0).unwrap(),
        true,
        Some(String::from("Updated location")),
        Some(String::from("Updated notes")),
    )
    .unwrap();
    match Calendar::new(IosCalendarBackend::new()).update_event(&event, &replacement) {
        Ok(()) => 1,
        Err(error) => error.kind() as u8,
    }
}
EOF

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    IPHONEOS_DEPLOYMENT_TARGET=17.0 CARGO_TARGET_DIR="$scratch/target" cargo build \
        --manifest-path "$scratch/Cargo.toml" \
        --release \
        --target "$target"
    artifact="$scratch/target/$target/release/libios_calendar_import_probe.dylib"
    imports=$(otool -L "$artifact" | awk 'NR > 2 {print $1}')
    printf '%s\n' "$imports" | grep -Fq '/EventKit.framework/EventKit'
    printf '%s\n' "$imports" | grep -Fq '/Foundation.framework/Foundation'
    while IFS= read -r import; do
        case "$import" in
            */EventKit.framework/EventKit|*/Foundation.framework/Foundation|*/libSystem.B.dylib|*/libobjc.A.dylib) ;;
            *) printf 'unexpected native import %s in %s\n' "$import" "$artifact" >&2; exit 1 ;;
        esac
    done <<EOF_IMPORTS
$imports
EOF_IMPORTS
    symbols="$scratch/$target-undefined.txt"
    nm -u "$artifact" >"$symbols"
    grep -q 'objc_getClass' "$symbols"
    grep -q 'objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|fetchReminders|predicateForReminders|removeReminder|requestWriteOnlyAccessToReminders' "$symbols"; then
        printf 'unexpected Swift runtime or reminder read/delete/write-only symbol in %s\n' "$symbols" >&2
        exit 1
    fi
    selectors="$scratch/$target-selectors.txt"
    strings "$artifact" >"$selectors"
    for selector in calendarsForEntityType: calendarIdentifier title calendarWithIdentifier: allowedEntityTypes allowsContentModifications eventWithEventStore: setCalendar: setTitle: setStartDate: setEndDate: setAllDay: setLocation: setNotes: saveEvent:span:commit:error: eventWithIdentifier: eventIdentifier startDate endDate calendar removeEvent:span:commit:error: requestFullAccessToRemindersWithCompletion: defaultCalendarForNewReminders reminderWithEventStore: saveReminder:commit:error:; do
        if ! grep -Fq "$selector" "$selectors"; then
            printf 'missing EventKit selector %s in %s\n' "$selector" "$artifact" >&2
            exit 1
        fi
    done
    build_info="$scratch/$target-build.txt"
    vtool -show-build "$artifact" >"$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "17.0" ]; then
        printf '%s has deployment target %s; expected 17.0\n' "$target" "${actual_deployment_target:-unknown}" >&2
        exit 1
    fi
    printf '%s\n' "$target imports:" "$imports"
done

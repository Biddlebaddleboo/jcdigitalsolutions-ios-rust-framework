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
use framework_calendar::{Calendar, CalendarAuthorizationBackend, CalendarAuthorizationStatus};
use ios_calendar::IosCalendarBackend;
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
EOF

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    CARGO_TARGET_DIR="$scratch/target" cargo build \
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
    printf '%s\n' "$target imports:" "$imports"
done

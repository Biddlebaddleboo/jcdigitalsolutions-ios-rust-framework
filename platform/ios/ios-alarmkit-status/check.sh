#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

manifest=platform/ios/ios-alarmkit-status/Cargo.toml
scoped_files="PLAN_CAPABILITIES_ALARMKIT.md PLAN_SWIFT_ABI.md docs/ios/alarmkit-status.md .github/workflows/ci.yml $manifest platform/ios/ios-alarmkit-status/build.rs platform/ios/ios-alarmkit-status/check.sh platform/ios/ios-alarmkit-status/native/alarmkit_status.c platform/ios/ios-alarmkit-status/src/lib.rs"

if find platform/ios/ios-alarmkit-status -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source is not allowed in ios-alarmkit-status" >&2
    exit 1
fi
if rg -n '[[:blank:]]+$' $scoped_files; then
    echo "trailing whitespace found in B280-scoped files" >&2
    exit 1
fi

sh -n platform/ios/ios-alarmkit-status/check.sh
cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked --offline -p ios-alarmkit-status
cargo +1.94.1 check --locked --offline -p ios-alarmkit-status --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-alarmkit-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --offline --lib -p ios-alarmkit-status -- -D warnings
cargo +1.94.1 clippy --locked --offline --lib -p ios-alarmkit-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --offline --lib -p ios-alarmkit-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --offline --no-deps -p ios-alarmkit-status
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --offline --no-deps -p ios-alarmkit-status --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
git diff --check -- $scoped_files

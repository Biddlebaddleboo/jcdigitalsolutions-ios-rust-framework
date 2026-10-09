#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")/../../.." rev-parse --show-toplevel)"
cd "$repo_root"

manifest=platform/ios/ios-activitykit-status/Cargo.toml
source_files="platform/ios/ios-activitykit-status/src/lib.rs platform/ios/ios-activitykit-status/native/activitykit_status.c platform/ios/ios-activitykit-status/build.rs platform/ios/ios-activitykit-status/examples/activitykit_status_link_probe.rs"
scoped_files="PLAN_CAPABILITIES_ACTIVITYKIT.md $manifest platform/ios/ios-activitykit-status/README.md platform/ios/ios-activitykit-status/check.sh platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh platform/ios/ios-activitykit-status/scripts/check-link-imports.sh $source_files docs/ios/activitykit-status.md"

if ! rg -Fq 'features = ["apple-runtime"]' "$manifest"; then
    echo "swift-abi-core apple-runtime feature is required" >&2
    exit 1
fi
if ! rg -Fq 'ActivityAuthorizationInfo().areActivitiesEnabled' platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh; then
    echo "compiler oracle does not call the selected ActivityKit API" >&2
    exit 1
fi
if ! rg -Fq '0A17AuthorizationInfoC20areActivitiesEnabledSbvg' platform/ios/ios-activitykit-status/native/activitykit_status.c; then
    echo "the native thunk does not target the selected ActivityKit getter" >&2
    exit 1
fi
if rg -n 'activityEnablementUpdates|frequentPushesEnabled|Activity\.request|\.update\(|\.end\(|ActivityAttributes|WidgetCenter' $source_files; then
    echo "out-of-scope ActivityKit lifecycle or WidgetKit API found" >&2
    exit 1
fi
if find platform/ios/ios-activitykit-status -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source is not allowed in ios-activitykit-status" >&2
    exit 1
fi
if rg -n '[[:blank:]]+$' $scoped_files; then
    echo "trailing whitespace found in B209-scoped files" >&2
    exit 1
fi

sh -n platform/ios/ios-activitykit-status/check.sh
sh -n platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh
sh -n platform/ios/ios-activitykit-status/scripts/check-link-imports.sh
cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked -p ios-activitykit-status
cargo +1.94.1 check --locked -p ios-activitykit-status --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-activitykit-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-activitykit-status -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-activitykit-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-activitykit-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-activitykit-status
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-activitykit-status --target aarch64-apple-ios
cargo +1.94.1 tree --locked -p ios-activitykit-status --target aarch64-apple-ios -e features
sh platform/ios/ios-activitykit-status/scripts/check-swiftcall.sh
sh platform/ios/ios-activitykit-status/scripts/check-link-imports.sh
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
git diff --check -- PLAN_CAPABILITIES_ACTIVITYKIT.md

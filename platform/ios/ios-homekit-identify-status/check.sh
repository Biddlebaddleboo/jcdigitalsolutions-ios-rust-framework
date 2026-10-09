#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-homekit-identify-status/src/lib.rs
manifest=platform/ios/ios-homekit-identify-status/Cargo.toml
if ! rg -Fq 'unsafe { accessory.supportsIdentify() }' "$source_file"; then
    echo 'no typed supportsIdentify getter found' >&2
    exit 1
fi
if ! rg -Fq 'objc2::available!(ios = 11.3, ..)' "$source_file"; then
    echo 'iOS 11.3 API availability guard is missing' >&2
    exit 1
fi
if rg -n 'HMHomeManager::|HMHomeManager::new|requestAuthorization\(|\.identifyWithCompletionHandler\(|\.identify\(' "$source_file"; then
    echo 'out-of-scope HomeKit lifecycle or identify operation found' >&2
    exit 1
fi
if ! rg -Fq 'objc2-home-kit = { version = "=0.3.2", default-features = false, features = ["HMAccessory"] }' "$manifest"; then
    echo 'unexpected objc2-home-kit dependency or feature set' >&2
    exit 1
fi

cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked -p ios-homekit-identify-status
cargo +1.94.1 clippy --locked --all-targets -p ios-homekit-identify-status -- -D warnings
cargo +1.94.1 doc --locked -p ios-homekit-identify-status --no-deps
cargo +1.94.1 check --locked -p ios-homekit-identify-status --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-homekit-identify-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --all-targets -p ios-homekit-identify-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --all-targets -p ios-homekit-identify-status --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked -p ios-homekit-identify-status --target aarch64-apple-ios --no-deps
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
git diff --check -- "$source_file" "$manifest" docs/ios/homekit-identify-status.md PLAN_CAPABILITIES_HOMEKIT.md

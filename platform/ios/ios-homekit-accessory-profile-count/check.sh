#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-homekit-accessory-profile-count/src/lib.rs
manifest=platform/ios/ios-homekit-accessory-profile-count/Cargo.toml
if ! rg -Fq 'let profiles = unsafe { accessory.profiles() };' "$source_file"; then
    echo 'no typed profiles getter found' >&2
    exit 1
fi
if ! rg -Fq 'Ok(profiles.len())' "$source_file"; then
    echo 'profile count does not read only NSArray length' >&2
    exit 1
fi
if ! rg -Fq 'objc2::available!(ios = 11, ..)' "$source_file"; then
    echo 'iOS 11.0 API availability guard is missing' >&2
    exit 1
fi
if rg -n 'profiles\.(iter|objectAtIndex)|\.uniqueIdentifier|\.categoryType|\.services\(|HMHomeManager::|requestAuthorization\(|\.identifyWithCompletionHandler\(|\.identify\(' "$source_file"; then
    echo 'profile content or out-of-scope HomeKit operation found' >&2
    exit 1
fi
if ! rg -Fq 'objc2-home-kit = { version = "=0.3.2", default-features = false, features = ["HMAccessory", "HMAccessoryProfile"] }' "$manifest"; then
    echo 'unexpected objc2-home-kit dependency or feature set' >&2
    exit 1
fi

cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked --offline -p ios-homekit-accessory-profile-count
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-accessory-profile-count -- -D warnings
cargo +1.94.1 doc --locked --offline -p ios-homekit-accessory-profile-count --no-deps
# Fetch target-only bindings before the offline device and simulator checks.
cargo +1.94.1 check --locked -p ios-homekit-accessory-profile-count --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-homekit-accessory-profile-count --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-homekit-accessory-profile-count --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-accessory-profile-count --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-accessory-profile-count --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --offline -p ios-homekit-accessory-profile-count --target aarch64-apple-ios --no-deps
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
git diff --check -- "$source_file" "$manifest" platform/ios/ios-homekit-accessory-profile-count/check.sh docs/ios/homekit-accessory-profile-count.md PLAN_CAPABILITIES_HOMEKIT.md PLAN_CAPABILITIES.md docs/capabilities/capability-status.json .github/workflows/ci.yml

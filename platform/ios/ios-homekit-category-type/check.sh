#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-homekit-category-type/src/lib.rs
manifest=platform/ios/ios-homekit-category-type/Cargo.toml
guide=docs/ios/homekit-category-type.md
plan=PLAN_CAPABILITIES_HOMEKIT.md
if ! rg -Fq 'unsafe { category.categoryType() }' "$source_file"; then
    echo 'no typed categoryType getter found' >&2
    exit 1
fi
if ! rg -Fq 'objc2::available!(ios = 9, ..)' "$source_file"; then
    echo 'iOS 9.0 API availability guard is missing' >&2
    exit 1
fi
if rg -n '\.category\(|\.profiles\(|\.services\(|HMHomeManager::|requestAuthorization\(|identifyWithCompletionHandler|HMCameraProfile|HMService::' "$source_file"; then
    echo 'out-of-scope category acquisition, profile, service, or HomeKit lifecycle operation found' >&2
    exit 1
fi
if ! rg -Fq 'objc2-home-kit = { version = "=0.3.2", default-features = false, features = ["HMAccessoryCategory"] }' "$manifest"; then
    echo 'unexpected objc2-home-kit dependency or feature set' >&2
    exit 1
fi

cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked --offline -p ios-homekit-category-type
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-category-type -- -D warnings
cargo +1.94.1 doc --locked --offline -p ios-homekit-category-type --no-deps
cargo +1.94.1 check --locked --offline -p ios-homekit-category-type --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-homekit-category-type --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-category-type --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-category-type --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --offline -p ios-homekit-category-type --target aarch64-apple-ios --no-deps
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
git diff --check -- "$source_file" "$manifest" platform/ios/ios-homekit-category-type/check.sh "$guide" "$plan" .github/workflows/ci.yml

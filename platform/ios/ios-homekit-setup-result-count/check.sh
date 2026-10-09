#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-homekit-setup-result-count/src/lib.rs
manifest=platform/ios/ios-homekit-setup-result-count/Cargo.toml
guide=docs/ios/homekit-setup-result-count.md
plan=PLAN_CAPABILITIES_HOMEKIT.md
if ! rg -Fq 'unsafe { result.accessoryUniqueIdentifiers() }' "$source_file"; then
    echo 'no typed accessoryUniqueIdentifiers getter found' >&2
    exit 1
fi
if ! rg -Fq 'Ok(identifiers.len())' "$source_file"; then
    echo 'setup result count inspects more than array length' >&2
    exit 1
fi
if ! rg -Fq 'objc2::available!(ios = 15.4, ..)' "$source_file"; then
    echo 'iOS 15.4 API availability guard is missing' >&2
    exit 1
fi
if rg -n 'objectAtIndex|\.iter\(|uniqueIdentifier\(|HMAccessorySetupManager::|performAccessorySetup|requestAuthorization\(|\.profiles\(|\.services\(|HMHomeManager::' "$source_file"; then
    echo 'out-of-scope identifier, service/profile enumeration, or HomeKit lifecycle operation found' >&2
    exit 1
fi
if ! rg -Fq 'objc2-home-kit = { version = "=0.3.2", default-features = false, features = ["HMAccessorySetupResult"] }' "$manifest"; then
    echo 'unexpected objc2-home-kit dependency or feature set' >&2
    exit 1
fi

cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked --offline -p ios-homekit-setup-result-count
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-setup-result-count -- -D warnings
cargo +1.94.1 doc --locked --offline -p ios-homekit-setup-result-count --no-deps
cargo +1.94.1 check --locked --offline -p ios-homekit-setup-result-count --target aarch64-apple-ios
cargo +1.94.1 check --locked --offline -p ios-homekit-setup-result-count --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-setup-result-count --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --offline --all-targets -p ios-homekit-setup-result-count --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --offline -p ios-homekit-setup-result-count --target aarch64-apple-ios --no-deps
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
git diff --check -- "$source_file" "$manifest" platform/ios/ios-homekit-setup-result-count/check.sh "$guide" "$plan" .github/workflows/ci.yml

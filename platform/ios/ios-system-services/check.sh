#!/bin/sh
set -eu

repo_root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
cd "$repo_root"

source_file=platform/ios/ios-system-services/src/classkit.rs
if rg -n '^\s*(use|pub use).*?(CLSDataStore|CLSContext|CLSActivity)|\.(contextIdentifierPath|saveWithCompletion|contextsMatching|completeAllAssignedActivities)\s*\(' "$source_file"; then
    echo "out-of-scope ClassKit path, store, activity, or progress API found" >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-system-services/Cargo.toml -- --check
cargo check --locked -p ios-system-services --lib
cargo check --locked -p ios-system-services --lib --target aarch64-apple-ios
cargo check --locked -p ios-system-services --lib --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-system-services --lib --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-system-services --lib --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-system-services --no-deps --target aarch64-apple-ios

host_dependencies=$(cargo tree --locked -p ios-system-services --target aarch64-apple-darwin)
if printf '%s\n' "$host_dependencies" | grep -F 'objc2-class-kit'; then
    echo "ClassKit dependency must remain off non-iOS targets" >&2
    exit 1
fi

features=$(cargo tree --locked -p ios-system-services --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-class-kit feature "NSUserActivity_CLSDeepLinks"' >/dev/null
if printf '%s\n' "$features" | grep -E 'objc2-class-kit feature "(default|CLSDataStore|CLSContext|CLSActivity|block2|std)"'; then
    echo "ClassKit dependency enabled a default, data-store, or activity feature" >&2
    exit 1
fi

sh platform/ios/ios-system-services/check-link-imports.sh

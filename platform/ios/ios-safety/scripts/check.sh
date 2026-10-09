#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-safety/src/lib.rs
if ! rg -q 'unsafe \{ SACrashDetectionManager::isAvailable\(\) \}' "$source_file"; then
    printf '%s\n' 'missing the SafetyKit availability getter' >&2
    exit 1
fi
if ! rg -q 'objc2::available!\(ios = 16\.0, \.\.\)' "$source_file"; then
    printf '%s\n' 'missing the iOS 16.0 availability guard' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 1 ]; then
    printf '%s\n' 'out-of-scope SafetyKit call found' >&2
    exit 1
fi
if rg -n 'SACrashDetectionManager::(authorizationStatus|setDelegate|requestAuthorizationWithCompletionHandler|alloc|new|init)|SAEmergencyResponseManager::|SACrashDetectionEvent::' "$source_file"; then
    printf '%s\n' 'authorization, delegate, event, emergency-response, or manager creation API found' >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-safety/Cargo.toml -- --check
cargo check --locked -p ios-safety
cargo clippy --locked -p ios-safety --all-targets -- -D warnings
cargo doc --locked -p ios-safety --no-deps
cargo check --locked -p ios-safety --target aarch64-apple-ios
cargo check --locked -p ios-safety --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-safety --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-safety --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-safety --target aarch64-apple-ios --no-deps
sh platform/ios/ios-safety/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

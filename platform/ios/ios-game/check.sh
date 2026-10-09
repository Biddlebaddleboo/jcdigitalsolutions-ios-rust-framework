#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-game/src/platform.rs
if rg -n 'authenticateHandler|authenticateWithCompletionHandler|GKPlayerAuthenticationDidChangeNotificationName|GKGameCenterViewController|GKPlayer::|playerID|teamPlayerID' "$source_file"; then
    printf '%s\n' 'out-of-scope Game Center authentication, identity, or UI API found' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 2 ]; then
    printf '%s\n' 'expected only the local-player and authentication-status reads' >&2
    exit 1
fi

cargo fmt --all -- --check
cargo check --locked -p framework-game --no-default-features
cargo clippy --locked -p framework-game --all-targets --no-default-features -- -D warnings
cargo doc --locked -p framework-game --no-deps
cargo check --locked -p ios-game --target aarch64-apple-ios
cargo check --locked -p ios-game --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-game --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-game --all-targets --target aarch64-apple-ios-sim -- -D warnings
sh platform/ios/ios-game/check-link-imports.sh device
sh platform/ios/ios-game/check-link-imports.sh simulator
cargo xtask no-std-check
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

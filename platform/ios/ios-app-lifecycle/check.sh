#!/bin/sh
set -eu

rustfmt +1.94.1 --edition 2024 --check \
    platform/ios/ios-app-lifecycle/src/lib.rs \
    platform/ios/ios-app-lifecycle/src/mapping.rs \
    platform/ios/ios-app-lifecycle/src/observers.rs \
    platform/ios/ios-app-lifecycle/src/platform.rs \
    platform/ios/ios-app-lifecycle/src/queue.rs
sh -n platform/ios/ios-app-lifecycle/check.sh \
    platform/ios/ios-app-lifecycle/check-surface.sh
cargo +1.94.1 test --locked -p ios-app-lifecycle
cargo +1.94.1 clippy --locked --all-targets -p ios-app-lifecycle -- -D warnings
cargo +1.94.1 check --locked -p ios-app-lifecycle --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-app-lifecycle --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked -p ios-app-lifecycle --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked -p ios-app-lifecycle --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked -p ios-app-lifecycle --no-deps --target aarch64-apple-ios
sh platform/ios/ios-app-lifecycle/check-surface.sh

#!/bin/sh
set -eu

cargo +1.94.1 check --locked -p ios-app-scenes --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-app-scenes --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked -p ios-app-scenes --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked -p ios-app-scenes --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked -p ios-app-scenes --no-deps --target aarch64-apple-ios
sh platform/ios/ios-app-scenes/check-link-imports.sh

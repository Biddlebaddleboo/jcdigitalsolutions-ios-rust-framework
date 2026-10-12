#!/bin/sh
set -eu

cargo +1.94.1 test --locked --offline --no-run -p ios-bluetooth --test connection_link --target aarch64-apple-ios
cargo +1.94.1 test --locked --offline --no-run -p ios-bluetooth --test connection_link --target aarch64-apple-ios-sim

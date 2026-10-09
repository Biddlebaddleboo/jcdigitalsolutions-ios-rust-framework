#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

cargo fmt --manifest-path platform/ios/ios-modelio-status/Cargo.toml --package ios-modelio-status -- --check
cargo check --locked --all-targets -p ios-modelio-status
cargo check --locked --lib -p ios-modelio-status --target aarch64-apple-ios
cargo check --locked --lib -p ios-modelio-status --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-modelio-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-modelio-status --target aarch64-apple-ios-sim -- -D warnings

host_features=$(cargo tree --locked -p ios-modelio-status --target aarch64-apple-darwin -e features)
if printf '%s\n' "$host_features" | grep -E 'objc2-model-io|objc2-foundation'; then
    echo "ModelIO and Foundation dependencies must remain off non-iOS targets" >&2
    exit 1
fi

features=$(cargo tree --locked -p ios-modelio-status --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-model-io feature "MDLAsset"' >/dev/null
if printf '%s\n' "$features" | grep -F 'objc2-model-io feature "default"'; then
    echo "objc2-model-io default features must remain disabled" >&2
    exit 1
fi

sh platform/ios/ios-modelio-status/check-link-imports.sh device
sh platform/ios/ios-modelio-status/check-link-imports.sh simulator
cargo doc --locked --no-deps -p ios-modelio-status --target aarch64-apple-ios

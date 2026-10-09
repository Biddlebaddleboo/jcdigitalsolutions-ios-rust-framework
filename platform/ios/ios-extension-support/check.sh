#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

if find platform/ios/ios-extension-support -type f -name '*.swift' -print | grep -q .; then
    echo "ios-extension-support must not contain Swift source" >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-extension-support/Cargo.toml --package ios-extension-support -- --check
cargo check --locked --all-targets -p ios-extension-support
cargo check --locked --lib -p ios-extension-support --target aarch64-apple-ios
cargo check --locked --lib -p ios-extension-support --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-extension-support --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-extension-support --target aarch64-apple-ios-sim -- -D warnings

features=$(cargo tree --locked -p ios-extension-support --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-foundation feature "NSBundle"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-foundation feature "NSDictionary"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-foundation feature "NSString"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-foundation feature "NSURL"' >/dev/null
if printf '%s\n' "$features" | grep -F 'objc2-foundation feature "default"'; then
    echo "objc2-foundation default features must remain disabled" >&2
    exit 1
fi

sh platform/ios/ios-extension-support/check-link-imports.sh device
sh platform/ios/ios-extension-support/check-link-imports.sh simulator
cargo doc --locked --no-deps -p ios-extension-support --target aarch64-apple-ios

#!/bin/sh
set -eu

package_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
manifest="$package_dir/Cargo.toml"
repo_root=$(git -C "$package_dir" rev-parse --show-toplevel)
export CARGO_TARGET_DIR="$repo_root/target/ios-call-observer"

cargo fmt --manifest-path "$manifest" -- --check
cargo check --locked --manifest-path "$manifest" --lib
cargo check --locked --manifest-path "$manifest" --lib --target aarch64-apple-ios
cargo check --locked --manifest-path "$manifest" --lib --target aarch64-apple-ios-sim
cargo clippy --locked --manifest-path "$manifest" --lib --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --manifest-path "$manifest" --lib --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked --manifest-path "$manifest" --no-deps --target aarch64-apple-ios

host_dependencies=$(cargo tree --locked --manifest-path "$manifest" --target aarch64-apple-darwin)
if printf '%s\n' "$host_dependencies" | grep -F 'objc2-call-kit'; then
    echo "CallKit dependency must remain off non-iOS targets" >&2
    exit 1
fi

features=$(cargo tree --locked --manifest-path "$manifest" --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-call-kit feature "CXCallObserver"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-call-kit feature "CXCall"' >/dev/null
if printf '%s\n' "$features" | grep -E 'objc2-call-kit feature "(default|CXCallController|CXProvider|CXProviderConfiguration|CXCallDirectory|block2|dispatch2|objc2-avf-audio|std)"'; then
    echo "CallKit observer package enabled an out-of-scope or default feature" >&2
    exit 1
fi

sh "$package_dir/check-link-imports.sh"

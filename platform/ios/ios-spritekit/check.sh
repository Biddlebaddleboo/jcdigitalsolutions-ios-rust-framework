#!/bin/sh
set -eu

repo_root="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$repo_root"

cargo fmt --manifest-path crates/framework-spritekit/Cargo.toml --package framework-spritekit -- --check
cargo fmt --manifest-path platform/ios/ios-spritekit/Cargo.toml --package ios-spritekit -- --check

if find crates/framework-spritekit platform/ios/ios-spritekit -type f -name '*.swift' -print | grep -q .; then
    echo "unexpected Swift source in the SpriteKit crates" >&2
    exit 1
fi

cargo check --locked --offline -p framework-spritekit --no-default-features
cargo clippy --locked --offline --all-targets -p framework-spritekit -- -D warnings
cargo doc --locked --offline -p framework-spritekit --no-deps

cargo check --locked --offline -p ios-spritekit
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked --offline -p ios-spritekit --target "$target"
    cargo clippy --locked --offline --all-targets -p ios-spritekit --target "$target" -- -D warnings
    cargo doc --locked --offline -p ios-spritekit --target "$target" --no-deps
done

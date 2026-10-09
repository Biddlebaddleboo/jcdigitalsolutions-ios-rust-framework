#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --check -p framework-media -p ios-sound-analysis
cargo check --locked --no-default-features -p framework-media
cargo clippy --locked --all-targets --no-default-features -p framework-media -- -D warnings
cargo doc --locked --no-deps -p framework-media
cargo check --locked -p ios-sound-analysis

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-sound-analysis --target "$target"
    cargo clippy --locked -p ios-sound-analysis --target "$target" --lib -- -D warnings
done

cargo doc --locked --no-deps -p ios-sound-analysis
features="$(cargo tree --locked -p ios-sound-analysis --target aarch64-apple-ios -e features)"
if printf '%s\n' "$features" | grep -E 'feature "(block2|objc2-avf-audio|objc2-core-media|objc2-core-ml|SNAnalyzer|SNClassificationResult|SNRequest|SNResult|SNTimeDurationConstraint)"'; then
    printf '%s\n' 'out-of-scope SoundAnalysis features are enabled' >&2
    exit 1
fi
sh platform/ios/ios-sound-analysis/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

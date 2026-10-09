#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

cargo fmt --check -p framework-media -p ios-media
cargo check --locked --no-default-features -p framework-media
cargo clippy --locked --all-targets --no-default-features -p framework-media -- -D warnings
cargo doc --locked --no-deps -p framework-media
cargo check --locked -p ios-media

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-media --target "$target"
    cargo clippy --locked -p ios-media --all-targets --target "$target" -- -D warnings
done

cargo doc --locked --no-deps -p ios-media
features=$(cargo tree --locked -p ios-media --target aarch64-apple-ios -e features)
if printf '%s\n' "$features" | grep -E 'feature "(AVAudioEngine|AVAudioPlayer|AVAudioRecorder|AVCaptureDevice|AVAudioUnit|AVAudioSourceNode|AVAudioSinkNode|block2)"'; then
    printf '%s\n' 'out-of-scope AVFAudio or audio-input features are enabled' >&2
    exit 1
fi
sh platform/ios/ios-media/check-audio-playback-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

cargo fmt --check -p framework-media -p ios-media
cargo check --locked --no-default-features -p framework-media
cargo clippy --locked --all-targets --no-default-features -p framework-media -- -D warnings
cargo doc --locked --no-deps -p framework-media
cargo check --locked --no-default-features -p ios-media

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked --no-default-features -p ios-media --target "$target"
    cargo clippy --locked --no-default-features -p ios-media --all-targets --target "$target" -- -D warnings
done

cargo doc --locked --no-default-features --no-deps -p ios-media
features=$(cargo tree --locked -p ios-media --no-default-features --target aarch64-apple-ios -e features)
if printf '%s\n' "$features" | grep -q 'objc2-video-toolbox'; then
    printf '%s\n' 'VideoToolbox capability leaked into the audio-only feature graph' >&2
    exit 1
fi
if printf '%s\n' "$features" | grep -q 'objc2-core-media'; then
    printf '%s\n' 'CoreMedia time capability leaked into the audio-only feature graph' >&2
    exit 1
fi
if printf '%s\n' "$features" | grep -E 'feature "(AVAudioEngine|AVAudioPlayer|AVAudioRecorder|AVCaptureDevice|AVAudioUnit|AVAudioSourceNode|AVAudioSinkNode|block2)"'; then
    printf '%s\n' 'out-of-scope AVFAudio or audio-input features are enabled' >&2
    exit 1
fi
sh platform/ios/ios-media/check-audio-playback-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

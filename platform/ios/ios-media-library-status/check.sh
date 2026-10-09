#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

cargo fmt --manifest-path platform/ios/ios-media-library-status/Cargo.toml --package ios-media-library-status -- --check
cargo check --locked --lib -p ios-media-library-status
cargo check --locked --lib -p ios-media-library-status --target aarch64-apple-ios
cargo check --locked --lib -p ios-media-library-status --target aarch64-apple-ios-sim
cargo clippy --locked --lib -p ios-media-library-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --lib -p ios-media-library-status --target aarch64-apple-ios-sim -- -D warnings

host_features=$(cargo tree --locked -p ios-media-library-status --target aarch64-apple-darwin -e features)
if printf '%s\n' "$host_features" | grep -F 'objc2-media-player'; then
    echo "MediaPlayer dependency must remain off non-iOS targets" >&2
    exit 1
fi

features=$(cargo tree --locked -p ios-media-library-status --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-media-player feature \"MPMediaLibrary\"' >/dev/null
if printf '%s\n' "$features" | grep -E 'objc2-media-player feature \"(default|block2|MPMediaItem|MPMediaQuery|MPMusicPlayerController)\"'; then
    echo "MediaPlayer authorization query must not enable default, callback, item, query, or player APIs" >&2
    exit 1
fi

cargo doc --locked --no-deps -p ios-media-library-status --target aarch64-apple-ios

#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-speech-status/src/lib.rs
if ! rg -q 'unsafe \{ SFSpeechRecognizer::authorizationStatus\(\) \}' "$source_file"; then
    printf '%s\n' 'no Speech authorization status method found' >&2
    exit 1
fi
if ! rg -q 'objc2::available!\(ios = 10\.0, \.\.\)' "$source_file"; then
    printf '%s\n' 'no iOS 10.0 availability guard found' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 1 ]; then
    printf '%s\n' 'out-of-scope Speech call found' >&2
    exit 1
fi
if rg -n 'SFSpeechRecognizer::(alloc|new|init|initWithLocale|requestAuthorization|recognitionTask|supportedLocales)|SFSpeechRecognition(Request|Result|Task)::|AVAudio' "$source_file"; then
    printf '%s\n' 'permission request, audio, recognition, or locale API found' >&2
    exit 1
fi
if ! rg -q 'objc2-speech = \{ workspace = true, features = \["SFSpeechRecognizer"\] \}' platform/ios/ios-speech-status/Cargo.toml; then
    printf '%s\n' 'unexpected objc2-speech feature set' >&2
    exit 1
fi

sh platform/ios/ios-speech-status/scripts/check-link-imports.sh
cargo fmt --manifest-path platform/ios/ios-speech-status/Cargo.toml -- --check
cargo check --locked -p ios-speech-status
cargo clippy --locked -p ios-speech-status --all-targets -- -D warnings
cargo doc --locked -p ios-speech-status --no-deps
cargo check --locked -p ios-speech-status --target aarch64-apple-ios
cargo check --locked -p ios-speech-status --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-speech-status --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-speech-status --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-speech-status --target aarch64-apple-ios --no-deps
cargo xtask zero-swift-source
cargo xtask docs-check
git diff --check

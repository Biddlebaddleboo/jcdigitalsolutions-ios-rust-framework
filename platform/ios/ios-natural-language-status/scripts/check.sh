#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-natural-language-status/src/lib.rs
if ! rg -q 'NLContextualEmbedding::contextualEmbeddingWithLanguage\(language\)' "$source_file"; then
    printf '%s\n' 'no English contextual-model factory call found' >&2
    exit 1
fi
if ! rg -q 'objc2::available!\(ios = 17\.0, \.\.\)' "$source_file"; then
    printf '%s\n' 'no iOS 17.0 availability guard found' >&2
    exit 1
fi
if ! rg -q 'unsafe \{ model\.hasAvailableAssets\(\) \}' "$source_file"; then
    printf '%s\n' 'no contextual-model asset status call found' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 3 ]; then
    printf '%s\n' 'out-of-scope Natural Language call found' >&2
    exit 1
fi
if rg -n 'requestEmbeddingAssets|loadWithError|\.unload\(|embeddingResultForString|requestAssets|NLLanguageRecognizer|NLTagger' "$source_file"; then
    printf '%s\n' 'asset request, model load, text, or recognizer API found' >&2
    exit 1
fi
if ! rg -q 'objc2-natural-language = \{ workspace = true, features = \["NLContextualEmbedding", "NLLanguage"\] \}' platform/ios/ios-natural-language-status/Cargo.toml; then
    printf '%s\n' 'unexpected objc2-natural-language feature set' >&2
    exit 1
fi

sh platform/ios/ios-natural-language-status/scripts/check-link-imports.sh
cargo fmt --manifest-path platform/ios/ios-natural-language-status/Cargo.toml -- --check
cargo check --locked -p ios-natural-language-status
cargo clippy --locked -p ios-natural-language-status --all-targets -- -D warnings
cargo doc --locked -p ios-natural-language-status --no-deps
cargo check --locked -p ios-natural-language-status --target aarch64-apple-ios
cargo check --locked -p ios-natural-language-status --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-natural-language-status --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-natural-language-status --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-natural-language-status --target aarch64-apple-ios --no-deps
cargo xtask zero-swift-source
cargo xtask docs-check
git diff --check

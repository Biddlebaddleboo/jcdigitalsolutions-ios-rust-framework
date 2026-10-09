#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-shared-with-you-support/src/lib.rs
if ! rg -q 'unsafe \{ SWHighlightCenter::isSystemCollaborationSupportAvailable\(\) \}' "$source_file"; then
    printf '%s\n' 'missing the system collaboration support query' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 1 ]; then
    printf '%s\n' 'out-of-scope SharedWithYou call found' >&2
    exit 1
fi
if rg -n '::(alloc|new|highlights|setDelegate|delegate|postNoticeForHighlightEvent|clearNoticesForHighlight)|SWCollaboration[A-Za-z]+|msg_send!|sel!' "$source_file"; then
    printf '%s\n' 'highlight data, callbacks, collaboration operations, or UI found' >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-shared-with-you-support/Cargo.toml -- --check
cargo check --locked -p ios-shared-with-you-support
cargo check --locked -p ios-shared-with-you-support --target aarch64-apple-ios
cargo check --locked -p ios-shared-with-you-support --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-shared-with-you-support --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-shared-with-you-support --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-shared-with-you-support --target aarch64-apple-ios --no-deps
sh platform/ios/ios-shared-with-you-support/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-message-ui-support/src/lib.rs
if ! rg -q 'unsafe \{ MFMailComposeViewController::canSendMail\(mtm\) \}' "$source_file"; then
    printf '%s\n' 'missing the canSendMail status query' >&2
    exit 1
fi
if ! rg -q 'unsafe \{ MFMessageComposeViewController::canSendText\(mtm\) \}' "$source_file"; then
    printf '%s\n' 'missing the canSendText status query' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 2 ]; then
    printf '%s\n' 'out-of-scope MessageUI call found' >&2
    exit 1
fi
if rg -n 'present|delegate|setToRecipients|setCcRecipients|setBccRecipients|setSubject|setMessageBody|addAttachment|TextMessageAvailability|MFMailComposeResult|MessageComposeResult' "$source_file"; then
    printf '%s\n' 'compose UI, callback, content, or result API found' >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-message-ui-support/Cargo.toml -- --check
cargo check --locked -p ios-message-ui-support
cargo check --locked -p ios-message-ui-support --target aarch64-apple-ios
cargo check --locked -p ios-message-ui-support --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-message-ui-support --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-message-ui-support --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-message-ui-support --target aarch64-apple-ios --no-deps
sh platform/ios/ios-message-ui-support/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check

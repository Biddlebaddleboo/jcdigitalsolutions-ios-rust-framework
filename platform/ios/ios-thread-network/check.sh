#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

manifest=platform/ios/ios-thread-network/Cargo.toml
source_files="platform/ios/ios-thread-network/src/lib.rs platform/ios/ios-thread-network/src/platform.rs"
scoped_files="PLAN_CAPABILITIES_THREAD.md $manifest platform/ios/ios-thread-network/README.md $source_files docs/ios/thread-network.md"

if ! rg -Fq 'objc2-thread-network = { version = "=0.3.2", default-features = false, features = ["THClient", "block2", "std"] }' "$manifest"; then
    echo "unexpected objc2-thread-network feature set" >&2
    exit 1
fi
if ! rg -Fq 'client.isPreferredNetworkAvailableWithCompletion(&handler)' platform/ios/ios-thread-network/src/platform.rs; then
    echo "no preferred-network availability request found" >&2
    exit 1
fi
if rg -n 'retrievePreferredCredentials|retrieveCredentialsForExtendedPANID|retrieveAllCredentials|retrieveAllActiveCredentials|checkPreferredNetworkForActiveOperationalDataset|isPreferredNetworkAvailableWithCompletion:' $source_files; then
    echo "credential access or an out-of-scope Thread API found" >&2
    exit 1
fi
if find platform/ios/ios-thread-network -type f -name '*.swift' -print | grep -q .; then
    echo "Swift source is not allowed in ios-thread-network" >&2
    exit 1
fi
if rg -n '[[:blank:]]+$' $scoped_files; then
    echo "trailing whitespace found in B206-scoped files" >&2
    exit 1
fi

sh -n platform/ios/ios-thread-network/check.sh
cargo +1.94.1 fmt --manifest-path "$manifest" -- --check
cargo +1.94.1 check --locked -p ios-thread-network --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-thread-network --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-thread-network --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked --lib -p ios-thread-network --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-thread-network --target aarch64-apple-ios
cargo +1.94.1 tree --locked -p ios-thread-network --target aarch64-apple-ios -e features
cargo +1.94.1 xtask docs-check
git diff --check -- PLAN_CAPABILITIES_THREAD.md

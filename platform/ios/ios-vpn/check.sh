#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

manifest=platform/ios/ios-vpn/Cargo.toml
source_files="platform/ios/ios-vpn/src/lib.rs platform/ios/ios-vpn/src/completion.rs platform/ios/ios-vpn/src/platform.rs platform/ios/ios-vpn/examples/link_check.rs"
scoped_files="PLAN_IOS_VPN_STATUS.md crates/framework-vpn/Cargo.toml crates/framework-vpn/README.md crates/framework-vpn/src/lib.rs platform/ios/ios-vpn/Cargo.toml platform/ios/ios-vpn/README.md $source_files docs/capabilities/vpn.md docs/ios/vpn-status.md"

if ! rg -Fq 'objc2-network-extension = { version = "=0.3.2", default-features = false, features = ["block2"] }' "$manifest"; then
    echo "unexpected objc2-network-extension feature set" >&2
    exit 1
fi
if ! rg -Fq 'objc2-foundation = { workspace = true, features = ["NSError", "NSString"] }' "$manifest"; then
    echo "unexpected objc2-foundation feature set" >&2
    exit 1
fi
if ! rg -Fq 'manager.loadFromPreferencesWithCompletionHandler(&handler)' platform/ios/ios-vpn/src/platform.rs; then
    echo "no preference-load request found" >&2
    exit 1
fi
if ! rg -Fq 'connection.status()' platform/ios/ios-vpn/src/platform.rs; then
    echo "no connection-status read found" >&2
    exit 1
fi
if ! rg -Fq 'manager.isEnabled()' platform/ios/ios-vpn/src/platform.rs; then
    echo "no Personal VPN enabled-flag read found" >&2
    exit 1
fi
if ! rg -Fq 'manager.isOnDemandEnabled()' platform/ios/ios-vpn/src/platform.rs; then
    echo "no Connect On Demand flag read found" >&2
    exit 1
fi
if ! rg -Fq 'MainThreadMarker::new()' platform/ios/ios-vpn/src/platform.rs; then
    echo "no callback main-thread check found" >&2
    exit 1
fi
if rg -n 'setEnabled|setOnDemandEnabled|onDemandRules|saveToPreferencesWithCompletionHandler|removeFromPreferencesWithCompletionHandler|startVPNTunnel|stopVPNTunnel|NETunnelProvider|NEPacketTunnelProvider|NEAppProxyProvider|NEFilterProvider' $source_files; then
    echo "configuration mutation, tunnel control, or provider API found" >&2
    exit 1
fi
if find platform/ios/ios-vpn -type f -name '*.swift' -print | grep -q .; then
    echo "Swift source is not allowed in ios-vpn" >&2
    exit 1
fi
if rg -n '[[:blank:]]+$' $scoped_files; then
    echo "trailing whitespace found in B79-scoped files" >&2
    exit 1
fi

sh -n platform/ios/ios-vpn/check.sh platform/ios/ios-vpn/check-link-imports.sh
cargo fmt --manifest-path crates/framework-vpn/Cargo.toml -- --check
cargo fmt --manifest-path "$manifest" -- --check
cargo check --locked -p framework-vpn --no-default-features
cargo clippy --locked --lib -p framework-vpn --no-default-features -- -D warnings
cargo check --locked -p ios-vpn --target aarch64-apple-ios
cargo check --locked -p ios-vpn --target aarch64-apple-ios-sim
cargo clippy --locked --lib -p ios-vpn --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --lib -p ios-vpn --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps -p framework-vpn -p ios-vpn --target aarch64-apple-ios
cargo tree --locked -p ios-vpn --target aarch64-apple-ios -e features
sh platform/ios/ios-vpn/check-link-imports.sh
git diff --check -- PLAN_IOS_VPN_STATUS.md

#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the Personal VPN link/import check" >&2
        exit 1
    fi
done

cat > target/ios-vpn-imports-expected.txt <<'IMPORTS'
Foundation
NetworkExtension
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-vpn-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-vpn --example link_check --target "$target"
    binary="$target_dir/$target/release/examples/link_check"
    imports="target/ios-vpn-imports-$target.txt"
    libraries="target/ios-vpn-libraries-$target.txt"
    symbols="target/ios-vpn-symbols-$target.txt"
    binary_strings="target/ios-vpn-strings-$target.txt"
    build_info="target/ios-vpn-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-vpn-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    strings "$binary" > "$binary_strings"
    grep -Eq '_objc_(msgSend|getClass)' "$symbols"
    grep -Fq 'NEVPNManager' "$binary_strings"
    grep -Fq 'sharedManager' "$binary_strings"
    grep -Fq 'loadFromPreferencesWithCompletionHandler:' "$binary_strings"
    grep -Fxq 'connection' "$binary_strings"
    grep -Fxq 'status' "$binary_strings"
    if grep -Eiq 'swift|saveToPreferencesWithCompletionHandler|removeFromPreferencesWithCompletionHandler|startVPNTunnel|stopVPNTunnel|NETunnelProvider|NEPacketTunnelProvider|NEAppProxyProvider|NEFilterProvider' "$symbols" "$binary_strings"; then
        echo "unexpected Swift runtime, profile mutation, tunnel control, or provider API in $symbols or $binary_strings" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target="$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")"
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

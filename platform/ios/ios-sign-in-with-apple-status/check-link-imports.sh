#!/bin/sh
set -eu

package_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(git -C "$package_dir" rev-parse --show-toplevel)
cd "$repo_root"

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the Sign in with Apple link/import check" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=13.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="$repo_root/target/ios-sign-in-with-apple-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-sign-in-with-apple-status \
        --example credential_state_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/credential_state_link_probe"
    imports="$target_dir/imports.txt"
    libraries="$target_dir/libraries.txt"
    symbols="$target_dir/symbols.txt"
    text="$target_dir/strings.txt"
    build_info="$target_dir/build-info.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "$libraries"
    diff -u "$package_dir/link-imports-expected.txt" "$libraries"

    nm -u "$binary" > "$symbols"
    strings "$binary" > "$text"
    grep -Fq 'ASAuthorizationAppleIDProvider' "$text"
    grep -Fq 'getCredentialStateForUserID:completion:' "$text"
    grep -Fq '_objc_getClass' "$symbols"
    grep -Fq '_objc_msgSend' "$symbols"
    grep -Fq '__Block_copy' "$symbols"
    grep -Fq '__Block_release' "$symbols"
    if grep -Eiq 'swift|ASAuthorizationController|ASAuthorizationAppleIDRequest|ASAuthorizationPlatformPublicKeyCredential|ASAuthorizationWebBrowser' "$symbols" "$text"; then
        echo "unexpected Swift, authorization-flow, or passkey symbol in $symbols" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

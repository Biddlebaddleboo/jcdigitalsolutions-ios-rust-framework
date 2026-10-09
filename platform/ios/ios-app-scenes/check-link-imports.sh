#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS app-scenes link/import check" >&2
        exit 1
    fi
done

cat > target/ios-app-scenes-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=13.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-app-scenes-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-app-scenes --example link_check --target "$target"
    binary="$target_dir/$target/release/examples/link_check"
    imports="target/ios-app-scenes-imports-$target.txt"
    libraries="target/ios-app-scenes-libraries-$target.txt"
    symbols="target/ios-app-scenes-symbols-$target.txt"
    binary_strings="target/ios-app-scenes-strings-$target.txt"
    build_info="target/ios-app-scenes-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-app-scenes-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    strings "$binary" > "$binary_strings"
    grep -Eq '_objc_(msgSend|getClass)' "$symbols"
    grep -Fq 'UIApplication' "$binary_strings"
    grep -Fq 'connectedScenes' "$binary_strings"
    grep -Fq 'activationState' "$binary_strings"
    if grep -Eiq 'swift|SceneDelegate|setDelegate:|requestSceneSession|openSessions' "$symbols" "$binary_strings"; then
        echo "unexpected Swift runtime or scene-lifecycle/configuration API in $symbols or $binary_strings" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target="$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")"
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

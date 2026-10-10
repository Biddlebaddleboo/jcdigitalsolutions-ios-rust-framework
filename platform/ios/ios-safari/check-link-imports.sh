#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo diff find grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS SafariServices link/import check" >&2
        exit 1
    fi
done

cat > target/ios-safari-link-imports-expected.txt <<'IMPORTS'
Foundation
SafariServices
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-safari-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-safari --example ios_safari_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_safari_link_probe"
    imports="target/ios-safari-link-imports-$target.txt"
    libraries="target/ios-safari-link-libraries-$target.txt"
    symbols="target/ios-safari-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-safari-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in objc_getClass objc_msgSend; do
        if ! grep -Fq "_$symbol" "$symbols"; then
            echo "missing expected Objective-C lookup/message import _$symbol in $symbols" >&2
            exit 1
        fi
    done
    string_file="target/ios-safari-link-strings-$target.txt"
    strings "$binary" > "$string_file"
    if ! grep -Fq 'SFSafariViewController' "$string_file" || ! grep -Fq 'initWithURL:' "$string_file"; then
        echo "missing expected SafariServices class/selector metadata in $string_file" >&2
        exit 1
    fi
    if grep -Eqi 'swift_|Py[A-Z_]|WKWebView|NSURLSession|NSURLConnection|nw_|SCNetwork|_OBJC_(CLASS|METACLASS)_\$_(WKWebView|NSURLSession)|\$s[0-9]' "$symbols"; then
        echo "unexpected Swift/Python runtime, WebKit, URLSession, or Network import in $symbols" >&2
        exit 1
    fi

    build_info="target/ios-safari-link-build-$target.txt"
    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-safari -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-safari" >&2
    exit 1
fi

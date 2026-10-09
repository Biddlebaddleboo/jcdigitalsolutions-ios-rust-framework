#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS URL link/import check" >&2
        exit 1
    fi
done

cat > target/ios-url-link-imports-expected.txt <<'IMPORTS'
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    target_dir="target/ios-url-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET=17.0 CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-url --example ios_url_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_url_link_import_probe"
    imports="target/ios-url-link-imports-$target.txt"
    libraries="target/ios-url-link-libraries-$target.txt"
    symbols="target/ios-url-link-symbols-$target.txt"
    selector_strings="target/ios-url-link-strings-$target.txt"
    build_info="target/ios-url-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-url-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in _objc_alloc _objc_getClass _objc_msgSend; do
        if ! grep -Fxq "$symbol" "$symbols"; then
            echo "expected Objective-C runtime symbol $symbol was not linked in $symbols" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]|_OBJC_(CLASS|METACLASS)_\$_(UIApplication|UIView|WKWebView|SFSafariViewController)|\$s[0-9]|Sec[A-Z]|nw_|SCNetwork|NSURLSession|NSURLConnection' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated capability symbol import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$selector_strings"
    for value in NSString NSURL initWithBytes:length:encoding: URLWithString:encodingInvalidCharacters:; do
        if ! grep -Fxq "$value" "$selector_strings"; then
            echo "expected Foundation class or selector $value is absent from $selector_strings" >&2
            exit 1
        fi
    done
    if grep -Fxq 'URLWithString:' "$selector_strings"; then
        echo "older NSURL URLWithString: selector found in $selector_strings" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if ! awk -v version="$actual_deployment_target" 'BEGIN { split(version, part, "."); if ((part[1] + 0) > 17 || ((part[1] + 0) == 17 && (part[2] + 0) >= 0)) exit 0; exit 1 }'; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected iOS 17.0 or later" >&2
        exit 1
    fi
done

if find platform/ios/ios-url -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-url" >&2
    exit 1
fi

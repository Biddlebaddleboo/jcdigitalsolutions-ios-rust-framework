#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort uname vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the Core NFC link/import audit" >&2
        exit 1
    fi
done

if [ "$(uname -s)" != Darwin ]; then
    echo "the Core NFC link/import audit requires macOS" >&2
    exit 1
fi

expected_frameworks=target/ios-nfc-link-imports-expected.txt
printf '%s\n' CoreNFC Foundation > "$expected_frameworks"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=11.0; expected_platform=IOS; legacy_build_command=LC_VERSION_MIN_IPHONEOS ;;
        aarch64-apple-ios-sim) deployment_target=14.0; expected_platform=IOSSIMULATOR; legacy_build_command=LC_VERSION_MIN_IPHONESIMULATOR ;;
    esac
    dependencies="target/ios-nfc-dependencies-$target.txt"
    cargo tree --locked --offline --target "$target" --features ndef-read --edges normal --prefix none -p ios-nfc > "$dependencies"

    if ! grep -Eq '^objc2-core-nfc v[0-9]' "$dependencies"; then
        echo "objc2-core-nfc is not selected for $target" >&2
        exit 1
    fi
    if ! grep -Eq '^dispatch2 v[0-9]' "$dependencies"; then
        echo "dispatch2 callback queue support is not selected for $target" >&2
        exit 1
    fi
    if grep -Eiq '^(pyo3|cpython|python3-sys|rustpython|swift-bridge|swift-rs)([- ]|$)' "$dependencies"; then
        echo "unexpected Python or Swift binding dependency for $target" >&2
        exit 1
    fi
    if grep -E '^(ios|framework)-[^ ]+' "$dependencies" | grep -Ev '^(ios-nfc|framework-nfc|framework-core) '; then
        echo "unexpected capability crate in $target dependency graph" >&2
        exit 1
    fi

    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --offline --release -p ios-nfc --example ios_nfc_link_import_probe --features ndef-read --target "$target"
    binary="target/$target/release/examples/ios_nfc_link_import_probe"
    imports="target/ios-nfc-link-imports-$target.txt"
    frameworks="target/ios-nfc-frameworks-$target.txt"
    symbols="target/ios-nfc-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 && $1 ~ /\.framework\// { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$frameworks"
    diff -u "$expected_frameworks" "$frameworks"

    if grep -Eiq 'libswift|libpython|Python\.framework|Swift[^/]*\.framework' "$imports"; then
        echo "unexpected Swift or Python runtime library for $target" >&2
        exit 1
    fi

    nm -u "$binary" > "$symbols"
    if grep -Eiq '(^|[^[:alnum:]])(_swift[A-Za-z0-9_]*|\$s|_?Py[A-Z_][A-Za-z0-9_]*)' "$symbols"; then
        echo "unexpected Swift or Python runtime symbol for $target" >&2
        exit 1
    fi

    if actual_deployment_target=$(vtool -show-build "$binary" | awk -v expected_platform="$expected_platform" -v legacy_build_command="$legacy_build_command" '
        $1 == "Load" && $2 == "command" { build_mode = ""; platform = ""; next }
        $1 == "cmd" && $2 == "LC_BUILD_VERSION" { build_mode = "modern"; next }
        $1 == "cmd" && $2 == legacy_build_command { build_mode = "legacy"; next }
        build_mode == "modern" && $1 == "platform" { platform = $2; next }
        build_mode == "modern" && platform == expected_platform && $1 == "minos" { versions[++count] = $2; next }
        build_mode == "legacy" && $1 == "version" { versions[++count] = $2; next }
        END {
            if (count == 0) exit 1
            for (i = 1; i <= count; i++) {
                if (versions[i] !~ /^[0-9]+\.[0-9]+(\.[0-9]+)?$/) exit 1
                if (i > 1 && versions[i] != versions[1]) exit 1
            }
            print versions[1]
        }
    '); then
        :
    else
        actual_deployment_target=
    fi
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target NFC probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

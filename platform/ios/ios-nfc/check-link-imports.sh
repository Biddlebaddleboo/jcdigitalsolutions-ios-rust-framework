#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort uname; do
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

portable_dependencies=target/framework-nfc-portable-dependencies.txt
cargo tree --locked -p framework-nfc --no-default-features --edges normal --prefix none > "$portable_dependencies"
if grep -Eq '^objc2-core-nfc v[0-9]' "$portable_dependencies"; then
    echo "objc2-core-nfc leaked into the portable framework-nfc dependency graph" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    dependencies="target/ios-nfc-dependencies-$target.txt"
    cargo tree --locked --target "$target" --edges normal --prefix none -p ios-nfc > "$dependencies"

    if ! grep -Eq '^objc2-core-nfc v[0-9]' "$dependencies"; then
        echo "objc2-core-nfc is not selected for $target" >&2
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

    cargo build --locked --release -p ios-nfc --example ios_nfc_link_import_probe --target "$target"
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
done

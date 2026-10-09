#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS data link/import check" >&2
        exit 1
    fi
done

cat > target/ios-data-link-imports-expected.txt <<'IMPORTS'
CoreFoundation
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-data --example ios_data_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_data_link_import_probe"
    imports="target/ios-data-link-imports-$target.txt"
    libraries="target/ios-data-link-libraries-$target.txt"
    symbols="target/ios-data-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-data-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in _CFDataCreate _CFDataGetBytes _CFDataGetLength; do
        if ! grep -Fxq "$symbol" "$symbols"; then
            echo "expected Core Foundation data symbol $symbol was not linked in $symbols" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]|_OBJC_(CLASS|METACLASS)_\$_|\$s[0-9]|Sec[A-Z]|CFData(CreateWithBytesNoCopy|GetMutableBytePtr|SetLength|AppendBytes)|CF(MutableData|String)|NS(Mutable)?(Data|String)' "$symbols"; then
        echo "unexpected Swift/Python runtime, Objective-C, Security, mutable/no-copy data, or string symbol import in $symbols" >&2
        exit 1
    fi
done

if find platform/ios/ios-data -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-data" >&2
    exit 1
fi

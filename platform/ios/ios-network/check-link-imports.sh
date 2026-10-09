#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS network link/import check" >&2
        exit 1
    fi
done

cat > target/ios-network-link-imports-expected.txt <<'IMPORTS'
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-network --example ios_network_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_network_link_import_probe"
    imports="target/ios-network-link-imports-$target.txt"
    libraries="target/ios-network-link-libraries-$target.txt"
    symbols="target/ios-network-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-network-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi '(^|[[:space:]])_?(swift_|Py|Sec(Item|AccessControl|Key|Trust|Certificate)|nw_|SCNetwork)|_OBJC_CLASS_\$_(UIApplication|UIView|UNUserNotification)|_?\$s[0-9]' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated capability symbol import in $symbols" >&2
        exit 1
    fi
done

if find platform/ios/ios-network -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-network" >&2
    exit 1
fi

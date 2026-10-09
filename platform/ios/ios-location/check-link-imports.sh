#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS location link/import check" >&2
        exit 1
    fi
done

cat > target/ios-location-core-location-features-expected.txt <<'FEATURES'
objc2-core-location feature "CLLocation"
objc2-core-location feature "CLLocationManager"
objc2-core-location feature "CLLocationManagerDelegate"
FEATURES

cat > target/ios-location-link-imports-expected.txt <<'IMPORTS'
CoreLocation
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    feature_tree="target/ios-location-features-$target.txt"
    enabled_features="target/ios-location-core-location-features-$target.txt"
    cargo tree --locked -e features -p ios-location --target "$target" > "$feature_tree"
    grep -Eo 'objc2-core-location feature "[^"]+"' "$feature_tree" | sort -u > "$enabled_features"
    diff -u target/ios-location-core-location-features-expected.txt "$enabled_features"

    cargo build --locked --release -p ios-location --example ios_location_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_location_link_import_probe"
    imports="target/ios-location-link-imports-$target.txt"
    libraries="target/ios-location-link-libraries-$target.txt"
    symbols="target/ios-location-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-location-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|OBJC_CLASS_\$_(UIApplication|UIView|UNUserNotificationCenter|LAContext|SKStoreProductViewController)|(_OBJC_CLASS_\$_)?(UNUserNotificationCenter|LAContext|SKStoreProductViewController)' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated capability symbol import in $symbols" >&2
        exit 1
    fi
done

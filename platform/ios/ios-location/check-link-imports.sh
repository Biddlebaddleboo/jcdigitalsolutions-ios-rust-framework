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

out_of_scope_apis='startUpdatingLocation|startMonitoringForRegion|startMonitoringSignificantLocationChanges|startMonitoringVisits|startUpdatingHeading|startRangingBeacons|allowDeferredLocationUpdates|requestAlwaysAuthorization|requestTemporaryFullAccuracyAuthorization|allowsBackgroundLocationUpdates|showsBackgroundLocationIndicator|CLCircularRegion|CLRegion|CLVisit|CLHeading'
if grep -REn "$out_of_scope_apis" platform/ios/ios-location/src platform/ios/ios-location/examples; then
    echo "out-of-scope Core Location operation in B5 Rust source" >&2
    exit 1
fi
if ! grep -REq 'requestLocation\(\)' platform/ios/ios-location/src; then
    echo "B5 one-shot requestLocation call is missing" >&2
    exit 1
fi
if ! grep -REq 'requestWhenInUseAuthorization\(\)' platform/ios/ios-location/src; then
    echo "B5 foreground authorization call is missing" >&2
    exit 1
fi
if ! awk '
    /fn start_authorization_request\(/ { in_authorization_request = 1; next }
    /fn start_current\(/ { in_authorization_request = 0; in_current_request = 1; next }
    /fn new_operation_objects\(/ { in_current_request = 0 }
    /requestWhenInUseAuthorization[[:space:]]*\(/ {
        all_authorization_calls++
        if (in_authorization_request) authorization_request_calls++
        if (in_current_request) current_request_calls++
    }
    END {
        exit !(all_authorization_calls == 1 && authorization_request_calls == 1 && current_request_calls == 0)
    }
' platform/ios/ios-location/src/platform.rs; then
    echo "B5 permission prompt call must stay in the explicit authorization request path" >&2
    exit 1
fi
if ! awk '
    /pub fn accuracy_authorization\(/ { accuracy_query_count++; in_accuracy_query = 1 }
    in_accuracy_query {
        if (/objc2::available!\(ios = 14\.0, \.\.\)/) guarded_query_count++
        if (/manager\.accuracyAuthorization\(\)/) guarded_getter_count++
        line = $0
        opens = gsub(/\{/, "", line)
        closes = gsub(/\}/, "", line)
        brace_depth += opens - closes
        if (brace_depth == 0) in_accuracy_query = 0
    }
    /accuracyAuthorization\(\)/ { getter_count++ }
    END {
        exit !(accuracy_query_count == 1 && guarded_query_count == 1 && guarded_getter_count == 1 && getter_count == 1)
    }
' platform/ios/ios-location/src/platform.rs; then
    echo "B87 accuracy-authorization getter must stay inside the iOS 14 availability guard" >&2
    exit 1
fi
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
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-location-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|OBJC_CLASS_\$_(UIApplication|UIView|UNUserNotificationCenter|LAContext|SKStoreProductViewController)|(_OBJC_CLASS_\$_)?(UNUserNotificationCenter|LAContext|SKStoreProductViewController)' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated capability symbol import in $symbols" >&2
        exit 1
    fi
done

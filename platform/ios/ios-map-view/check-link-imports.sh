#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS map-view link/import check" >&2
        exit 1
    fi
done

mkdir -p target
cat > target/ios-map-view-expected-imports.txt <<'IMPORTS'
CoreFoundation
CoreLocation
Foundation
MapKit
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            sdk=iphoneos
            deployment_target=12.0
            ;;
        aarch64-apple-ios-sim)
            sdk=iphonesimulator
            deployment_target=14.0
            ;;
    esac

    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="target/ios-map-view-link-$target" cargo build --locked --release -p ios-map-view --example ios_map_view_link_probe --target "$target"
    binary="target/ios-map-view-link-$target/$target/release/examples/ios_map_view_link_probe"
    imports="target/ios-map-view-imports-$target.txt"
    libraries="target/ios-map-view-libraries-$target.txt"
    symbols="target/ios-map-view-symbols-$target.txt"
    strings_file="target/ios-map-view-strings-$target.txt"
    build_info="target/ios-map-view-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-map-view-expected-imports.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_getClass' "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|CLLocationManager|CLGeocoder|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKAnnotation|MKOverlay|showsUserLocation|userLocation' "$symbols"; then
        echo "unexpected Swift, Python, location service, search, directions, annotation, overlay, or user-location import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    grep -Fx 'MKMapView' "$strings_file"
    grep -F 'setRegion:animated:' "$strings_file" >/dev/null
    grep -F 'region' "$strings_file" >/dev/null
    if grep -Eqi 'swift_|CLLocationManager|CLGeocoder|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKAnnotation|MKOverlay|showsUserLocation|userLocation' "$strings_file"; then
        echo "unexpected Swift, location service, search, directions, annotation, overlay, or user-location selector in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

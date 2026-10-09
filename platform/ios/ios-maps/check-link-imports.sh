#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS maps link/import check" >&2
        exit 1
    fi
done

cat > target/ios-maps-link-imports-expected.txt <<'IMPORTS'
ARKit
CoreLocation
Foundation
MapKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=12.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            ;;
    esac

    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="target/ios-maps-link-$target" cargo build --locked --release -p ios-maps --example ios_maps_link_probe --target "$target"
    binary="target/ios-maps-link-$target/$target/release/examples/ios_maps_link_probe"
    imports="target/ios-maps-link-imports-$target.txt"
    libraries="target/ios-maps-link-libraries-$target.txt"
    symbols="target/ios-maps-link-symbols-$target.txt"
    strings_file="target/ios-maps-strings-$target.txt"
    build_info="target/ios-maps-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-maps-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    grep -q '_MKMapPointForCoordinate' "$symbols"
    grep -q '_MKCoordinateForMapPoint' "$symbols"
    grep -q '_MKMetersBetweenMapPoints' "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|ARSession|ARFrame|ARCamera|AVCapture|MKMapView|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|CLLocationManager|CLGeocoder|UIApplication|UIView|UIWindow' "$symbols"; then
        echo "unexpected runtime, session, frame, camera, MapKit service, location service, or UI symbol import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    grep -q 'ARWorldTrackingConfiguration' "$strings_file"
    grep -q 'isSupported' "$strings_file"
    if grep -Eqi 'requestAccessForMediaType|ARSession|ARFrame|ARCamera|AVCapture|MKMapView|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|CLLocationManager|CLGeocoder|UIApplication|UIView|UIWindow|swift_' "$strings_file"; then
        echo "unexpected camera, session, frame, MapKit service, location service, UI, or Swift selector in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

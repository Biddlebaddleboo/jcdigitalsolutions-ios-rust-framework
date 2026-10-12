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

grep -F 'fn duplicate_ids_reject_snapshot_before_native_changes()' platform/ios/ios-map-view/src/annotations.rs >/dev/null
grep -F 'fn route_rejects_empty_and_single_point_paths()' platform/ios/ios-map-view/src/route.rs >/dev/null
grep -F 'fn route_rejects_paths_over_the_bounded_coordinate_limit()' platform/ios/ios-map-view/src/route.rs >/dev/null
grep -F 'fn route_delegate_policy_accepts_only_empty_or_owned_slots()' platform/ios/ios-map-view/src/route.rs >/dev/null
grep -F 'fn none_mode_is_allowed_without_location_authorization()' platform/ios/ios-map-view/src/tracking.rs >/dev/null
grep -F 'fn follow_modes_reject_unavailable_authorization_states()' platform/ios/ios-map-view/src/tracking.rs >/dev/null
grep -F 'pub fn set_user_tracking_mode(' platform/ios/ios-map-view/src/platform.rs >/dev/null
grep -F 'self.apply_user_tracking_mode(MKUserTrackingMode::None, false, false);' platform/ios/ios-map-view/src/platform.rs >/dev/null
grep -F 'authorizationStatus_class' platform/ios/ios-map-view/src/tracking.rs >/dev/null
grep -F 'return Err(MapRouteError::DelegateSlotOccupied);' platform/ios/ios-map-view/src/platform.rs >/dev/null
grep -F 'pub fn clear_route(&mut self)' platform/ios/ios-map-view/src/platform.rs >/dev/null
grep -F 'impl Drop for IosMapView' platform/ios/ios-map-view/src/platform.rs >/dev/null
if grep -E 'requestWhenInUseAuthorization|requestAlwaysAuthorization|requestTemporaryFullAccuracyAuthorization' platform/ios/ios-map-view/src/platform.rs platform/ios/ios-map-view/src/tracking.rs; then
    echo "ios-map-view must not request or elevate location authorization" >&2
    exit 1
fi

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
    if grep -Eqi 'swift_|Py[A-Z_]|CLGeocoder|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKAnnotationView|MKTileOverlay' "$symbols"; then
        echo "unexpected Swift, Python, geocoder, search, directions, custom annotation, or tile overlay import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    grep -Fx 'MKMapView' "$strings_file"
    grep -Fx 'MKPointAnnotation' "$strings_file"
    grep -F 'setRegion:animated:' "$strings_file" >/dev/null
    grep -F 'region' "$strings_file" >/dev/null
    grep -F 'addAnnotation:' "$strings_file" >/dev/null
    grep -F 'removeAnnotation:' "$strings_file" >/dev/null
    grep -Fx 'MKPolyline' "$strings_file" >/dev/null
    grep -Fx 'MKPolylineRenderer' "$strings_file" >/dev/null
    grep -F 'polylineWithCoordinates:count:' "$strings_file" >/dev/null
    grep -F 'mapView:rendererForOverlay:' "$strings_file" >/dev/null
    grep -F 'addOverlay:' "$strings_file" >/dev/null
    grep -F 'removeOverlay:' "$strings_file" >/dev/null
    grep -F 'blueColor' "$strings_file" >/dev/null
    grep -F 'setStrokeColor:' "$strings_file" >/dev/null
    grep -F 'setLineWidth:' "$strings_file" >/dev/null
    grep -F 'setCoordinate:' "$strings_file" >/dev/null
    grep -F 'setTitle:' "$strings_file" >/dev/null
    grep -Fx 'CLLocationManager' "$strings_file" >/dev/null
    grep -F 'authorizationStatus' "$strings_file" >/dev/null
    grep -F 'setShowsUserLocation:' "$strings_file" >/dev/null
    grep -F 'setUserTrackingMode:animated:' "$strings_file" >/dev/null
    if grep -Eqi 'requestWhenInUseAuthorization|requestAlwaysAuthorization|requestTemporaryFullAccuracyAuthorization' "$strings_file"; then
        echo "unexpected location authorization request selector in $strings_file" >&2
        exit 1
    fi
    if grep -Eqi 'swift_|CLGeocoder|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKAnnotationView|MKTileOverlay' "$strings_file"; then
        echo "unexpected Swift, geocoder, search, directions, custom annotation, or tile overlay selector in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

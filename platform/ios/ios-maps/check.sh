#!/bin/sh
set -eu

if ! command -v rg >/dev/null 2>&1; then
    echo "rg is required for the zero-Swift-source check" >&2
    exit 1
fi

if rg --files crates/framework-maps platform/ios/ios-maps -g '*.swift' | grep -q .; then
    echo "Swift source is out of scope for framework-maps and ios-maps" >&2
    exit 1
fi

cargo fmt --manifest-path crates/framework-maps/Cargo.toml -- --check
cargo fmt --manifest-path platform/ios/ios-maps/Cargo.toml -- --check
cargo check --locked --no-default-features -p framework-maps
cargo clippy --locked --lib --no-default-features -p framework-maps -- -D warnings
cargo doc --locked --no-deps -p framework-maps
cargo check --locked --lib -p ios-maps
cargo clippy --locked --lib -p ios-maps -- -D warnings

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked --lib -p ios-maps --target "$target"
    cargo clippy --locked --lib -p ios-maps --target "$target" -- -D warnings
done

cargo doc --locked --no-deps -p ios-maps

features=$(cargo tree --locked -p ios-maps --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-ar-kit feature "ARConfiguration"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-ar-kit feature "objc2"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-map-kit feature "MKGeometry"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-map-kit feature "objc2-core-location"' >/dev/null
printf '%s\n' "$features" | grep -F 'objc2-core-location feature "CLLocation"' >/dev/null
if printf '%s\n' "$features" | grep -E 'objc2-ar-kit feature "(default|ARSession|ARFrame|ARCamera|ARKitUI|ARKitCore|ARKitFoundation)"'; then
    echo "out-of-scope ARKit features are enabled" >&2
    exit 1
fi
if printf '%s\n' "$features" | grep -E 'objc2-map-kit feature "(default|MKMapView|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKLocalSearchCompleter|MKGeocodingRequest)"'; then
    echo "out-of-scope MapKit features are enabled" >&2
    exit 1
fi

sh platform/ios/ios-maps/check-link-imports.sh

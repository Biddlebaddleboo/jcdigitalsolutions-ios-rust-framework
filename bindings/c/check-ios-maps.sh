#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target
sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for this audit: $llvm_nm" >&2
    exit 1
fi

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-maps.sh
cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_maps as $maps
    | $maps.cargo_feature == "ios-maps"
    and $maps.header == "framework_ios_maps.h"
    and ($maps.symbols | sort) == [
        "framework_ios_maps_coordinate_for_map_point",
        "framework_ios_maps_map_point_for_coordinate",
        "framework_ios_maps_meters_between_map_points"
    ]
    and $maps.link_probe_deployment_minimums["aarch64-apple-ios"] == "12.0"
    and $maps.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($maps.api | contains("iOS 4.0"))
    and ($maps.status_mapping.unavailable | contains("outputs remain zero"))
    and ($maps.ownership.outputs | contains("non-overlapping"))
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_maps.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-ios-maps-expected-symbols.txt
rg -o 'framework_ios_maps_[A-Za-z0-9_]+' bindings/c/include/framework_ios_maps.h \
    | sort -u > target/framework-c-ios-maps-header-symbols.txt
diff -u target/framework-c-ios-maps-expected-symbols.txt \
    target/framework-c-ios-maps-header-symbols.txt

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-maps-default-tree.txt
if rg -q 'framework-maps|ios-maps|objc2-map-kit|MapKit' \
    target/framework-c-ios-maps-default-tree.txt; then
    echo "MapKit dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-maps \
    > target/framework-c-ios-maps-ios-tree.txt
rg -q 'framework-maps' target/framework-c-ios-maps-ios-tree.txt
rg -q 'ios-maps' target/framework-c-ios-maps-ios-tree.txt
rg -q 'objc2-map-kit feature "MKGeometry"' target/framework-c-ios-maps-ios-tree.txt
rg -q 'objc2-map-kit feature "objc2-core-location"' target/framework-c-ios-maps-ios-tree.txt
rg -q 'objc2-core-location feature "CLLocation"' target/framework-c-ios-maps-ios-tree.txt
if rg -q 'objc2-map-kit feature "(default|MKMapView|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKLocalSearchCompleter|MKGeocodingRequest)"' \
    target/framework-c-ios-maps-ios-tree.txt; then
    echo "out-of-scope MapKit UI or service feature leaked into F13" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-maps \
    > target/framework-c-ios-maps-host-tree.txt
if rg -q 'ios-maps|objc2-map-kit|MapKit|ARKit|CoreLocation' \
    target/framework-c-ios-maps-host-tree.txt; then
    echo "Apple Maps dependencies leaked into the non-iOS feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-maps
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-maps -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-maps

cat > target/framework-c-ios-maps-c.c <<'FIXTURE_C'
#include <framework_ios_maps.h>
int main(void) {
    double x = 0.0;
    double y = 0.0;
    double latitude = 0.0;
    double longitude = 0.0;
    FrameworkIosMapsDistanceMeters meters = 0.0;
    FrameworkStatus a = framework_ios_maps_map_point_for_coordinate(0.0, 0.0, &x, &y);
    FrameworkStatus b = framework_ios_maps_coordinate_for_map_point(x, y, &latitude, &longitude);
    FrameworkStatus c = framework_ios_maps_meters_between_map_points(x, y, x, y, &meters);
    return (int)(a | b | c);
}
FIXTURE_C
cat > target/framework-c-ios-maps-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_maps.h>
static_assert(sizeof(FrameworkIosMapsDistanceMeters) == sizeof(double), "distance ABI width");
int main() {
    double x = 0.0;
    double y = 0.0;
    double latitude = 0.0;
    double longitude = 0.0;
    FrameworkIosMapsDistanceMeters meters = 0.0;
    FrameworkStatus a = framework_ios_maps_map_point_for_coordinate(0.0, 0.0, &x, &y);
    FrameworkStatus b = framework_ios_maps_coordinate_for_map_point(x, y, &latitude, &longitude);
    FrameworkStatus c = framework_ios_maps_meters_between_map_points(x, y, x, y, &meters);
    return static_cast<int>(a | b | c);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-maps-c.c "$host_archive" -o target/framework-c-ios-maps-c-host
# These fixtures use only C ABI declarations; avoid libc++ headers at the iOS 12 compile floor.
clang++ -nostdinc++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-maps-cpp.cpp "$host_archive" -o target/framework-c-ios-maps-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_maps_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-maps-host-symbols.txt
diff -u target/framework-c-ios-maps-expected-symbols.txt \
    target/framework-c-ios-maps-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null > target/framework-c-ios-maps-host-undefined.txt
if rg -qi 'MapKit|MKMap|objc_msgSend|OBJC_CLASS|objc2|ARKit|CoreLocation' \
    target/framework-c-ios-maps-host-undefined.txt; then
    echo "Apple Maps or Objective-C import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0; sdk=iphoneos; clang_target=arm64-apple-ios12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-maps --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-maps \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-maps --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-maps-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-maps-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-maps-$language-$target"
        case "$language" in
            c)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
                    -Wall -Wextra -Werror -pedantic -I bindings/c/include \
                    "$source" "$archive" -framework MapKit -o "$binary"
                ;;
            cpp)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -nostdinc++ \
                    -std="$standard" -Wall -Wextra -Werror -pedantic -I bindings/c/include \
                    "$source" "$archive" -framework MapKit -o "$binary"
                ;;
        esac
        libraries="target/framework-c-ios-maps-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_maps.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-maps-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-maps-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-maps-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        for symbol in MKMapPointForCoordinate MKCoordinateForMapPoint MKMetersBetweenMapPoints; do
            rg -q "_$symbol" "$symbols"
        done
        if rg -qi 'MKMapView|MKLocalSearch|MKDirections|MKMapItem|MKMapSnapshotter|MKLocalSearchCompleter|MKGeocodingRequest|CLLocationManager|CLGeocoder|UIApplication|UIView|UIWindow|objc_msgSend|OBJC_CLASS|swift_' "$symbols"; then
            echo "out-of-scope MapKit service, location, Objective-C, UI, or Swift symbol import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-maps-$language-$target-build.txt"
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_maps_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u > "target/framework-c-ios-maps-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-maps-expected-symbols.txt \
        "target/framework-c-ios-maps-$target-archive-symbols.txt"
done

printf 'F13 checks complete; no tests or consumer binaries were executed\n'

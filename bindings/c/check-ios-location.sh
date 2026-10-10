#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-location.sh
cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_location as $location
    | $location.cargo_feature == "ios-location"
    and $location.header == "framework_ios_location.h"
    and ($location.symbols | sort) == [
        "framework_ios_location_authorization_query_start",
        "framework_ios_location_authorization_request_start",
        "framework_ios_location_availability",
        "framework_ios_location_current_start",
        "framework_ios_location_operation_cancel",
        "framework_ios_location_operation_destroy",
        "framework_ios_location_operation_poll"
    ]
    and $location.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $location.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and $location.FrameworkIosLocationResultV1.size == 64
    and $location.FrameworkIosLocationResultV1.align == 8
    and ($location.api | contains("requestLocation()"))
    and ($location.status_mapping.pointer_validation | contains("valid, aligned, and writable"))
    and ($location.ownership.readiness_callback | contains("nonterminal"))
    and ($location.threading | contains("main thread"))
' bindings/c/abi-manifest.json > /dev/null
rg -q 'Pointer preconditions are caller obligations' bindings/c/include/framework_ios_location.h
rg -q 'Availability output must not overlap a live iOS operation handle' bindings/c/include/framework_ios_location.h
rg -q 'must not contain a live handle on entry' bindings/c/include/framework_ios_location.h
rg -q 'overlap live operation storage' bindings/c/include/framework_ios_location.h
rg -q 'do not copy or alias a handle' bindings/c/include/framework_ios_location.h

jq -r '.optional_capabilities.ios_location.symbols[]' \
    bindings/c/abi-manifest.json | sort -u \
    > target/framework-c-ios-location-expected-symbols.txt
rg -o 'framework_ios_location_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_location.h | sort -u \
    > target/framework-c-ios-location-header-symbols.txt
diff -u target/framework-c-ios-location-expected-symbols.txt \
    target/framework-c-ios-location-header-symbols.txt

{
    cat <<'ASSERT_HEADER'
#if defined(__cplusplus)
#define FRAMEWORK_LOCATION_STATIC_ASSERT static_assert
#define FRAMEWORK_LOCATION_ALIGNOF alignof
#else
#define FRAMEWORK_LOCATION_STATIC_ASSERT _Static_assert
#define FRAMEWORK_LOCATION_ALIGNOF _Alignof
#endif
ASSERT_HEADER
    for tag_group in availability_tags authorization_tags operation_kinds; do
        jq -r --arg group "$tag_group" '
            .optional_capabilities.ios_location[$group]
            | to_entries[]
            | "FRAMEWORK_LOCATION_STATIC_ASSERT(" + .key + " == " + (.value | tostring)
              + ", \"" + .key + " ABI value\");"
        ' bindings/c/abi-manifest.json
    done
    jq -r '
        .optional_capabilities.ios_location.FrameworkIosLocationResultV1 as $record
        | "FRAMEWORK_LOCATION_STATIC_ASSERT(sizeof(FrameworkIosLocationResultV1) == "
          + ($record.size | tostring) + ", \"result size\");"
        + "\nFRAMEWORK_LOCATION_STATIC_ASSERT(FRAMEWORK_LOCATION_ALIGNOF(FrameworkIosLocationResultV1) == "
          + ($record.align | tostring) + ", \"result alignment\");"
        + ( $record.fields | to_entries | map(
            "\nFRAMEWORK_LOCATION_STATIC_ASSERT(offsetof(FrameworkIosLocationResultV1, "
            + .key + ") == " + (.value | tostring) + ", \"" + .key + " offset\");"
          ) | join("") )
    ' bindings/c/abi-manifest.json
} > target/framework-c-ios-location-manifest-asserts.h

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-location-default-tree.txt
if rg -q 'framework-location|ios-location|objc2-core-location|CoreLocation' \
    target/framework-c-ios-location-default-tree.txt; then
    echo "location dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-location \
    > target/framework-c-ios-location-ios-tree.txt
rg -q 'framework-location' target/framework-c-ios-location-ios-tree.txt
rg -q 'ios-location' target/framework-c-ios-location-ios-tree.txt
rg -q 'ios-runtime' target/framework-c-ios-location-ios-tree.txt
rg -q 'objc2-core-location feature "CLLocation"' \
    target/framework-c-ios-location-ios-tree.txt
rg -q 'objc2-core-location feature "CLLocationManager"' \
    target/framework-c-ios-location-ios-tree.txt
rg -q 'objc2-core-location feature "CLLocationManagerDelegate"' \
    target/framework-c-ios-location-ios-tree.txt
rg -q 'objc2-foundation feature "NSArray"' target/framework-c-ios-location-ios-tree.txt
rg -q 'objc2-foundation feature "NSDate"' target/framework-c-ios-location-ios-tree.txt
rg -q 'objc2-foundation feature "NSObject"' target/framework-c-ios-location-ios-tree.txt
if rg -q 'objc2-core-location feature "(CLGeocoder|CLHeading|CLMonitor|CLVisit|CLBeaconRegion|default)"' \
    target/framework-c-ios-location-ios-tree.txt; then
    echo "out-of-scope Core Location services leaked into F15" >&2
    exit 1
fi
if rg -q 'requestAlwaysAuthorization|requestTemporaryFullAccuracyAuthorization|startUpdatingLocation|CLGeocoder|CLVisit|CLHeading' \
    bindings/c/src/ios_location.rs; then
    echo "out-of-scope location operation leaked into F15" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-location \
    > target/framework-c-ios-location-host-tree.txt
if rg -q '(^|[[:space:]])ios-location v|objc2-core-location|CoreLocation|Foundation.framework' \
    target/framework-c-ios-location-host-tree.txt; then
    echo "Core Location dependencies leaked into the host feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-location
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-location -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-location

cat > target/framework-c-ios-location-c.c <<'FIXTURE_C'
#include <stddef.h>
#include <framework_ios_location.h>
#include "framework-c-ios-location-manifest-asserts.h"
_Static_assert(sizeof(FrameworkIosLocationAvailability) == sizeof(uint32_t), "availability width");
_Static_assert(sizeof(FrameworkIosLocationAuthorization) == sizeof(uint32_t), "authorization width");
_Static_assert(sizeof(FrameworkIosLocationOperationKind) == sizeof(uint32_t), "operation kind width");
static void ready(void *context) { (void)context; }
int main(void) {
    FrameworkIosLocationAvailability availability = FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN;
    FrameworkIosLocationOperation *operation = NULL;
    FrameworkIosLocationResultV1 result = {0};
    uint8_t is_ready = 0;
    FrameworkStatus status = framework_ios_location_availability(&availability);
    status |= framework_ios_location_authorization_query_start(ready, NULL, &operation);
    status |= framework_ios_location_operation_poll(operation, &is_ready, &result);
    status |= framework_ios_location_operation_cancel(operation);
    status |= framework_ios_location_operation_destroy(&operation);
    status |= framework_ios_location_authorization_request_start(ready, NULL, &operation);
    status |= framework_ios_location_current_start(0.0, ready, NULL, &operation);
    return (int)status;
}
FIXTURE_C

cat > target/framework-c-ios-location-cpp.cpp <<'FIXTURE_CPP'
#include <stddef.h>
#include <framework_ios_location.h>
#include "framework-c-ios-location-manifest-asserts.h"
static_assert(sizeof(FrameworkIosLocationAvailability) == sizeof(uint32_t), "availability width");
static_assert(sizeof(FrameworkIosLocationAuthorization) == sizeof(uint32_t), "authorization width");
static_assert(sizeof(FrameworkIosLocationOperationKind) == sizeof(uint32_t), "operation kind width");
static void ready(void *context) { (void)context; }
int main() {
    FrameworkIosLocationAvailability availability = FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN;
    FrameworkIosLocationOperation *operation = nullptr;
    FrameworkIosLocationResultV1 result{};
    uint8_t is_ready = 0;
    FrameworkStatus status = framework_ios_location_availability(&availability);
    status |= framework_ios_location_authorization_query_start(ready, nullptr, &operation);
    status |= framework_ios_location_operation_poll(operation, &is_ready, &result);
    status |= framework_ios_location_operation_cancel(operation);
    status |= framework_ios_location_operation_destroy(&operation);
    status |= framework_ios_location_authorization_request_start(ready, nullptr, &operation);
    status |= framework_ios_location_current_start(0.0, ready, nullptr, &operation);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-ios-location-c.c "$host_archive" \
    -o target/framework-c-ios-location-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-ios-location-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-location-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_location_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-ios-location-host-symbols.txt
diff -u target/framework-c-ios-location-expected-symbols.txt \
    target/framework-c-ios-location-host-symbols.txt
nm -u "$host_archive" 2>/dev/null > target/framework-c-ios-location-host-undefined.txt
if rg -q 'CoreLocation|CLLocationManager|objc_msgSend|OBJC_CLASS|UIKit|swift_|Py[A-Z_]' \
    target/framework-c-ios-location-host-undefined.txt; then
    echo "Apple, Swift, or Python import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0; sdk=iphoneos; clang_target=arm64-apple-ios10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-location --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-location \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-location --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-location-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-location-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-location-$language-$target"
        case "$language" in
            c)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
                    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
                    "$source" "$archive" -framework CoreLocation -framework Foundation -lobjc \
                    -o "$binary"
                ;;
            cpp)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -nostdinc++ \
                    -std="$standard" -Wall -Wextra -Werror -pedantic \
                    -I bindings/c/include -I target "$source" "$archive" \
                    -framework CoreLocation -framework Foundation -lobjc -o "$binary"
                ;;
        esac
        libraries="target/framework-c-ios-location-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_location.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-location-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-location-$language-imports-expected.txt" "$libraries"
        symbols="target/framework-c-ios-location-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_getClass' "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        strings "$binary" | rg -q '^CLLocationManager$'
        strings "$binary" | rg -q '^requestLocation$'
        if rg -q 'CLGeocoder|CLHeading|CLMonitor|CLVisit|CLBeaconRegion|UIApplication|UIView|UIKit|Photos|CoreMotion|swift_|Py[A-Z_]' "$symbols"; then
            echo "out-of-scope Apple capability, Swift, or Python import in $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-location-$language-$target-build.txt"
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_location_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u \
        > "target/framework-c-ios-location-$target-archive-symbols.txt"
    diff -u target/framework-c-ios-location-expected-symbols.txt \
        "target/framework-c-ios-location-$target-archive-symbols.txt"
done

printf 'F15 checks complete; no tests or consumer binaries were executed\n'

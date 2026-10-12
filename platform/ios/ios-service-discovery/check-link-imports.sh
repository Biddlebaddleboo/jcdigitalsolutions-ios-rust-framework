#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS Bonjour discovery link/import check" >&2
        exit 1
    fi
done

cat > target/ios-service-discovery-imports-expected.txt <<'IMPORTS'
Network
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=13.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-service-discovery-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release --example ios_bonjour_link_probe -p ios-service-discovery --target "$target"

    binary="$target_dir/$target/release/examples/ios_bonjour_link_probe"
    imports="target/ios-service-discovery-imports-$target.txt"
    libraries="target/ios-service-discovery-libraries-$target.txt"
    symbols="target/ios-service-discovery-symbols-$target.txt"
    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-service-discovery-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    required_symbols='nw_browse_descriptor_create_bonjour_service nw_browse_descriptor_set_include_txt_record nw_browser_create nw_browser_set_queue nw_browser_set_state_changed_handler nw_browser_set_browse_results_changed_handler nw_browser_start nw_browser_cancel nw_browse_result_get_changes nw_browse_result_copy_endpoint nw_endpoint_get_type nw_endpoint_get_bonjour_service_name nw_endpoint_get_bonjour_service_type nw_endpoint_get_bonjour_service_domain nw_error_get_error_domain nw_error_get_error_code nw_release dispatch_queue_create dispatch_async pthread_mutex_lock pthread_mutex_unlock'
    for symbol in $required_symbols; do
        if ! grep -q "_$symbol" "$symbols"; then
            echo "missing expected Network.framework/Dispatch/libSystem import _$symbol in $symbols" >&2
            exit 1
        fi
    done
    if grep -Eq 'swift_|Py[A-Z_]|_OBJC_(CLASS|METACLASS)_|NetServiceBrowser|nw_listener|nw_connection|nw_path_monitor|nw_advertise|nw_udp|nw_parameters_create_secure' "$symbols" || grep -Fq '$s' "$symbols"; then
        echo "unexpected Swift/Python/Objective-C, deprecated browser, or unrelated Network API import in $symbols" >&2
        exit 1
    fi

    build_info="target/ios-service-discovery-build-$target.txt"
    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "Bonjour probe $target binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-service-discovery -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-service-discovery" >&2
    exit 1
fi

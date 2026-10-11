#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS connection link/import check" >&2
        exit 1
    fi
done

cat > target/ios-connection-link-imports-expected.txt <<'IMPORTS'
Network
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-connection-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release --examples -p ios-connection --target "$target"

    for probe in ios_connection_link_import_probe ios_tcp_listener_link_probe; do
        binary="$target_dir/$target/release/examples/$probe"
        imports="target/ios-connection-link-imports-$probe-$target.txt"
        libraries="target/ios-connection-link-libraries-$probe-$target.txt"
        symbols="target/ios-connection-link-symbols-$probe-$target.txt"

        otool -L "$binary" > "$imports"
        awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
        diff -u target/ios-connection-link-imports-expected.txt "$libraries"

        nm -u "$binary" > "$symbols"
        case "$probe" in
            ios_connection_link_import_probe)
                required_symbols='nw_endpoint_create_host nw_parameters_create_secure_tcp nw_connection_create nw_connection_set_state_changed_handler nw_connection_set_queue nw_connection_start nw_connection_send nw_connection_receive nw_connection_cancel nw_content_context_get_is_final nw_error_get_error_domain nw_error_get_error_code'
                forbidden_network='nw_listener|nw_browse|nw_advertise|nw_path_monitor|nw_framer|nw_protocol|nw_resolver|nw_service|nw_path_copy|nw_path_enumerate|nw_path_has|nw_path_is|nw_path_uses'
                ;;
            ios_tcp_listener_link_probe)
                required_symbols='nw_parameters_create nw_parameters_copy_default_protocol_stack nw_protocol_stack_set_transport_protocol nw_tcp_create_options nw_listener_create_with_port nw_listener_set_queue nw_listener_set_state_changed_handler nw_listener_set_new_connection_handler nw_listener_start nw_listener_get_port nw_listener_cancel nw_connection_set_state_changed_handler nw_connection_set_queue nw_connection_start nw_connection_send nw_connection_receive nw_connection_cancel nw_content_context_get_is_final nw_error_get_error_domain nw_error_get_error_code'
                forbidden_network='nw_browse|nw_advertise|nw_path_monitor|nw_framer|nw_resolver|nw_service|nw_path_copy|nw_path_enumerate|nw_path_has|nw_path_is|nw_path_uses|nw_udp_create_options|nw_parameters_create_secure_udp|nw_listener_create_with_launchd_key|nw_listener_set_new_connection_group_handler|nw_listener_set_advertised_endpoint_changed_handler'
                ;;
        esac
        for symbol in $required_symbols; do
            if ! grep -q "_$symbol" "$symbols"; then
                echo "missing expected Network.framework import _$symbol in $symbols" >&2
                exit 1
            fi
        done
        if grep -Eq "swift_|Py[A-Z_]|Sec(Item|AccessControl|Key|Trust|Certificate)|$forbidden_network|_OBJC_(CLASS|METACLASS)_" "$symbols" || grep -Fq '$s' "$symbols"; then
            echo "unexpected Swift/Python runtime, Objective-C class, or unrelated Network API import in $symbols" >&2
            exit 1
        fi

        build_info="target/ios-connection-link-build-$probe-$target.txt"
        vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$probe $target binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
    done
done

if find platform/ios/ios-connection -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-connection" >&2
    exit 1
fi

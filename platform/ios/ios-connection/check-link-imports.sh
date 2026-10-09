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
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-connection --example ios_connection_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_connection_link_import_probe"
    imports="target/ios-connection-link-imports-$target.txt"
    libraries="target/ios-connection-link-libraries-$target.txt"
    symbols="target/ios-connection-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-connection-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in nw_endpoint_create_host nw_parameters_create_secure_tcp nw_connection_create nw_connection_set_state_changed_handler nw_connection_set_queue nw_connection_start nw_connection_send nw_connection_receive nw_connection_cancel nw_content_context_get_is_final nw_error_get_error_domain nw_error_get_error_code; do
        if ! grep -q "_$symbol" "$symbols"; then
            echo "missing expected Network.framework import _$symbol in $symbols" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]|Sec(Item|AccessControl|Key|Trust|Certificate)|nw_(browse|listener|path_monitor|framer|protocol|resolver|service|path_copy|path_enumerate|path_has|path_is|path_uses)|_OBJC_(CLASS|METACLASS)_\$_|\$s[0-9]' "$symbols"; then
        echo "unexpected Swift/Python runtime, Objective-C class, or unrelated capability import in $symbols" >&2
        exit 1
    fi

    build_info="target/ios-connection-link-build-$target.txt"
    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-connection -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-connection" >&2
    exit 1
fi

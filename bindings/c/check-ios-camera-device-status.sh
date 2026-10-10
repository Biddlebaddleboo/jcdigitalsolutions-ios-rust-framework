#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

source=bindings/c/src/ios_camera_device_status.rs
header=bindings/c/include/framework_ios_camera_device_status.h
guide=docs/bindings/ios-camera-device-status.md
plan=PLAN_BINDINGS_F26.md

sh -n bindings/c/check-ios-camera-device-status.sh
sh -n bindings/c/check-ios-camera-device-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_camera_device_status_has_default_video_capture_device
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'out_present.write\(0\)' "$source"
rg -q 'valid, properly aligned writable memory for one' "$source"
rg -q 'duration of the synchronous call' "$source"
rg -q 'prevent unsynchronized' "$source"
rg -q 'concurrent access' "$source"
rg -q 'checks only nullness' "$source"
rg -q 'does not retain' "$source"
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
rg -q 'checks only' "$header"
rg -q 'any non-null pointer must actually address valid, properly' "$header"
rg -q 'aligned writable memory for the synchronous call' "$header"
rg -q 'zero before platform' "$header"
rg -q 'not retained' "$header"
for contract_file in "$guide" "$plan"; do
    rg -q 'valid' "$contract_file"
    rg -q 'aligned writable' "$contract_file"
    rg -q 'synchronous call' "$contract_file"
    rg -q 'unsynchronized concurrent access' "$contract_file"
    rg -q 'retain' "$contract_file"
    rg -q 'zero before platform' "$contract_file"
done

if jq -e '.optional_capabilities.ios_camera_device_status' \
    bindings/c/abi-manifest.json > /dev/null 2>&1; then
    jq -e '
        .optional_capabilities.ios_camera_device_status as $camera
        | $camera.cargo_feature == "ios-camera-device-status"
        and $camera.header == "framework_ios_camera_device_status.h"
        and $camera.symbols == ["framework_ios_camera_device_status_has_default_video_capture_device"]
        and ($camera.api | contains("iOS 4.0"))
        and $camera.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
        and $camera.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
        and ($camera.status_mapping.success | contains("0 or 1"))
        and ($camera.status_mapping.non_ios | contains("UNSUPPORTED"))
        and ($camera.status_mapping.panic | contains("PANIC"))
        and ($camera.ownership.output | contains("not retained"))
        and ($camera.ownership.output | contains("nullness is checked only"))
        and ($camera.ownership.output | contains("concurrently without synchronization"))
        and ($camera.direct_imports_64_bit_ios.c | sort) == [
            "AVFoundation", "libSystem.B.dylib", "libobjc.A.dylib"
        ]
        and ($camera.direct_imports_64_bit_ios.cpp | sort) == [
            "AVFoundation", "libSystem.B.dylib", "libobjc.A.dylib"
        ]
    ' bindings/c/abi-manifest.json > /dev/null
else
    printf 'F26 ABI manifest contract pending root wiring\n'
fi

if rg -n '[[:blank:]]+$' \
    "$plan" "$guide" \
    "$source" "$header" bindings/c/check-ios-camera-device-status.sh \
    bindings/c/check-ios-camera-device-status-link.sh; then
    echo "F26 file has trailing whitespace" >&2
    exit 1
fi

printf '#include <framework_ios_camera_device_status.h>\n' | \
    clang -std=c11 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c -fsyntax-only -
printf '#include <stddef.h>\n#include <framework_ios_camera_device_status.h>\n' | \
    clang++ -std=c++17 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c++ -fsyntax-only -

printf 'F26 static checks complete; no Cargo build, link, test, consumer, or probe was run\n'

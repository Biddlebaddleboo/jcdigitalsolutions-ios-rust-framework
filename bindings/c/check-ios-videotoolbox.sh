#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

source=bindings/c/src/ios_videotoolbox.rs
header=bindings/c/include/framework_ios_videotoolbox.h
guide=docs/bindings/ios-videotoolbox.md
plan=PLAN_BINDINGS_F25.md

sh -n bindings/c/check-ios-videotoolbox.sh
sh -n bindings/c/check-ios-videotoolbox-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_videotoolbox_hardware_decode_supported
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'FrameworkStatus::UNAVAILABLE' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
rg -q 'catch_unwind_status' "$source"
rg -q 'out_supported.write\(0\)' "$source"
rg -U -Fq 'unsafe { out_supported.write(0) };

    catch_unwind_status' "$source"
rg -U -Fq 'if out_supported.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }' "$source"
awk '
    /if out_supported\.is_null\(\)/ { null_check = NR }
    /out_supported\.write\(0\)/ && !zero_write { zero_write = NR }
    /catch_unwind_status\(AssertUnwindSafe/ { platform_boundary = NR }
    END {
        if (!(null_check < zero_write && zero_write < platform_boundary)) exit 1
    }
' "$source" || {
    echo "F25 output initialization order mismatch" >&2
    exit 1
}
for contract_file in "$source" "$header" "$guide" "$plan"; do
    rg -Fq 'valid' "$contract_file"
    rg -Fq 'properly aligned' "$contract_file"
    rg -Fq 'writable byte' "$contract_file"
    rg -Fq 'full' "$contract_file"
    rg -Fq 'synchronous call' "$contract_file"
    rg -Fq 'caller must prevent' "$contract_file"
    rg -Fq 'unsynchronized' "$contract_file"
    rg -Fq 'concurrent access' "$contract_file"
    rg -Fq 'only nullness' "$contract_file"
    rg -Fq 'does not' "$contract_file"
    rg -Fq 'retain the output pointer' "$contract_file"
    rg -Fq 'output pointer' "$contract_file"
done
for contract_file in "$header" "$guide" "$plan"; do
    rg -Fq 'null output pointer returns' "$contract_file"
    rg -Fq 'FRAMEWORK_STATUS_INVALID_ARGUMENT' "$contract_file"
    rg -Fq 'without a write' "$contract_file"
    rg -Fq 'zero before platform' "$contract_file"
done

if jq -e '.optional_capabilities.ios_videotoolbox' \
    bindings/c/abi-manifest.json > /dev/null 2>&1; then
    jq -e '
        .optional_capabilities.ios_videotoolbox as $video
        | $video.cargo_feature == "ios-videotoolbox"
        and $video.header == "framework_ios_videotoolbox.h"
        and $video.symbols == ["framework_ios_videotoolbox_hardware_decode_supported"]
        and ($video.api | contains("iOS 11.0"))
        and $video.link_probe_deployment_minimums["aarch64-apple-ios"] == "11.0"
        and $video.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
        and ($video.status_mapping.success | contains("0 or 1"))
        and ($video.status_mapping.backend_unavailable | contains("UNAVAILABLE"))
        and ($video.status_mapping.non_ios | contains("UNSUPPORTED"))
        and ($video.status_mapping.panic | contains("PANIC"))
        and ($video.ownership.output | contains("valid, aligned writable byte"))
        and ($video.ownership.output | contains("full call"))
        and ($video.ownership.output | contains("avoid unsynchronized concurrent access"))
        and ($video.ownership.output | contains("No output address persists after return"))
        and ($video.direct_imports_64_bit_ios.c | sort) == [
            "VideoToolbox", "libSystem.B.dylib"
        ]
        and ($video.direct_imports_64_bit_ios.cpp | sort) == [
            "VideoToolbox", "libSystem.B.dylib"
        ]
    ' bindings/c/abi-manifest.json > /dev/null
else
    printf 'F25 ABI manifest contract pending root wiring\n'
fi

if rg -n '[[:blank:]]+$' \
    PLAN_BINDINGS_F25.md docs/bindings/ios-videotoolbox.md \
    "$source" "$header" bindings/c/check-ios-videotoolbox.sh \
    bindings/c/check-ios-videotoolbox-link.sh; then
    echo "F25 file has trailing whitespace" >&2
    exit 1
fi

printf '#include <framework_ios_videotoolbox.h>\n' | \
    clang -std=c11 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c -fsyntax-only -
printf '#include <stddef.h>\n#include <framework_ios_videotoolbox.h>\n' | \
    clang++ -std=c++17 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c++ -fsyntax-only -

printf 'F25 static checks complete; no Cargo build, link, test, consumer, or probe was run\n'

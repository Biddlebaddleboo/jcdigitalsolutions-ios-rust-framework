#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

source=bindings/c/src/ios_mps_status.rs
header=bindings/c/include/framework_ios_mps_status.h

sh -n bindings/c/check-ios-mps-status.sh
sh -n bindings/c/check-ios-mps-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

source_symbol=framework_ios_mps_status_preferred_device_available
rg -q "pub unsafe extern \"C\" fn $source_symbol" "$source"
rg -q "FrameworkStatus $source_symbol" "$header"
rg -q '#\[cfg\(not\(target_os = "ios"\)\)\]' "$source" &&
    rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
for contract_file in "$source" "$header"; do
    rg -q 'valid, properly aligned writable' "$contract_file"
    rg -q 'memory for one byte' "$contract_file"
    rg -q 'during this synchronous call' "$contract_file"
    rg -q 'unsynchronized concurrent access' "$contract_file"
    rg -q 'checks nullness only' "$contract_file"
    rg -q 'does not retain the output address' "$contract_file"
done

if jq -e '.optional_capabilities.ios_mps_status' \
    bindings/c/abi-manifest.json > /dev/null 2>&1; then
    jq -e '
        .optional_capabilities.ios_mps_status as $mps
        | $mps.cargo_feature == "ios-mps-status"
        and $mps.header == "framework_ios_mps_status.h"
        and $mps.symbols == ["framework_ios_mps_status_preferred_device_available"]
        and ($mps.api | contains("iOS 12.2"))
        and $mps.link_probe_deployment_minimums["aarch64-apple-ios"] == "12.2"
        and $mps.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
        and ($mps.status_mapping.success | contains("0 or 1"))
        and ($mps.status_mapping.non_ios | contains("UNSUPPORTED"))
        and ($mps.status_mapping.panic | contains("PANIC"))
        and ($mps.ownership.output | contains("valid, properly aligned writable memory"))
        and ($mps.ownership.output | contains("during the synchronous call"))
        and ($mps.ownership.output | contains("prevent unsynchronized concurrent access"))
        and ($mps.ownership.output | contains("checks nullness only"))
        and ($mps.ownership.output | contains("does not retain the output address"))
        and ($mps.direct_imports_64_bit_ios.c | sort) == [
            "Foundation", "Metal", "MetalPerformanceShaders", "libSystem.B.dylib", "libobjc.A.dylib"
        ]
        and ($mps.direct_imports_64_bit_ios.cpp | sort) == [
            "Foundation", "Metal", "MetalPerformanceShaders", "libSystem.B.dylib", "libc++.1.dylib", "libobjc.A.dylib"
        ]
    ' bindings/c/abi-manifest.json > /dev/null
else
    printf 'F24 ABI manifest shape pending root wiring\n'
fi

if rg -n '[[:blank:]]+$' \
    PLAN_BINDINGS_F24.md \
    docs/bindings/ios-mps-status.md \
    "$source" "$header" bindings/c/check-ios-mps-status.sh \
    bindings/c/check-ios-mps-status-link.sh; then
    echo "F24 file has trailing whitespace" >&2
    exit 1
fi

printf '#include <framework_ios_mps_status.h>\n' | \
    clang -std=c11 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c -fsyntax-only -
printf '#include <stddef.h>\n#include <framework_ios_mps_status.h>\n' | \
    clang++ -std=c++17 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c++ -fsyntax-only -

printf 'F24 static checks complete; no Cargo build, link, test, consumer, or probe was run\n'

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

source=bindings/c/src/ios_core_ml_status.rs
header=bindings/c/include/framework_ios_core_ml_status.h
guide=docs/bindings/ios-core-ml-status.md
plan=PLAN_BINDINGS_F27.md

for tool in awk cargo clang clang++ diff jq python3 rg rustfmt sh; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F27 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-core-ml-status.sh
sh -n bindings/c/check-ios-core-ml-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_core_ml_status_has_available_compute_device
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'out_available.write\(0\)' "$source"
rg -q 'valid, properly aligned writable memory for one' "$source"
rg -q 'duration of the synchronous call' "$source"
rg -q 'prevent unsynchronized' "$source"
rg -q 'concurrent access' "$source"
rg -q 'checks only nullness' "$source"
rg -q 'does not retain' "$source"
rg -q 'has_available_compute_device\(\)' "$source"
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
rg -q 'checks only' "$header"
rg -q 'valid, properly aligned writable memory for the' "$header"
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

jq -e '
    .optional_capabilities.ios_core_ml_status as $core_ml
    | $core_ml.cargo_feature == "ios-core-ml-status"
    and $core_ml.header == "framework_ios_core_ml_status.h"
    and $core_ml.symbols == ["framework_ios_core_ml_status_has_available_compute_device"]
    and ($core_ml.api | contains("iOS 17.0"))
    and $core_ml.link_probe_deployment_minimums["aarch64-apple-ios"] == "11.0"
    and $core_ml.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($core_ml.status_mapping.success | contains("0 or 1"))
    and ($core_ml.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($core_ml.status_mapping.panic | contains("PANIC"))
    and ($core_ml.ownership.output | contains("not retained"))
    and ($core_ml.ownership.output | contains("nullness is checked only"))
    and ($core_ml.ownership.output | contains("concurrently without synchronization"))
    and ($core_ml.direct_imports_64_bit_ios.c | sort) == [
        "CoreML", "Foundation", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
    and ($core_ml.direct_imports_64_bit_ios.cpp | sort) == [
        "CoreML", "Foundation", "libSystem.B.dylib", "libobjc.A.dylib"
    ]
' bindings/c/abi-manifest.json > /dev/null

if rg -n '[[:blank:]]+$' \
    "$plan" "$guide" \
    "$source" "$header" bindings/c/check-ios-core-ml-status.sh \
    bindings/c/check-ios-core-ml-status-link.sh; then
    echo "F27 file has trailing whitespace" >&2
    exit 1
fi

printf '#include <framework_ios_core_ml_status.h>\n' | \
    clang -std=c11 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c -fsyntax-only -
printf '#include <framework_ios_core_ml_status.h>\n' | \
    clang++ -std=c++17 -Wall -Wextra -Werror -pedantic \
        -I bindings/c/include -x c++ -fsyntax-only -

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-core-ml-status-default-tree.txt
if rg -q 'ios-core-ml-status v|objc2-core-ml v|CoreML.framework' \
    target/framework-c-ios-core-ml-status-default-tree.txt; then
    echo "Core ML dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-core-ml-status \
    > target/framework-c-ios-core-ml-status-ios-tree.txt
rg -q 'ios-core-ml-status v' target/framework-c-ios-core-ml-status-ios-tree.txt
rg -q 'objc2-core-ml feature "MLModel"' target/framework-c-ios-core-ml-status-ios-tree.txt
rg -q 'objc2-core-ml feature "MLModel_MLComputeDevice"' target/framework-c-ios-core-ml-status-ios-tree.txt
rg -q 'objc2-core-ml feature "MLComputeDeviceProtocol"' target/framework-c-ios-core-ml-status-ios-tree.txt
if rg -q 'objc2-core-ml feature "default"' target/framework-c-ios-core-ml-status-ios-tree.txt; then
    echo "objc2-core-ml default features leaked into F27" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-core-ml-status \
    > target/framework-c-ios-core-ml-status-host-tree.txt
if rg -q 'ios-core-ml-status v|objc2-core-ml v|CoreML.framework' \
    target/framework-c-ios-core-ml-status-host-tree.txt; then
    echo "iOS Core ML backend leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-core-ml-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-core-ml-status -- -D warnings
cargo doc --locked -p framework-c-api --no-deps
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-core-ml-status --target aarch64-apple-ios
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-core-ml-status --target aarch64-apple-ios-sim
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-core-ml-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-core-ml-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p framework-c-api --no-deps --target aarch64-apple-ios

printf 'F27 static/build gate passed; no tests, C/C++ consumers, or link probes were built or run\n'

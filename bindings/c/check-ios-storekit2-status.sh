#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

source=bindings/c/src/ios_storekit2_status.rs
header=bindings/c/include/framework_ios_storekit2_status.h

for tool in cargo clang clang++ jq python3 rg rustfmt; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F32 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-storekit2-status.sh
sh -n bindings/c/check-ios-storekit2-status-link.sh
rustfmt --check --edition 2024 "$source"
python3 -m json.tool bindings/c/abi-manifest.json > /dev/null

symbol=framework_ios_storekit2_status_can_make_payments
rg -q "pub unsafe extern \"C\" fn $symbol" "$source"
rg -q "FrameworkStatus $symbol" "$header"
rg -q 'if out_can_make_payments\.is_null\(\)' "$source"
rg -q 'out_can_make_payments\.write\(0\)' "$source"
rg -q 'FrameworkStatus::INVALID_ARGUMENT' "$source"
rg -q 'FrameworkStatus::UNSUPPORTED' "$source"
rg -q 'catch_unwind_status' "$source"
rg -q 'ios_storekit2_status::can_make_payments\(\)' "$source"
rg -q 'out_can_make_payments\.write\(u8::from\(can_make_payments\)\)' "$source"
rg -q 'valid, properly aligned writable memory' "$source" "$header"
rg -q 'duration of this synchronous call' "$source"
rg -q 'synchronous call' "$header"
rg -q 'unsynchronized' "$source" "$header"
rg -q 'does not retain the' "$source"
rg -q 'pointer is not retained' "$header"
rg -q 'weak symbol is absent' "$source" "$header"
rg -q 'not been runtime-tested below iOS 15\.0' "$header"
rg -q 'FRAMEWORK_STATUS_PANIC' "$header"
if rg -n 'SKPaymentQueue|SKPayment|AppStore::|Transaction::|StoreKitProduct|Product::|purchase\(' "$source"; then
    echo "out-of-scope StoreKit operation found in F32 source" >&2
    exit 1
fi

jq -e '
    .optional_capabilities.ios_storekit2_status as $storekit
    | $storekit.cargo_feature == "ios-storekit2-status"
    and $storekit.header == "framework_ios_storekit2_status.h"
    and $storekit.dependencies == ["ios-storekit2-status"]
    and $storekit.symbols == ["framework_ios_storekit2_status_can_make_payments"]
    and $storekit.link_probe_deployment_minimums["aarch64-apple-ios"] == "10.0"
    and $storekit.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($storekit.api | contains("iOS 15.0"))
    and ($storekit.weak_imports_64_bit_ios.c | sort) == [
        "StoreKit.framework", "_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ"
    ]
    and ($storekit.weak_imports_64_bit_ios.cpp | sort) == [
        "StoreKit.framework", "_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ"
    ]
    and ($storekit.status_mapping.success | contains("exactly 0 or 1"))
    and ($storekit.status_mapping.success | contains("absent weak symbol"))
    and ($storekit.status_mapping.non_ios | contains("UNSUPPORTED"))
    and ($storekit.status_mapping.panic | contains("PANIC"))
    and ($storekit.ownership.output | contains("valid, properly aligned writable uint8_t"))
    and ($storekit.ownership.output | contains("full synchronous call"))
    and ($storekit.ownership.output | contains("caller prevents unsynchronized access"))
    and ($storekit.ownership.output | contains("not retained"))
    and ($storekit.direct_imports_64_bit_ios.c | sort) == ["StoreKit", "libSystem.B.dylib"]
    and ($storekit.direct_imports_64_bit_ios.cpp | sort) == ["StoreKit", "libSystem.B.dylib"]
' bindings/c/abi-manifest.json > /dev/null

for file in PLAN_BINDINGS_F32.md docs/bindings/ios-storekit2-status.md "$source" "$header" \
    bindings/c/check-ios-storekit2-status.sh bindings/c/check-ios-storekit2-status-link.sh; do
    if rg -n '[[:blank:]]+$' "$file"; then
        echo "F32 file has trailing whitespace: $file" >&2
        exit 1
    fi
done

cargo tree --locked --no-default-features -p framework-c-api --target aarch64-apple-ios \
    > target/framework-c-ios-storekit2-status-default-tree.txt
if rg -q 'ios-storekit2-status v|StoreKit.framework' \
    target/framework-c-ios-storekit2-status-default-tree.txt; then
    echo "F32 backend leaked into the default iOS graph" >&2
    exit 1
fi
cargo tree --locked --no-default-features --features ios-storekit2-status \
    -p framework-c-api --target x86_64-apple-darwin \
    > target/framework-c-ios-storekit2-status-host-tree.txt
if rg -q 'ios-storekit2-status v|StoreKit.framework' \
    target/framework-c-ios-storekit2-status-host-tree.txt; then
    echo "F32 backend leaked into the host feature graph" >&2
    exit 1
fi
cargo tree --locked --no-default-features --features ios-storekit2-status \
    -p framework-c-api --target aarch64-apple-ios \
    > target/framework-c-ios-storekit2-status-ios-tree.txt
rg -q 'ios-storekit2-status v' target/framework-c-ios-storekit2-status-ios-tree.txt

cat > target/framework-c-ios-storekit2-status-c.c <<'FIXTURE_C'
#include <framework_ios_storekit2_status.h>
int main(void) {
    uint8_t can_make_payments = 0;
    return (int)framework_ios_storekit2_status_can_make_payments(&can_make_payments);
}
FIXTURE_C
cat > target/framework-c-ios-storekit2-status-cpp.cpp <<'FIXTURE_CPP'
#include <stddef.h>
#include <framework_ios_storekit2_status.h>
int main() {
    uint8_t can_make_payments = 0;
    return static_cast<int>(framework_ios_storekit2_status_can_make_payments(&can_make_payments));
}
FIXTURE_CPP

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-storekit2-status-c.c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-storekit2-status-cpp.cpp

printf 'F32 static checks passed; no tests, consumers, or probes were executed\n'

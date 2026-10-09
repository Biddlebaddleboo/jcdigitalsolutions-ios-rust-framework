#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in cargo clang clang++ jq rg; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F31 static/build gate" >&2
        exit 1
    fi
done

sh -n bindings/c/check-ios-roomplan-status.sh
cargo fmt --all -- --check
jq -e '
    .optional_capabilities.ios_roomplan_status.cargo_feature == "ios-roomplan-status" and
    .optional_capabilities.ios_roomplan_status.header == "framework_ios_roomplan_status.h" and
    .optional_capabilities.ios_roomplan_status.dependencies == ["ios-roomplan"] and
    .optional_capabilities.ios_roomplan_status.symbols == ["framework_ios_roomplan_status_is_supported"] and
    (.optional_capabilities.ios_roomplan_status.ownership.output | contains("full synchronous call")) and
    (.optional_capabilities.ios_roomplan_status.ownership.output | contains("unsynchronized access")) and
    (.optional_capabilities.ios_roomplan_status.ownership.output | contains("not retained")) and
    .optional_capabilities.ios_roomplan_status.direct_imports_64_bit_ios.c == ["RoomPlan", "libSystem.B.dylib"] and
    .optional_capabilities.ios_roomplan_status.direct_imports_64_bit_ios.cpp == ["RoomPlan", "libSystem.B.dylib"] and
    .optional_capabilities.ios_roomplan_status.link_probe_deployment_minimums["aarch64-apple-ios"] == "16.0" and
    .optional_capabilities.ios_roomplan_status.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "16.0" and
    (.optional_capabilities.ios_roomplan_status.targets.non_ios | contains("UNSUPPORTED"))
' bindings/c/abi-manifest.json >/dev/null

rg -Fq 'valid, properly aligned writable memory' \
    bindings/c/src/ios_roomplan_status.rs bindings/c/include/framework_ios_roomplan_status.h
rg -Fq 'duration of this synchronous call' bindings/c/src/ios_roomplan_status.rs
rg -Fq 'full synchronous call' bindings/c/include/framework_ios_roomplan_status.h
rg -Fq 'unsynchronized concurrent access' \
    bindings/c/src/ios_roomplan_status.rs bindings/c/include/framework_ios_roomplan_status.h
rg -Fq 'not retain the pointer' bindings/c/src/ios_roomplan_status.rs
rg -Fq 'pointer is not retained' bindings/c/include/framework_ios_roomplan_status.h
rg -Fq 'if out_supported.is_null()' bindings/c/src/ios_roomplan_status.rs
rg -Fq 'null pointer returns' bindings/c/include/framework_ios_roomplan_status.h

cargo tree --locked -p framework-c-api --no-default-features --target x86_64-apple-darwin \
    > target/framework-c-ios-roomplan-status-default-tree.txt
if rg -q 'ios-roomplan v|framework-roomplan v|RoomPlan.framework' \
    target/framework-c-ios-roomplan-status-default-tree.txt; then
    echo "F31 leaked into the default host dependency graph" >&2
    exit 1
fi
cargo tree --locked -p framework-c-api --no-default-features \
    --features ios-roomplan-status --target x86_64-apple-darwin \
    > target/framework-c-ios-roomplan-status-host-tree.txt
if rg -q 'ios-roomplan v|framework-roomplan v|RoomPlan.framework' \
    target/framework-c-ios-roomplan-status-host-tree.txt; then
    echo "iOS RoomPlan dependency leaked into the host feature graph" >&2
    exit 1
fi
cargo tree --locked -p framework-c-api --no-default-features \
    --features ios-roomplan-status --target aarch64-apple-ios \
    > target/framework-c-ios-roomplan-status-device-tree.txt
rg -q 'ios-roomplan v' target/framework-c-ios-roomplan-status-device-tree.txt
rg -q 'framework-roomplan v' target/framework-c-ios-roomplan-status-device-tree.txt

cat > target/framework-c-ios-roomplan-status-c.c <<'FIXTURE_C'
#include "framework_ios_roomplan_status.h"

void framework_roomplan_status_header_smoke(void) {
    uint8_t supported = 0;
    (void) framework_ios_roomplan_status_is_supported(&supported);
}
FIXTURE_C
cat > target/framework-c-ios-roomplan-status-cpp.cpp <<'FIXTURE_CPP'
#include "framework_ios_roomplan_status.h"

void framework_roomplan_status_header_smoke() {
    uint8_t supported = 0;
    (void) framework_ios_roomplan_status_is_supported(&supported);
}
FIXTURE_CPP

clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-roomplan-status-c.c
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    -fsyntax-only target/framework-c-ios-roomplan-status-cpp.cpp

cargo check --locked -p framework-c-api --no-default-features --features ios-roomplan-status
cargo clippy --locked -p framework-c-api --no-default-features --features ios-roomplan-status \
    -- -D warnings
cargo doc --locked --no-deps -p framework-c-api --no-default-features \
    --features ios-roomplan-status
cargo check --locked -p framework-c-api --no-default-features --features ios-roomplan-status \
    --target aarch64-apple-ios
cargo clippy --locked -p framework-c-api --no-default-features --features ios-roomplan-status \
    --target aarch64-apple-ios -- -D warnings
cargo doc --locked --no-deps -p framework-c-api --no-default-features \
    --features ios-roomplan-status --target aarch64-apple-ios
cargo check --locked -p framework-c-api --no-default-features --features ios-roomplan-status \
    --target aarch64-apple-ios-sim
cargo clippy --locked -p framework-c-api --no-default-features --features ios-roomplan-status \
    --target aarch64-apple-ios-sim -- -D warnings

printf 'F31 static/build gate passed; no tests, native consumers, link probes, or RoomPlan calls ran\n'

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target
sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for this audit: $llvm_nm" >&2
    exit 1
fi

python3 -m json.tool bindings/c/abi-manifest.json > /dev/null
sh -n bindings/c/check-ios-call-observer.sh
cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check
git diff --check

jq -e '
    .optional_capabilities.ios_call_observer as $call
    | $call.cargo_feature == "ios-call-observer"
    and $call.header == "framework_ios_call_observer.h"
    and $call.symbols == ["framework_ios_call_observer_active_call_snapshot"]
    and $call.state_flags == {
        "FRAMEWORK_IOS_CALL_OBSERVER_STATE_OUTGOING": 1,
        "FRAMEWORK_IOS_CALL_OBSERVER_STATE_CONNECTED": 2,
        "FRAMEWORK_IOS_CALL_OBSERVER_STATE_ON_HOLD": 4,
        "FRAMEWORK_IOS_CALL_OBSERVER_STATE_ENDED": 8
    }
    and $call.link_probe_deployment_minimums["aarch64-apple-ios"] == "12.0"
    and $call.link_probe_deployment_minimums["aarch64-apple-ios-sim"] == "14.0"
    and ($call.threading | contains("may block"))
    and ($call.ownership.native_objects | contains("no native handle or object crosses C"))
' bindings/c/abi-manifest.json > /dev/null

jq -r '.optional_capabilities.ios_call_observer.symbols[]' bindings/c/abi-manifest.json \
    | sort -u > target/framework-c-call-observer-expected-symbols.txt
rg -o 'framework_ios_call_observer_[A-Za-z0-9_]+' \
    bindings/c/include/framework_ios_call_observer.h | sort -u \
    > target/framework-c-call-observer-header-symbols.txt
diff -u target/framework-c-call-observer-expected-symbols.txt \
    target/framework-c-call-observer-header-symbols.txt
{
    cat <<'ASSERT_HEADER'
#if defined(__cplusplus)
#define FRAMEWORK_CALL_OBSERVER_STATIC_ASSERT static_assert
#else
#define FRAMEWORK_CALL_OBSERVER_STATIC_ASSERT _Static_assert
#endif
ASSERT_HEADER
    jq -r '
        .optional_capabilities.ios_call_observer.state_flags
        | to_entries[]
        | "#define FRAMEWORK_C_MANIFEST_" + .key + " " + (.value | tostring)
          + "\nFRAMEWORK_CALL_OBSERVER_STATIC_ASSERT(" + .key
          + " == FRAMEWORK_C_MANIFEST_" + .key
          + ", \"" + .key + " must match ABI manifest\");"
    ' bindings/c/abi-manifest.json
} > target/framework-c-call-observer-manifest-asserts.h

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-call-observer-default-tree.txt
if rg -q 'ios-call-observer|objc2-call-kit|CallKit' \
    target/framework-c-call-observer-default-tree.txt; then
    echo "CallKit dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-call-observer \
    > target/framework-c-call-observer-ios-tree.txt
rg -q 'ios-call-observer' target/framework-c-call-observer-ios-tree.txt
rg -q 'objc2-call-kit feature "CXCallObserver"' target/framework-c-call-observer-ios-tree.txt
rg -q 'objc2-call-kit feature "CXCall"' target/framework-c-call-observer-ios-tree.txt
if rg -q 'objc2-call-kit feature "(default|CXCallController|CXProvider|CXProviderConfiguration|CXCallDirectory|block2|dispatch2|objc2-avf-audio|std)"' \
    target/framework-c-call-observer-ios-tree.txt; then
    echo "out-of-scope or default CallKit features leaked into F12" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-call-observer \
    > target/framework-c-call-observer-host-tree.txt
if rg -q 'ios-call-observer|objc2-call-kit|CallKit' \
    target/framework-c-call-observer-host-tree.txt; then
    echo "CallKit dependency leaked into the non-iOS feature graph" >&2
    exit 1
fi

cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features --features ios-call-observer
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-call-observer -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-call-observer

cat > target/framework-c-call-observer-c.c <<'FIXTURE_C'
#include <framework_ios_call_observer.h>
#include "framework-c-call-observer-manifest-asserts.h"
_Static_assert(sizeof(FrameworkIosCallObserverStateFlags) == sizeof(uint32_t), "state flag width");
int main(void) {
    uint64_t count = UINT64_MAX;
    FrameworkIosCallObserverStateFlags flags = UINT32_MAX;
    FrameworkStatus status = framework_ios_call_observer_active_call_snapshot(&count, &flags);
    return (int)status;
}
FIXTURE_C
cat > target/framework-c-call-observer-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_call_observer.h>
#include "framework-c-call-observer-manifest-asserts.h"
static_assert(sizeof(FrameworkIosCallObserverStateFlags) == sizeof(uint32_t), "state flag width");
int main() {
    uint64_t count = UINT64_MAX;
    FrameworkIosCallObserverStateFlags flags = UINT32_MAX;
    FrameworkStatus status = framework_ios_call_observer_active_call_snapshot(&count, &flags);
    return static_cast<int>(status);
}
FIXTURE_CPP

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-call-observer-c.c "$host_archive" \
    -o target/framework-c-call-observer-c-host
# These fixtures use only C ABI declarations; avoid libc++ headers at the iOS 12 compile floor.
clang++ -nostdinc++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
    target/framework-c-call-observer-cpp.cpp "$host_archive" \
    -o target/framework-c-call-observer-cpp-host
"$llvm_nm" -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_call_observer_[A-Za-z0-9_]+' | sed 's/^_//' | sort -u \
    > target/framework-c-call-observer-host-symbols.txt
diff -u target/framework-c-call-observer-expected-symbols.txt \
    target/framework-c-call-observer-host-symbols.txt
"$llvm_nm" -u "$host_archive" 2>/dev/null > target/framework-c-call-observer-host-undefined.txt
if rg -qi 'CallKit|CXCall|objc_msgSend|OBJC_CLASS|objc2' \
    target/framework-c-call-observer-host-undefined.txt; then
    echo "CallKit or Objective-C import leaked into the host archive" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0; sdk=iphoneos; clang_target=arm64-apple-ios12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0; sdk=iphonesimulator; clang_target=arm64-apple-ios14.0-simulator ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-call-observer --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-call-observer \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-call-observer --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-call-observer-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-call-observer-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-call-observer-$language-$target"
        case "$language" in
            c)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
                    -Wall -Wextra -Werror -pedantic -I bindings/c/include -I target \
                    "$source" "$archive" -framework CallKit -framework Foundation -lobjc \
                    -o "$binary"
                ;;
            cpp)
                xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -nostdinc++ \
                    -std="$standard" -Wall -Wextra -Werror -pedantic \
                    -I bindings/c/include -I target "$source" "$archive" \
                    -framework CallKit -framework Foundation -lobjc -o "$binary"
                ;;
        esac
        libraries="target/framework-c-call-observer-$language-$target-libraries.txt"
        otool -L "$binary" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_call_observer.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-call-observer-$language-imports-expected.txt"
        diff -u "target/framework-c-call-observer-$language-imports-expected.txt" "$libraries"
        nm -u "$binary" 2>/dev/null \
            > "target/framework-c-call-observer-$language-$target-undefined.txt"
        rg -q '_objc_msgSend' "target/framework-c-call-observer-$language-$target-undefined.txt"
        strings "$binary" > "target/framework-c-call-observer-$language-$target-strings.txt"
        for selector in CXCallObserver calls isOutgoing hasConnected isOnHold hasEnded; do
            rg -q "$selector" "target/framework-c-call-observer-$language-$target-strings.txt"
        done
        if rg -qi 'CXCallController|CXProvider|CXCallDirectory|AVFAudio|PushKit|Swift|UIKit|UserNotifications' \
            "target/framework-c-call-observer-$language-$target-undefined.txt" \
            "target/framework-c-call-observer-$language-$target-strings.txt"; then
            echo "out-of-scope CallKit or unrelated framework symbol in $binary" >&2
            exit 1
        fi
        vtool -show-build "$binary" \
            > "target/framework-c-call-observer-$language-$target-build.txt"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' \
            "target/framework-c-call-observer-$language-$target-build.txt")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target $actual_deployment_target; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    "$llvm_nm" -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_call_observer_[A-Za-z0-9_]+' \
        | sed 's/^_//' | sort -u > "target/framework-c-call-observer-$target-archive-symbols.txt"
    diff -u target/framework-c-call-observer-expected-symbols.txt \
        "target/framework-c-call-observer-$target-archive-symbols.txt"
done

printf 'F12 checks complete; no tests or consumer binaries were executed\n'

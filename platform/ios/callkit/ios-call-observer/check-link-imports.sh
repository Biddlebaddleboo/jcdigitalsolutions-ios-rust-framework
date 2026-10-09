#!/bin/sh
set -eu

package_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(git -C "$package_dir" rev-parse --show-toplevel)
cd "$root"

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS CallKit observer link/import check" >&2
        exit 1
    fi
done

expected_imports="$package_dir/link-imports-expected.txt"
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-call-observer-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" \
        CARGO_TARGET_DIR="$root/$target_dir" \
        cargo build --locked --release -p ios-call-observer \
        --example ios_call_observer_link_probe --target "$target"

    binary="$root/$target_dir/$target/release/examples/ios_call_observer_link_probe"
    imports="$root/$target_dir/imports.txt"
    libraries="$root/$target_dir/libraries.txt"
    symbols="$root/$target_dir/undefined-symbols.txt"
    strings_file="$root/$target_dir/strings.txt"
    build_info="$root/$target_dir/build-info.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort -u > "$libraries"
    diff -u "$expected_imports" "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in 'objc_getClass' 'objc_msgSend'; do
        if ! grep -Eq "^_?$symbol$" "$symbols"; then
            echo "$target probe is missing expected Objective-C runtime symbol $symbol" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]|CXCallController|CXProvider|CXCallDirectory|AVAudioSession|PushKit|UIApplication|UIView|_OBJC_CLASS_\$_(CXCallController|CXProvider)' "$symbols"; then
        echo "unexpected Swift runtime, call-control, audio, PushKit, or UI symbol import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    for selector in CXCallObserver calls isOutgoing hasConnected isOnHold hasEnded; do
        if ! grep -Fxq "$selector" "$strings_file"; then
            echo "$target probe is missing expected CallKit class/selector string $selector" >&2
            exit 1
        fi
    done
    if grep -Eqi 'CXCallController|CXProvider|CXCallDirectory|AVAudioSession|PushKit|(^|[^[:alnum:]_])UUID([^[:alnum:]_]|$)' "$strings_file"; then
        echo "unexpected call-control, caller-identity, audio, or PushKit selector/string in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

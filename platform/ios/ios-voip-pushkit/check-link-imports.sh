#!/bin/sh
set -eu

package_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(git -C "$package_dir" rev-parse --show-toplevel)
cd "$root"

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS VoIP PushKit link/import check" >&2
        exit 1
    fi
done

expected_imports="$package_dir/link-imports-expected.txt"
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=11.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-voip-pushkit-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" \
        CARGO_TARGET_DIR="$root/$target_dir" \
        cargo build --locked --release -p ios-voip-pushkit \
        --example ios_voip_pushkit_link_probe --target "$target"

    binary="$root/$target_dir/$target/release/examples/ios_voip_pushkit_link_probe"
    imports="$root/$target_dir/imports.txt"
    libraries="$root/$target_dir/libraries.txt"
    symbols="$root/$target_dir/undefined-symbols.txt"
    strings_file="$root/$target_dir/strings.txt"
    build_info="$root/$target_dir/build-info.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort -u > "$libraries"
    diff -u "$expected_imports" "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in objc_getClass objc_msgSend; do
        if ! grep -Eq "^_?$symbol$" "$symbols"; then
            echo "$target probe is missing expected Objective-C runtime symbol $symbol" >&2
            exit 1
        fi
    done
    if ! grep -Eq '^_PKPushTypeVoIP$' "$symbols"; then
        echo "$target probe is missing the public PKPushTypeVoIP constant" >&2
        exit 1
    fi
    if grep -Eqi 'swift_|Py[A-Z_]|UIApplication|UIView|_OBJC_CLASS_\$_UIApplication' "$symbols"; then
        echo "unexpected Swift, Python, UIKit, or UIApplication import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    for symbol in PKPushRegistry NSJSONSerialization CXCallUpdate; do
        if ! grep -Fxq "$symbol" "$strings_file"; then
            echo "$target probe is missing expected PushKit/Foundation class string $symbol" >&2
            exit 1
        fi
    done
    for selector in \
        initWithQueue: setDelegate: setDesiredPushTypes: \
        pushRegistry:didUpdatePushCredentials:forType: \
        pushRegistry:didInvalidatePushTokenForType: \
        pushRegistry:didReceiveIncomingPushWithPayload:forType:withCompletionHandler: \
        dictionaryPayload token 'dataWithJSONObject:options:error:' \
        'reportNewIncomingCallWithUUID:update:completion:'; do
        if ! grep -Fxq "$selector" "$strings_file"; then
            echo "$target probe is missing expected public selector $selector" >&2
            exit 1
        fi
    done
    if grep -Eqi 'PKPushTypeComplication|PKPushTypeFileProvider|AVAudioSession|_TtC|swift_' "$strings_file"; then
        echo "unexpected non-VoIP PushKit, audio, or Swift runtime string in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

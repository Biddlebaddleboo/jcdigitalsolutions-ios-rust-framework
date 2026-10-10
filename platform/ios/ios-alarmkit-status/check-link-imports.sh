#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        min_flag=-miphoneos-version-min=26.0
    else
        sdk=iphonesimulator
        min_flag=-mios-simulator-version-min=26.0
    fi
    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    export IPHONEOS_DEPLOYMENT_TARGET=26.0
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo +1.94.1 build --locked --release --target "$target" \
        -p ios-alarmkit-status --example alarmkit_status_link_probe

    probe="target/$target/release/examples/alarmkit_status_link_probe"
    imports=$(otool -L "$probe")
    printf '%s\n' "$imports" | grep -F '/AlarmKit.framework/AlarmKit' >/dev/null || {
        echo "$target probe does not import the public AlarmKit framework" >&2
        exit 1
    }
    symbols=$(nm -u "$probe")
    symbol_details=$(nm -m "$probe")
    for symbol in \
        '_$s8AlarmKit0A7ManagerCMa' \
        '_$s8AlarmKit0A7ManagerC6sharedACvgZ' \
        '_$s8AlarmKit0A7ManagerC18authorizationStateAC013AuthorizationE0OvgTj' \
        '_$s8AlarmKit0A7ManagerC18AuthorizationStateOMa'; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target probe lacks expected public AlarmKit import $symbol" >&2
            exit 1
        }
        printf '%s\n' "$symbol_details" | grep -F "weak external $symbol (from AlarmKit)" >/dev/null || {
            echo "$target probe does not weak-import AlarmKit symbol $symbol" >&2
            exit 1
        }
    done
    build=$(xcrun vtool -show-build "$probe")
    printf '%s\n' "$build" | grep -E 'minos[[:space:]]+26\.0([.]0)?([[:space:]]|$)' >/dev/null || {
        echo "$target probe does not declare the iOS 26.0 deployment floor" >&2
        exit 1
    }
    printf '%s AlarmKit imports and iOS 26.0 minimum verified; link example not executed\n' "$target"
done

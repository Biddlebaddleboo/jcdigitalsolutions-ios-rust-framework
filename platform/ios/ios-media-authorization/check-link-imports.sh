#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"
export LC_ALL=C

for tool in awk cargo grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the media-authorization link/import check\n' "$tool" >&2
        exit 1
    fi
done

minimum_from_build() {
    printf '%s\n' "$1" | awk '$1 == "minos" || $1 == "version" { print $2; exit }'
}

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if [ "$target" = aarch64-apple-ios ]; then
        sdk=iphoneos
        minimum=10.0
    else
        sdk=iphonesimulator
        minimum=14.0
    fi

    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    unset IPHONEOS_DEPLOYMENT_TARGET RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
    cargo +1.94.1 build --locked --release --target "$target" \
        -p ios-media-authorization --example media_authorization_link_probe

    probe="target/$target/release/examples/media_authorization_link_probe"
    imports=$(otool -L "$probe")
    actual_libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | sort)
    expected_libraries=$(printf '%s\n' AVFoundation Foundation libSystem.B.dylib libobjc.A.dylib | sort)
    if [ "$actual_libraries" != "$expected_libraries" ]; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$actual_libraries" >&2
        exit 1
    fi

    symbols=$(nm -u "$probe")
    printf '%s\n' "$symbols" | grep -F '_objc_msgSend' >/dev/null || {
        printf '%s probe does not import objc_msgSend\n' "$target" >&2
        exit 1
    }
    binary_strings=$(strings "$probe")
    for selector in authorizationStatusForMediaType: requestAccessForMediaType:completionHandler:; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            printf '%s probe does not reference %s\n' "$target" "$selector" >&2
            exit 1
        }
    done
    if printf '%s\n%s\n' "$imports" "$symbols" | grep -E 'libswift|SwiftUI|_swift_|_\$s[0-9]' >/dev/null; then
        printf '%s probe unexpectedly imports Swift runtime symbols\n' "$target" >&2
        exit 1
    fi

    build=$(xcrun vtool -show-build "$probe")
    actual_minimum=$(minimum_from_build "$build")
    if [ "$actual_minimum" != "$minimum" ]; then
        printf '%s probe declares deployment minimum %s; expected Rust target floor %s\n' \
            "$target" "$actual_minimum" "$minimum" >&2
        exit 1
    fi
    printf '%s AVFoundation/Foundation/Objective-C imports and authorization selectors verified at Rust deployment minimum iOS %s; link probe not executed\n' \
        "$target" "$minimum"
done

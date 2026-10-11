#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

swift_sources=$(find platform/ios/ios-microphone-recorder -type f -name '*.swift' -print)
if [ -n "$swift_sources" ]; then
    printf 'Swift source is forbidden in the iOS microphone-recorder package:\n%s\n' "$swift_sources" >&2
    exit 1
fi

for tool in awk cargo find grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS microphone-recorder link/import check\n' "$tool" >&2
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
        min_flag=-miphoneos-version-min=10.0
    else
        sdk=iphonesimulator
        minimum=14.0
        min_flag=-mios-simulator-version-min=14.0
    fi

    export SDKROOT
    SDKROOT=$(xcrun --sdk "$sdk" --show-sdk-path)
    avfoundation_tbd="$SDKROOT/System/Library/Frameworks/AVFoundation.framework/AVFoundation.tbd"
    avfaudio_tbd="$SDKROOT/System/Library/Frameworks/AVFAudio.framework/AVFAudio.tbd"
    if ! grep -F "'/System/Library/Frameworks/AVFAudio.framework/AVFAudio'" "$avfoundation_tbd" >/dev/null; then
        printf '%s SDK AVFoundation metadata does not depend on AVFAudio\n' "$target" >&2
        exit 1
    fi
    for class in AVAudioFormat AVAudioRecorder; do
        if ! grep -F "$class" "$avfaudio_tbd" >/dev/null; then
            printf '%s SDK AVFAudio metadata does not export %s\n' "$target" "$class" >&2
            exit 1
        fi
    done
    unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
    export IPHONEOS_DEPLOYMENT_TARGET=$minimum
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo +1.94.1 build --locked --release --target "$target" \
        -p ios-microphone-recorder --example microphone_recorder_link_probe

    probe="target/$target/release/examples/microphone_recorder_link_probe"
    imports=$(otool -L "$probe")
    actual_libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | LC_ALL=C sort)
    expected_libraries=$(printf '%s\n' AVFoundation Foundation libSystem.B.dylib libobjc.A.dylib | LC_ALL=C sort)
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
    for symbol in \
        AVAudioRecorder \
        AVAudioFormat \
        initWithCommonFormat:sampleRate:channels:interleaved: \
        initWithURL:format:error: \
        authorizationStatusForMediaType: \
        record \
        stop \
        isRecording
    do
        printf '%s\n' "$binary_strings" | grep -F "$symbol" >/dev/null || {
            printf '%s probe does not retain the %s API reference\n' "$target" "$symbol" >&2
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
        printf '%s probe declares deployment minimum %s; expected %s\n' \
            "$target" "$actual_minimum" "$minimum" >&2
        exit 1
    fi
    printf '%s AVFoundation/Foundation/Objective-C imports, SDK AVFAudio dependency/API exports, and iOS %s deployment minimum verified; link probe not executed\n' \
        "$target" "$minimum"
done

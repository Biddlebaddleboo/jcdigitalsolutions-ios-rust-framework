#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

for tool in awk cargo grep nm otool sort strings xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS audio-session link/import check\n' "$tool" >&2
        exit 1
    fi
done

sdk_root=$(xcrun --sdk iphoneos --show-sdk-path)
header="$sdk_root/System/Library/Frameworks/AVFAudio.framework/Headers/AVAudioSession.h"
types_header="$sdk_root/System/Library/Frameworks/AVFAudio.framework/Headers/AVAudioSessionTypes.h"
grep -F 'setCategory:(AVAudioSessionCategory)category' "$header" >/dev/null
grep -F 'mode:(AVAudioSessionMode)mode' "$header" >/dev/null
grep -F 'options:(AVAudioSessionCategoryOptions)options' "$header" >/dev/null
grep -A 4 -F 'setCategory:(AVAudioSessionCategory)category' "$header" | grep -F 'ios(10.0)' >/dev/null
grep -F 'setActive:(BOOL)active withOptions:(AVAudioSessionSetActiveOptions)options' "$header" >/dev/null
for definition in \
    AVAudioSessionCategoryOptionMixWithOthers=0x1 \
    AVAudioSessionCategoryOptionDuckOthers=0x2 \
    AVAudioSessionCategoryOptionAllowBluetoothHFP=0x4 \
    AVAudioSessionCategoryOptionDefaultToSpeaker=0x8 \
    AVAudioSessionCategoryOptionInterruptSpokenAudioAndMixWithOthers=0x11 \
    AVAudioSessionCategoryOptionAllowBluetoothA2DP=0x20 \
    AVAudioSessionCategoryOptionAllowAirPlay=0x40 \
    AVAudioSessionSetActiveOptionNotifyOthersOnDeactivation=1
do
    symbol=${definition%%=*}
    value=${definition#*=}
    grep -E "$symbol.*=[[:space:]]*$value([,[:space:]]|$)" "$types_header" >/dev/null || {
        printf 'SDK header does not define public AVAudioSession option %s as %s\n' \
            "$symbol" "$value" >&2
        exit 1
    }
done

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
    export IPHONEOS_DEPLOYMENT_TARGET=$minimum
    export RUSTFLAGS="-C link-arg=$min_flag"
    cargo +1.94.1 build --locked --offline --release --target "$target" \
        -p ios-audio-session --example ios_audio_session_link_probe
    probe="target/$target/release/examples/ios_audio_session_link_probe"
    imports=$(otool -L "$probe")
    libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | LC_ALL=C sort)
    expected=$(printf '%s\n' AVFoundation Foundation libSystem.B.dylib libobjc.A.dylib | LC_ALL=C sort)
    if [ "$libraries" != "$expected" ]; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$libraries" >&2
        exit 1
    fi
    symbols=$(nm -u "$probe")
    printf '%s\n' "$symbols" | grep -F '_objc_msgSend' >/dev/null || {
        printf '%s probe does not import objc_msgSend\n' "$target" >&2
        exit 1
    }
    binary_strings=$(strings -n 3 "$probe")
    for selector in \
        'setCategory:mode:options:error:' \
        'setActive:withOptions:error:' \
        'AVAudioSessionCategoryPlayAndRecord' \
        'AVAudioSessionModeVoiceChat'
    do
        printf '%s\n%s\n' "$symbols" "$binary_strings" | grep -F "$selector" >/dev/null || {
            printf '%s probe does not retain the %s API reference\n' "$target" "$selector" >&2
            exit 1
        }
    done
    if printf '%s\n%s\n%s\n' "$imports" "$symbols" "$binary_strings" | \
        grep -E 'libswift|SwiftUI|_swift_|_\$s[0-9]|requestRecordPermission|AVAudioApplication|AVAudioEngine|AVAudioRecorder|AVCapture|setActive:withFlags:' >/dev/null; then
        printf '%s probe imports a forbidden runtime, permission, stream, or deprecated API\n' "$target" >&2
        exit 1
    fi
    build=$(xcrun vtool -show-build "$probe")
    actual_minimum=$(printf '%s\n' "$build" | awk '$1 == "minos" || $1 == "version" { print $2; exit }')
    if [ "$actual_minimum" != "$minimum" ]; then
        printf '%s probe declares deployment minimum %s; expected %s\n' \
            "$target" "${actual_minimum:-unknown}" "$minimum" >&2
        exit 1
    fi
    printf '%s AVAudioSession API/imports and iOS %s deployment minimum verified; link probe not executed\n' \
        "$target" "$minimum"
done

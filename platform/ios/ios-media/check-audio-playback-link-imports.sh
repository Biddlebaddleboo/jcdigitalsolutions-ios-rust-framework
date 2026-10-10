#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the AVAudioSession link/import check" >&2
        exit 1
    fi
done

cat > target/ios-media-audio-link-imports-expected.txt <<'IMPORTS'
AVFoundation
CoreFoundation
CoreMedia
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=12.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            ;;
    esac

    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="target/ios-media-audio-link-$target" cargo build --locked --release -p ios-media --example ios_audio_playback_link_probe --target "$target"
    binary="target/ios-media-audio-link-$target/$target/release/examples/ios_audio_playback_link_probe"
    imports="target/ios-media-audio-link-imports-$target.txt"
    libraries="target/ios-media-audio-link-libraries-$target.txt"
    symbols="target/ios-media-audio-link-symbols-$target.txt"
    strings_file="target/ios-media-audio-strings-$target.txt"
    build_info="target/ios-media-audio-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-media-audio-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|AVAudioEngine|AVAudioPlayer|AVAudioRecorder|requestRecordPermission|setActive|setCategory|setMode|AVCapture' "$symbols"; then
        echo "unexpected runtime, audio-operation, permission, or capture symbol import in $symbols" >&2
        exit 1
    fi

    strings "$binary" > "$strings_file"
    grep -q 'sharedInstance' "$strings_file"
    grep -q 'isOtherAudioPlaying' "$strings_file"
    if grep -Eqi 'requestRecordPermission|AVAudioEngine|AVAudioPlayer|AVAudioRecorder|AVCaptureDevice' "$strings_file"; then
        echo "unexpected permission, capture, or audio-operation selector in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

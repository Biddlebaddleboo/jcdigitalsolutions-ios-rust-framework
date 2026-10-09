#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang clang++ diff jq nm otool rg sed sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the F26 device/Simulator link-import gate" >&2
        exit 1
    fi
done

sh bindings/c/check-ios-camera-device-status.sh

cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features > target/framework-c-ios-camera-device-status-default-tree.txt
if rg -q '(^|[[:space:]])ios-camera-device-status v|objc2-av-foundation v|AVFoundation.framework' \
    target/framework-c-ios-camera-device-status-default-tree.txt; then
    echo "camera-device-status dependencies leaked into the default C ABI graph" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target aarch64-apple-ios \
    --no-default-features --features ios-camera-device-status \
    > target/framework-c-ios-camera-device-status-ios-tree.txt
rg -q 'ios-camera-device-status v' target/framework-c-ios-camera-device-status-ios-tree.txt
rg -q 'objc2-av-foundation feature "AVCaptureDevice"' \
    target/framework-c-ios-camera-device-status-ios-tree.txt
rg -q 'objc2-av-foundation feature "AVMediaFormat"' \
    target/framework-c-ios-camera-device-status-ios-tree.txt
if rg -q 'objc2-av-foundation feature "default"' \
    target/framework-c-ios-camera-device-status-ios-tree.txt; then
    echo "objc2-av-foundation default features leaked into F26" >&2
    exit 1
fi
cargo tree --locked -e features -p framework-c-api --target x86_64-apple-darwin \
    --no-default-features --features ios-camera-device-status \
    > target/framework-c-ios-camera-device-status-host-tree.txt
if rg -q '(^|[[:space:]])ios-camera-device-status v|objc2-av-foundation v|AVFoundation.framework' \
    target/framework-c-ios-camera-device-status-host-tree.txt; then
    echo "iOS camera backend leaked into the host feature graph" >&2
    exit 1
fi

cargo fmt --package framework-c-api -- --check
cargo check --locked -p framework-c-api --no-default-features
cargo check --locked -p framework-c-api --no-default-features \
    --features ios-camera-device-status
cargo clippy --locked -p framework-c-api --no-default-features \
    --features ios-camera-device-status -- -D warnings
cargo build --locked --release -p framework-c-api --no-default-features \
    --features ios-camera-device-status

cat > target/framework-c-ios-camera-device-status-c.c <<'FIXTURE_C'
#include <framework_ios_camera_device_status.h>
int main(void) {
    uint8_t present = 0;
    return (int)framework_ios_camera_device_status_has_default_video_capture_device(&present);
}
FIXTURE_C
cat > target/framework-c-ios-camera-device-status-cpp.cpp <<'FIXTURE_CPP'
#include <framework_ios_camera_device_status.h>
int main() {
    uint8_t present = 0;
    return static_cast<int>(framework_ios_camera_device_status_has_default_video_capture_device(&present));
}
FIXTURE_CPP

expected_symbols=target/framework-c-ios-camera-device-status-expected-symbols.txt
jq -r '.optional_capabilities.ios_camera_device_status.symbols[]' \
    bindings/c/abi-manifest.json | LC_ALL=C sort -u > "$expected_symbols"

host_archive=target/release/libframework_c_api.a
clang -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-camera-device-status-c.c "$host_archive" \
    -o target/framework-c-ios-camera-device-status-c-host
clang++ -std=c++17 -Wall -Wextra -Werror -pedantic -I bindings/c/include \
    target/framework-c-ios-camera-device-status-cpp.cpp "$host_archive" \
    -o target/framework-c-ios-camera-device-status-cpp-host
nm -g "$host_archive" 2>/dev/null \
    | rg -o '_framework_ios_camera_device_status_[A-Za-z0-9_]+' \
    | sed 's/^_//' | LC_ALL=C sort -u \
    > target/framework-c-ios-camera-device-status-host-symbols.txt
diff -u "$expected_symbols" target/framework-c-ios-camera-device-status-host-symbols.txt
nm -u "$host_archive" 2>/dev/null \
    > target/framework-c-ios-camera-device-status-host-undefined.txt
if rg -q 'AVCaptureDevice|AVMediaTypeVideo|AVFoundation|objc_|OBJC_CLASS|swift_' \
    target/framework-c-ios-camera-device-status-host-undefined.txt; then
    echo "AVFoundation, Objective-C, or Swift import leaked into the host archive" >&2
    exit 1
fi
for language in c cpp; do
    case "$language" in
        c) binary=target/framework-c-ios-camera-device-status-c-host; expected='libSystem.B.dylib' ;;
        cpp) binary=target/framework-c-ios-camera-device-status-cpp-host; expected=$(printf '%s\n' 'libc++.1.dylib' 'libSystem.B.dylib') ;;
    esac
    otool -L "$binary" \
        | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
        | LC_ALL=C sort > "target/framework-c-ios-camera-device-status-$language-host-libraries.txt"
    printf '%s\n' "$expected" | LC_ALL=C sort \
        > "target/framework-c-ios-camera-device-status-$language-host-libraries-expected.txt"
    diff -u "target/framework-c-ios-camera-device-status-$language-host-libraries-expected.txt" \
        "target/framework-c-ios-camera-device-status-$language-host-libraries.txt"
    nm -u "$binary" 2>/dev/null \
        > "target/framework-c-ios-camera-device-status-$language-host-undefined.txt"
    if rg -q 'AVCaptureDevice|AVMediaTypeVideo|AVFoundation|objc_|OBJC_CLASS|swift_' \
        "target/framework-c-ios-camera-device-status-$language-host-undefined.txt"; then
        echo "Apple framework, Objective-C, or Swift import leaked into host $language consumer" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            deployment_target=10.0
            sdk=iphoneos
            clang_target=arm64-apple-ios10.0
            rust_min_flag=-miphoneos-version-min=10.0
            ;;
        aarch64-apple-ios-sim)
            deployment_target=14.0
            sdk=iphonesimulator
            clang_target=arm64-apple-ios14.0-simulator
            rust_min_flag=-mios-simulator-version-min=14.0
            ;;
    esac
    export IPHONEOS_DEPLOYMENT_TARGET="$deployment_target"
    export RUSTFLAGS="-C link-arg=$rust_min_flag"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo check --locked \
        -p framework-c-api --no-default-features --features ios-camera-device-status \
        --target "$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo clippy --locked \
        -p framework-c-api --no-default-features --features ios-camera-device-status \
        --target "$target" -- -D warnings
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo build --locked --release \
        -p framework-c-api --no-default-features --features ios-camera-device-status \
        --target "$target"
    archive="target/$target/release/libframework_c_api.a"
    for language in c cpp; do
        case "$language" in
            c) compiler=clang; source=target/framework-c-ios-camera-device-status-c.c; standard=c11 ;;
            cpp) compiler=clang++; source=target/framework-c-ios-camera-device-status-cpp.cpp; standard=c++17 ;;
        esac
        binary="target/framework-c-ios-camera-device-status-$language-$target"
        xcrun --sdk "$sdk" "$compiler" -target "$clang_target" -std="$standard" \
            -Wall -Wextra -Werror -pedantic -I bindings/c/include "$source" "$archive" \
            -framework AVFoundation -framework Foundation -lobjc -Wl,-dead_strip_dylibs \
            -o "$binary"
        libraries="target/framework-c-ios-camera-device-status-$language-$target-libraries.txt"
        otool -L "$binary" \
            | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
            | LC_ALL=C sort > "$libraries"
        jq -r --arg language "$language" \
            '.optional_capabilities.ios_camera_device_status.direct_imports_64_bit_ios[$language][]' \
            bindings/c/abi-manifest.json | LC_ALL=C sort \
            > "target/framework-c-ios-camera-device-status-$language-imports-expected.txt"
        diff -u "target/framework-c-ios-camera-device-status-$language-imports-expected.txt" \
            "$libraries"
        symbols="target/framework-c-ios-camera-device-status-$language-$target-undefined.txt"
        nm -u "$binary" 2>/dev/null > "$symbols"
        rg -q '_objc_getClass' "$symbols"
        rg -q '_objc_msgSend' "$symbols"
        rg -q 'AVMediaTypeVideo' "$symbols"
        strings_file="target/framework-c-ios-camera-device-status-$language-$target-strings.txt"
        strings "$binary" > "$strings_file"
        rg -q 'AVCaptureDevice' "$strings_file"
        rg -q 'defaultDeviceWithMediaType:' "$strings_file"
        if rg -qi 'requestAccessForMediaType|authorizationStatusForMediaType|AVCaptureDeviceInput|AVCaptureSession|AVCapturePhotoOutput|AVCaptureVideoDataOutput|swift_' "$symbols" "$strings_file"; then
            echo "capture, authorization, or Swift behavior leaked into $binary" >&2
            exit 1
        fi
        build_info="target/framework-c-ios-camera-device-status-$language-$target-build.txt"
        xcrun vtool -show-build "$binary" > "$build_info"
        actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
        if [ "$actual_deployment_target" != "$deployment_target" ]; then
            echo "$binary has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
            exit 1
        fi
        printf '%s %s AVFoundation imports and iOS %s minimum verified; probe not executed\n' \
            "$target" "$language" "$deployment_target"
    done
    nm -g "$archive" 2>/dev/null \
        | rg -o '_framework_ios_camera_device_status_[A-Za-z0-9_]+' \
        | sed 's/^_//' | LC_ALL=C sort -u \
        > "target/framework-c-ios-camera-device-status-$target-archive-symbols.txt"
    diff -u "$expected_symbols" \
        "target/framework-c-ios-camera-device-status-$target-archive-symbols.txt"
done

printf 'F26 device/Simulator link-import gate complete; no tests or consumer/probe binaries were executed\n'

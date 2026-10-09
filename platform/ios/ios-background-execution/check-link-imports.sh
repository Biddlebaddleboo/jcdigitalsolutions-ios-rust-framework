#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS background-execution link/import check" >&2
        exit 1
    fi
done

cat > target/ios-background-execution-link-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

cat > target/ios-background-execution-selectors-expected.txt <<'SELECTORS'
beginBackgroundTaskWithExpirationHandler:
endBackgroundTask:
sharedApplication
SELECTORS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo +1.94.1 build --locked --release -p ios-background-execution --example ios_background_execution_link_probe --target "$target"
    binary="target/$target/release/examples/ios_background_execution_link_probe"
    imports="target/ios-background-execution-link-imports-$target.txt"
    libraries="target/ios-background-execution-link-libraries-$target.txt"
    symbols="target/ios-background-execution-link-symbols-$target.txt"
    strings_file="target/ios-background-execution-link-strings-$target.txt"
    build_info="target/ios-background-execution-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-background-execution-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|BGTaskScheduler|BGAppRefreshTask|BGProcessingTask|NSURLSession|UNUserNotificationCenter|nw_(browse|connection|listener|framer|protocol|resolver|service|path_copy|path_enumerate|path_has|path_is|path_uses)' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated framework symbol import in $symbols" >&2
        exit 1
    fi
    if ! grep -Fq '_UIBackgroundTaskInvalid' "$symbols"; then
        echo "missing UIBackgroundTaskInvalid import in $symbols" >&2
        exit 1
    fi

    strings -a "$binary" > "$strings_file"
    while IFS= read -r selector; do
        if ! grep -Fq "$selector" "$strings_file"; then
            echo "missing selected UIKit selector $selector in $strings_file" >&2
            exit 1
        fi
    done < target/ios-background-execution-selectors-expected.txt

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit } $1 == "cmd" && $2 ~ /^LC_VERSION_MIN_/ { legacy = 1; next } legacy && $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-background-execution -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-background-execution" >&2
    exit 1
fi

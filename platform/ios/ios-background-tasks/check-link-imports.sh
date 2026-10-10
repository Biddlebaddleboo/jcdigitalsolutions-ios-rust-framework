#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS background-tasks link/import check" >&2
        exit 1
    fi
done

cat > target/ios-background-tasks-link-imports-expected.txt <<'IMPORTS'
BackgroundTasks
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

cat > target/ios-background-tasks-selectors-expected.txt <<'SELECTORS'
BGAppRefreshTaskRequest
BGTaskScheduler
cancelTaskRequestWithIdentifier:
registerForTaskWithIdentifier:usingQueue:launchHandler:
setExpirationHandler:
setTaskCompletedWithSuccess:
submitTaskRequest:error:
SELECTORS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=13.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" cargo +1.94.1 build --locked --release -p ios-background-tasks --example ios_background_tasks_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_background_tasks_link_import_probe"
    imports="target/ios-background-tasks-link-imports-$target.txt"
    libraries="target/ios-background-tasks-link-libraries-$target.txt"
    symbols="target/ios-background-tasks-link-symbols-$target.txt"
    strings_file="target/ios-background-tasks-link-strings-$target.txt"
    build_info="target/ios-background-tasks-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-background-tasks-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|UIApplication|UIView|NSURLSession|UNUserNotificationCenter|nw_(browse|connection|listener|framer|protocol|resolver|service|path_copy|path_enumerate|path_has|path_is|path_uses)' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated framework symbol import in $symbols" >&2
        exit 1
    fi
    if ! grep -Fq '_BGTaskSchedulerErrorDomain' "$symbols"; then
        echo "missing BGTaskSchedulerErrorDomain import in $symbols" >&2
        exit 1
    fi

    strings -a "$binary" > "$strings_file"
    while IFS= read -r selector; do
        if ! grep -Fq "$selector" "$strings_file"; then
            echo "missing selected BackgroundTasks symbol or selector $selector in $strings_file" >&2
            exit 1
        fi
    done < target/ios-background-tasks-selectors-expected.txt

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-background-tasks -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-background-tasks" >&2
    exit 1
fi

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS app-data link/import check" >&2
        exit 1
    fi
done

cat > target/ios-app-data-ios-preferences-imports-expected.txt <<'IMPORTS'
CoreFoundation
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-app-data-link-ios-preferences-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-preferences --example ios_preferences_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_preferences_link_import_probe"
    imports="target/ios-app-data-ios-preferences-imports-$target.txt"
    libraries="target/ios-app-data-ios-preferences-libraries-$target.txt"
    symbols="target/ios-app-data-ios-preferences-symbols-$target.txt"
    build_info="target/ios-app-data-ios-preferences-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-app-data-ios-preferences-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|_OBJC_(CLASS|METACLASS)_\$_(UIApplication|UIView|UNUserNotificationCenter|CLLocationManager)|_OBJC_CLASS_\$_NSURLSession|nw_(browse|connection|listener|framer|protocol|resolver|service|path_copy|path_enumerate|path_has|path_is|path_uses)|_Sec(Item|AccessControl|Key|Trust|Certificate)' "$symbols"; then
        echo "unexpected Swift/Python runtime or unrelated capability symbol import in $symbols" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "ios-preferences $target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-preferences -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under the iOS preferences backend" >&2
    exit 1
fi

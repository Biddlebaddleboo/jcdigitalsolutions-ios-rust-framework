#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff find grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS connectivity link/import check" >&2
        exit 1
    fi
done

cat > target/ios-connectivity-link-imports-expected.txt <<'IMPORTS'
Network
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-connectivity-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-connectivity --example ios_connectivity_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_connectivity_link_import_probe"
    imports="target/ios-connectivity-link-imports-$target.txt"
    libraries="target/ios-connectivity-link-libraries-$target.txt"
    symbols="target/ios-connectivity-link-symbols-$target.txt"
    build_info="target/ios-connectivity-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-connectivity-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|Py[A-Z_]|_OBJC_(CLASS|METACLASS)_\$_|\$s[0-9]|nw_(browse|connection|listener|framer|protocol|resolver|service|path_copy|path_enumerate|path_has|path_is|path_uses)' "$symbols"; then
        echo "unexpected Swift/Python runtime, Objective-C, or unrelated Network capability symbol import in $symbols" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-connectivity -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-connectivity" >&2
    exit 1
fi

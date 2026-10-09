#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

if [ "$#" -ne 1 ]; then
    echo "usage: $0 device|simulator" >&2
    exit 2
fi

case "$1" in
    device)
        target=aarch64-apple-ios
        deployment_target=12.0
        ;;
    simulator)
        target=aarch64-apple-ios-sim
        deployment_target=14.0
        ;;
    *)
        echo "unknown target kind: $1" >&2
        exit 2
        ;;
esac

target_dir="target/ios-extension-support-link-$target-minos-$deployment_target"
IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" \
CARGO_TARGET_DIR="$target_dir" \
    cargo build --locked --release -p ios-extension-support --example link_check --target "$target"

binary="$target_dir/$target/release/examples/link_check"
otool -L "$binary" > "target/ios-extension-support-link-imports-$target.txt"
awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' \
    "target/ios-extension-support-link-imports-$target.txt" | LC_ALL=C sort \
    > "target/ios-extension-support-link-libraries-$target.txt"
cat > target/ios-extension-support-link-libraries-expected.txt <<'IMPORTS'
Foundation
libSystem.B.dylib
libobjc.A.dylib
IMPORTS
diff -u target/ios-extension-support-link-libraries-expected.txt \
    "target/ios-extension-support-link-libraries-$target.txt"

nm -u "$binary" > "target/ios-extension-support-link-symbols-$target.txt"
if ! grep -Fq '_objc_msgSend' "target/ios-extension-support-link-symbols-$target.txt"; then
    echo "missing Objective-C message-send import in target probe" >&2
    exit 1
fi
for selector in 'bundleWithURL:' 'infoDictionary' 'objectForKey:' 'fileURLWithPath:'; do
    if ! strings "$binary" | grep -Fq "$selector"; then
        echo "missing Foundation selector $selector in target probe" >&2
        exit 1
    fi
done
if grep -Eiq 'swift|framework_[A-Za-z0-9_]+|ExtensionKit|PlugInKit' \
    "target/ios-extension-support-link-symbols-$target.txt"; then
    echo "unexpected Swift, extension, or framework symbol import in target probe" >&2
    exit 1
fi

build_info="target/ios-extension-support-link-build-$target.txt"
vtool -show-build "$binary" > "$build_info"
actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
if [ "$actual_deployment_target" != "$deployment_target" ]; then
    echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
    exit 1
fi

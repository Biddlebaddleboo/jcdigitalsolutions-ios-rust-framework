#!/bin/sh
set -eu

case "${1:-}" in
    device) target=aarch64-apple-ios ;;
    simulator) target=aarch64-apple-ios-sim ;;
    *) printf '%s\n' "usage: $0 device|simulator" >&2; exit 2 ;;
esac

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH= cd -- "$script_dir/../../.." && pwd)
cd "$root"

cargo build --locked -p ios-cloud --example link_check --target "$target"
binary="$root/target/$target/debug/examples/link_check"
imports=$(otool -L "$binary")
printf '%s\n' "$imports"

printf '%s\n' "$imports" | grep -q '/CloudKit.framework/CloudKit' || {
    printf '%s\n' 'CloudKit.framework import is missing' >&2
    exit 1
}
printf '%s\n' "$imports" | grep -q '/Foundation.framework/Foundation' || {
    printf '%s\n' 'Foundation.framework import is missing' >&2
    exit 1
}
if printf '%s\n' "$imports" | grep -Eiq 'libswift|/usr/lib/swift|/(UIKit|Network|Security|CoreLocation|UserNotifications)\.framework/'; then
    printf '%s\n' 'unexpected Swift runtime or unrelated capability framework import' >&2
    exit 1
fi

printf '%s\n' "ios-cloud $target import gate passed; binary was not executed"

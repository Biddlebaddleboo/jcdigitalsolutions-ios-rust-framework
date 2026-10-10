#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")/../../.." rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the ActivityKit link check" >&2
        exit 1
    fi
done

cat > target/ios-activitykit-status-link-imports-expected.txt <<'IMPORTS'
ActivityKit
libSystem.B.dylib
libswiftCore.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo +1.94.1 build --locked --release -p ios-activitykit-status \
        --example activitykit_status_link_probe --target "$target"
    binary="target/$target/release/examples/activitykit_status_link_probe"
    imports="target/ios-activitykit-status-link-imports-$target.txt"
    libraries="target/ios-activitykit-status-link-libraries-$target.txt"
    symbols="target/ios-activitykit-status-link-symbols-$target.txt"
    symbol_details="target/ios-activitykit-status-link-symbol-details-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-activitykit-status-link-imports-expected.txt "$libraries"
    if ! grep -Fq '/System/Library/Frameworks/ActivityKit.framework/ActivityKit' "$imports" || ! grep -Fq ', weak)' "$imports"; then
        echo "ActivityKit is not a weak framework import in $binary" >&2
        exit 1
    fi

    nm -u "$binary" > "$symbols"
    nm -m "$binary" > "$symbol_details"
    grep -Fq '0A17AuthorizationInfoCMa' "$symbols"
    grep -Fq '0A17AuthorizationInfoCACycfC' "$symbols"
    grep -Fq '0A17AuthorizationInfoC20areActivitiesEnabledSbvg' "$symbols"
    grep -Fq '_swift_release' "$symbols"
    for symbol in \
        '_$s11ActivityKit0A17AuthorizationInfoCMa' \
        '_$s11ActivityKit0A17AuthorizationInfoCACycfC' \
        '_$s11ActivityKit0A17AuthorizationInfoC20areActivitiesEnabledSbvg'; do
        grep -Fq "weak external $symbol (from ActivityKit)" "$symbol_details"
    done
    grep -Fq 'external _swift_release (from libswiftCore)' "$symbol_details"
    printf '%s ActivityKit link imports and bridge symbols pass audit\n' "$target"
done

echo 'linked examples were inspected only and were not executed'

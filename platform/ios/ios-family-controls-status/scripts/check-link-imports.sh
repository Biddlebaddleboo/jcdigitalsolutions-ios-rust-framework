#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")/.." rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the Family Controls link check" >&2
        exit 1
    fi
done

cat > target/ios-family-controls-status-link-imports-expected.txt <<'IMPORTS'
FamilyControls
libSystem.B.dylib
libswiftCore.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-family-controls-status \
        --example family_controls_status_link_probe --target "$target"
    binary="target/$target/release/examples/family_controls_status_link_probe"
    imports="target/ios-family-controls-status-link-imports-$target.txt"
    libraries="target/ios-family-controls-status-link-libraries-$target.txt"
    symbols="target/ios-family-controls-status-link-symbols-$target.txt"
    symbol_details="target/ios-family-controls-status-link-symbol-details-$target.txt"
    load_commands="target/ios-family-controls-status-link-load-commands-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-family-controls-status-link-imports-expected.txt "$libraries"

    otool -l "$binary" > "$load_commands"
    if ! awk '/cmd LC_LOAD_WEAK_DYLIB/ { weak = 1; next } /cmd / { weak = 0 } weak && /name .*FamilyControls\.framework\/FamilyControls/ { found = 1 } END { exit !found }' "$load_commands"; then
        echo "FamilyControls is not a weak framework import in $binary" >&2
        exit 1
    fi

    nm -u "$binary" > "$symbols"
    nm -m "$binary" > "$symbol_details"
    grep -Fq 'AuthorizationStatusOMa' "$symbols"
    grep -Fq 'AuthorizationCenterCMa' "$symbols"
    grep -Fq 'AuthorizationCenterC6sharedACvgZ' "$symbols"
    grep -Fq 'AuthorizationCenterC19authorizationStatusAA0cF0OvgTj' "$symbols"
    grep -Fq 'AuthorizationStatusO8rawValueSivg' "$symbols"
    grep -Fq '_swift_release' "$symbols"
    for symbol in \
        '_$s14FamilyControls19AuthorizationStatusOMa' \
        '_$s14FamilyControls19AuthorizationCenterCMa' \
        '_$s14FamilyControls19AuthorizationCenterC6sharedACvgZ' \
        '_$s14FamilyControls19AuthorizationCenterC19authorizationStatusAA0cF0OvgTj' \
        '_$s14FamilyControls19AuthorizationStatusO8rawValueSivg'; do
        grep -Fq "weak external $symbol (from FamilyControls)" "$symbol_details"
    done
    grep -Fq 'external _swift_release (from libswiftCore)' "$symbol_details"
    printf '%s Family Controls imports and bridge symbols pass link audit\n' "$target"
done

echo 'linked probes were inspected only and were not executed'

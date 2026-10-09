#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the SharedWithYou link check" >&2
        exit 1
    fi
done

cat > target/ios-shared-with-you-support-link-imports-expected.txt <<'IMPORTS'
Foundation
SharedWithYou
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-shared-with-you-support --example ios_shared_with_you_support_link_probe --target "$target"
    binary="target/$target/release/examples/ios_shared_with_you_support_link_probe"
    imports="target/ios-shared-with-you-support-link-imports-$target.txt"
    libraries="target/ios-shared-with-you-support-link-libraries-$target.txt"
    symbols="target/ios-shared-with-you-support-link-symbols-$target.txt"
    strings_file="target/ios-shared-with-you-support-link-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-shared-with-you-support-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    strings "$binary" > "$strings_file"
    grep -q 'SWHighlightCenter' "$strings_file"
    grep -q 'isSystemCollaborationSupportAvailable' "$strings_file"
    if grep -Eqi 'swift_' "$symbols" "$strings_file"; then
        echo "unexpected Swift runtime import in $symbols" >&2
        exit 1
    fi
done

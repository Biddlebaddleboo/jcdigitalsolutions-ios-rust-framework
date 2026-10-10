#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the Natural Language link check" >&2
        exit 1
    fi
done

cat > target/ios-natural-language-status-link-imports-expected.txt <<'IMPORTS'
Foundation
NaturalLanguage
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-natural-language-status --example natural_language_status_link_probe --target "$target"
    binary="target/$target/release/examples/natural_language_status_link_probe"
    imports="target/ios-natural-language-status-link-imports-$target.txt"
    libraries="target/ios-natural-language-status-link-libraries-$target.txt"
    symbols="target/ios-natural-language-status-link-symbols-$target.txt"
    strings_file="target/ios-natural-language-status-link-strings-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-natural-language-status-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    grep -q '_NLLanguageEnglish' "$symbols"
    strings "$binary" > "$strings_file"
    grep -q 'NLContextualEmbedding' "$strings_file"
    grep -q 'contextualEmbeddingWithLanguage' "$strings_file"
    grep -q 'hasAvailableAssets' "$strings_file"
    if grep -Eqi 'swift_' "$symbols" "$strings_file"; then
        echo "unexpected Swift runtime import in $symbols" >&2
        exit 1
    fi
done

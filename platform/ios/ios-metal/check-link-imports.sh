#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS Metal link/import check" >&2
        exit 1
    fi
done

cat > target/ios-metal-link-imports-expected.txt <<'IMPORTS'
Foundation
Metal
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-metal --example ios_metal_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_metal_link_import_probe"
    imports="target/ios-metal-link-imports-$target.txt"
    libraries="target/ios-metal-link-libraries-$target.txt"
    symbols="target/ios-metal-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-metal-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_MTLCreateSystemDefaultDevice' "$symbols"
    grep -q '_objc_release' "$symbols"
    if grep -Eqi 'swift_|MTLCreateCommandQueue|MTLCommandQueue|MTKView|MPSGraph' "$symbols"; then
        echo "unexpected Swift runtime or out-of-scope Metal symbol import in $symbols" >&2
        exit 1
    fi
done

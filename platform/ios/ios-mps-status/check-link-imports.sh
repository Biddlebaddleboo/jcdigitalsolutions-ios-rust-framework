#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS MPS link/import check" >&2
        exit 1
    fi
done

cat > target/ios-mps-status-link-imports-expected.txt <<'IMPORTS'
Foundation
Metal
MetalPerformanceShaders
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.2 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-mps-status-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" cargo build --locked --release -p ios-mps-status --example link_check --target "$target"
    binary="$target_dir/$target/release/examples/link_check"
    imports="target/ios-mps-status-link-imports-$target.txt"
    libraries="target/ios-mps-status-link-libraries-$target.txt"
    symbols="target/ios-mps-status-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort > "$libraries"
    diff -u target/ios-mps-status-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -Fq '_MPSGetPreferredDevice' "$symbols"
    grep -Fq '_objc_release' "$symbols"
    if grep -Eiq 'swift|_MTLCreateSystemDefaultDevice|_MTLCommand|_MPSSupportsMTLDevice|_MPS(Graph|Image|Matrix|NDArray)' "$symbols"; then
        echo "unexpected Swift runtime or out-of-scope Metal/MPS import in $symbols" >&2
        exit 1
    fi

    build_info="target/ios-mps-status-link-build-$target.txt"
    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" || $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

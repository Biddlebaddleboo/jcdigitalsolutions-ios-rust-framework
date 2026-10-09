#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s\n' "$tool is required for the ReplayKit link/import check" >&2
        exit 1
    fi
done

cat > target/ios-replaykit-link-imports-expected.txt <<'IMPORTS'
Foundation
ReplayKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-replaykit --example ios_replaykit_link_probe --target "$target"
    binary="target/$target/release/examples/ios_replaykit_link_probe"
    imports="target/ios-replaykit-link-imports-$target.txt"
    libraries="target/ios-replaykit-link-libraries-$target.txt"
    symbols="target/ios-replaykit-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-replaykit-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|startRecording|startCapture|RPBroadcast|SCStream' "$symbols"; then
        printf '%s\n' "unexpected Swift runtime or out-of-scope capture symbol import in $symbols" >&2
        exit 1
    fi
done

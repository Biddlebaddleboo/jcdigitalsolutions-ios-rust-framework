#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the WatchConnectivity link/import check" >&2
        exit 1
    fi
done

cat > target/ios-watch-connectivity-link-imports-expected.txt <<'IMPORTS'
Foundation
WatchConnectivity
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-watch-connectivity --example ios_watch_connectivity_link_probe --target "$target"
    binary="target/$target/release/examples/ios_watch_connectivity_link_probe"
    imports="target/ios-watch-connectivity-link-imports-$target.txt"
    libraries="target/ios-watch-connectivity-link-libraries-$target.txt"
    symbols="target/ios-watch-connectivity-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-watch-connectivity-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    grep -q '_objc_msgSend' "$symbols"
    if grep -Eqi 'swift_|WCSession.*(defaultSession|activateSession|sendMessage|transfer)' "$symbols"; then
        echo "unexpected Swift runtime or out-of-scope WatchConnectivity symbol import in $symbols" >&2
        exit 1
    fi
done

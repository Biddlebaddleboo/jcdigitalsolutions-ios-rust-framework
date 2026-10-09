#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS authentication link/import check" >&2
        exit 1
    fi
done

cat > target/ios-auth-link-imports-expected.txt <<'IMPORTS'
AppTrackingTransparency
Foundation
LocalAuthentication
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --locked --release -p ios-auth --example ios_auth_link_import_probe --target "$target"
    binary="target/$target/release/examples/ios_auth_link_import_probe"
    imports="target/ios-auth-link-imports-$target.txt"
    libraries="target/ios-auth-link-libraries-$target.txt"
    symbols="target/ios-auth-link-symbols-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort > "$libraries"
    diff -u target/ios-auth-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eqi 'swift_|SecItem|SecAccessControl|SecKey|SecTrust|SecCertificate' "$symbols"; then
        echo "unexpected Swift runtime or Keychain symbol import in $symbols" >&2
        exit 1
    fi
done

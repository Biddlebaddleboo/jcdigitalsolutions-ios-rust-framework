#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS sharing share link/import check" >&2
        exit 1
    fi
done

cat > target/ios-sharing-share-link-imports-expected.txt <<'IMPORTS'
CoreFoundation
Foundation
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

feature_tree=target/ios-sharing-share-feature-tree.txt
cargo tree --locked -e features -i ios-sharing -p ios-sharing --target aarch64-apple-ios \
    --no-default-features --features share > "$feature_tree"
grep -Fq 'ios-sharing feature "share"' "$feature_tree"
if grep -Eiq 'ios-sharing feature "clipboard"|UIPasteboard|hasStrings|setString:|setItems:' \
    "$feature_tree"; then
    echo "clipboard-only APIs leaked into the ios-sharing share feature graph" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-sharing-share-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --release -p ios-sharing --no-default-features --features share \
            --example share_link_check --target "$target"
    binary="$target_dir/$target/release/examples/share_link_check"
    imports="target/ios-sharing-share-link-imports-$target.txt"
    libraries="target/ios-sharing-share-link-libraries-$target.txt"
    symbols="target/ios-sharing-share-link-symbols-$target.txt"
    strings_file="target/ios-sharing-share-link-strings-$target.txt"
    build_info="target/ios-sharing-share-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "$libraries"
    diff -u target/ios-sharing-share-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eq 'UIPasteboard|swift_|(^|_)Py[A-Z_]' "$symbols"; then
        echo "unexpected clipboard or language-runtime symbol import in $symbols" >&2
        exit 1
    fi

    strings -a "$binary" > "$strings_file"
    for marker in UIActivityViewController initWithActivityItems:applicationActivities: \
        setCompletionWithItemsHandler: popoverPresentationController setSourceView: setSourceRect: \
        presentViewController:animated:completion:; do
        if ! grep -Fq "$marker" "$strings_file"; then
            echo "missing share selector marker $marker in $strings_file" >&2
            exit 1
        fi
    done
    if grep -Eq 'UIPasteboard|hasStrings|setString:|setItems:|swift_|(^|_)Py[A-Z_]' "$strings_file"; then
        echo "unexpected clipboard or language-runtime string in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit } $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

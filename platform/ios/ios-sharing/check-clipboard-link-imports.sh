#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"
mkdir -p target

for tool in awk cargo diff grep nm otool sort strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS sharing link/import check" >&2
        exit 1
    fi
done

# This registered profile also proves the opt-in document-export feature uses only public UIKit
# selectors, remains isolated from clipboard/share features, and declares its iOS 14.0 floor.
document_export_tree=target/ios-sharing-document-export-feature-tree.txt
cargo tree --locked -e features -i ios-sharing -p ios-sharing --target aarch64-apple-ios \
    --no-default-features --features document-export > "$document_export_tree"
grep -Fq 'ios-sharing feature "document-export"' "$document_export_tree"
if grep -Eiq 'ios-sharing feature "clipboard"|ios-sharing feature "share"|UIPasteboard|UIActivityViewController|UIActivity' \
    "$document_export_tree"; then
    echo "clipboard or share-only APIs leaked into the ios-sharing document-export feature graph" >&2
    exit 1
fi

cat > target/ios-sharing-document-export-link-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    target_dir="target/ios-sharing-document-export-link-$target-minos-14.0"
    IPHONEOS_DEPLOYMENT_TARGET=14.0 CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --release -p ios-sharing --no-default-features --features document-export \
            --example document_export_link_check --target "$target"
    binary="$target_dir/$target/release/examples/document_export_link_check"
    imports="target/ios-sharing-document-export-link-imports-$target.txt"
    libraries="target/ios-sharing-document-export-link-libraries-$target.txt"
    symbols="target/ios-sharing-document-export-link-symbols-$target.txt"
    strings_file="target/ios-sharing-document-export-link-strings-$target.txt"
    build_info="target/ios-sharing-document-export-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "$libraries"
    diff -u target/ios-sharing-document-export-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eiq 'UIPasteboard|UIActivityViewController|UIActivity|swift_|Py[A-Z_]' "$symbols"; then
        echo "unexpected clipboard, share, or language-runtime symbol import in $symbols" >&2
        exit 1
    fi

    strings -a "$binary" > "$strings_file"
    for marker in UIDocumentPickerViewController initForExportingURLs:asCopy: \
        documentPicker:didPickDocumentsAtURLs: documentPickerWasCancelled: \
        presentViewController:animated:completion:; do
        if ! grep -Fq "$marker" "$strings_file"; then
            echo "missing document-export selector marker $marker in $strings_file" >&2
            exit 1
        fi
    done
    if grep -Eiq 'UIPasteboard|UIActivityViewController|UIActivity|swift_|Py[A-Z_]' "$strings_file"; then
        echo "unexpected clipboard, share, or language-runtime string in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit } $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "14.0" ]; then
        echo "$target document-export probe has deployment target ${actual_deployment_target:-unknown}; expected 14.0" >&2
        exit 1
    fi
done

# The registered profile also proves open-in-place document import is opt-in, iOS 14.0+, and
# isolated from clipboard, outgoing share, and document-export features.
document_import_tree=target/ios-sharing-document-import-feature-tree.txt
cargo tree --locked -e features -i ios-sharing -p ios-sharing --target aarch64-apple-ios \
    --no-default-features --features document-import > "$document_import_tree"
grep -Fq 'ios-sharing feature "document-import"' "$document_import_tree"
if grep -Eiq 'ios-sharing feature "(clipboard|share|document-export)"' "$document_import_tree"; then
    echo "clipboard, share, or export features leaked into the ios-sharing document-import feature graph" >&2
    exit 1
fi

document_import_ui_tree=target/ios-sharing-document-import-ui-feature-tree.txt
cargo tree --locked -e features -i objc2-ui-kit -p ios-sharing --target aarch64-apple-ios \
    --no-default-features --features document-import > "$document_import_ui_tree"
grep -Fq 'objc2-ui-kit feature "UIDocumentPickerViewController"' "$document_import_ui_tree"
grep -Fq 'objc2-ui-kit feature "objc2-uniform-type-identifiers"' "$document_import_ui_tree"
if grep -Eiq 'objc2-ui-kit feature "(UIPasteboard|UIActivity|UIActivityViewController|UIPopoverPresentationController)"' \
    "$document_import_ui_tree"; then
    echo "clipboard, share, or export UIKit features leaked into document import" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo clippy --locked --all-targets -p ios-sharing --target "$target" \
        --no-default-features --features document-import -- -D warnings
done

cat > target/ios-sharing-document-import-link-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
UniformTypeIdentifiers
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    target_dir="target/ios-sharing-document-import-link-$target-minos-14.0"
    IPHONEOS_DEPLOYMENT_TARGET=14.0 CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --release -p ios-sharing --no-default-features --features document-import \
            --example document_import_link_check --target "$target"
    binary="$target_dir/$target/release/examples/document_import_link_check"
    imports="target/ios-sharing-document-import-link-imports-$target.txt"
    libraries="target/ios-sharing-document-import-link-libraries-$target.txt"
    symbols="target/ios-sharing-document-import-link-symbols-$target.txt"
    strings_file="target/ios-sharing-document-import-link-strings-$target.txt"
    build_info="target/ios-sharing-document-import-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "$libraries"
    diff -u target/ios-sharing-document-import-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eiq 'UIPasteboard|UIActivityViewController|UIActivity|swift_|Py[A-Z_]' "$symbols"; then
        echo "unexpected clipboard, share, or language-runtime symbol import in $symbols" >&2
        exit 1
    fi

    strings -a "$binary" > "$strings_file"
    for marker in UIDocumentPickerViewController initForOpeningContentTypes:asCopy: \
        typeWithIdentifier: setAllowsMultipleSelection: documentPicker:didPickDocumentsAtURLs: \
        documentPickerWasCancelled: presentViewController:animated:completion:; do
        if ! grep -Fq "$marker" "$strings_file"; then
            echo "missing document-import selector marker $marker in $strings_file" >&2
            exit 1
        fi
    done
    if grep -Eiq 'UIPasteboard|UIActivityViewController|UIActivity|swift_|Py[A-Z_]' "$strings_file"; then
        echo "unexpected clipboard, share, or language-runtime string in $strings_file" >&2
        exit 1
    fi
    if grep -Eiq 'startAccessingSecurityScopedResource|coordinateReadingItemAtURL' \
        platform/ios/ios-sharing/src/document_import.rs; then
        echo "document import must leave security scope and provider I/O to the caller" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit } $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "14.0" ]; then
        echo "$target document-import probe has deployment target ${actual_deployment_target:-unknown}; expected 14.0" >&2
        exit 1
    fi
done

clipboard_source=platform/ios/ios-sharing/src/platform.rs
if ! awk '
    /pub fn read_image/ { reading = 1 }
    reading && index($0, "if !unsafe { self.pasteboard.hasImages() }") { preflight = NR }
    reading && index($0, "unsafe { self.pasteboard.image() }") { content_read = NR; exit }
    END { exit !(preflight && content_read && preflight < content_read) }
' "$clipboard_source"; then
    echo "image content read must follow the hasImages type preflight" >&2
    exit 1
fi

cat > target/ios-sharing-clipboard-link-imports-expected.txt <<'IMPORTS'
Foundation
UIKit
libSystem.B.dylib
libobjc.A.dylib
IMPORTS

feature_tree=target/ios-sharing-clipboard-feature-tree.txt
cargo tree --locked -e features -i ios-sharing -p ios-sharing --target aarch64-apple-ios \
    --no-default-features --features clipboard > "$feature_tree"
grep -Fq 'ios-sharing feature "clipboard"' "$feature_tree"
if grep -Eiq 'ios-sharing feature "share"|block2|UIActivityViewController|objc2-core-foundation' "$feature_tree"; then
    echo "share-only dependencies leaked into the ios-sharing clipboard feature graph" >&2
    exit 1
fi

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=10.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-sharing-clipboard-link-$target-minos-$deployment_target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --release -p ios-sharing --no-default-features --features clipboard \
            --example clipboard_link_check --target "$target"
    binary="$target_dir/$target/release/examples/clipboard_link_check"
    imports="target/ios-sharing-clipboard-link-imports-$target.txt"
    libraries="target/ios-sharing-clipboard-link-libraries-$target.txt"
    symbols="target/ios-sharing-clipboard-link-symbols-$target.txt"
    strings_file="target/ios-sharing-clipboard-link-strings-$target.txt"
    build_info="target/ios-sharing-clipboard-link-build-$target.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" \
        | LC_ALL=C sort > "$libraries"
    diff -u target/ios-sharing-clipboard-link-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    if grep -Eiq 'UIActivityViewController|UIActivity|swift_|Py[A-Z_]' "$symbols"; then
        echo "unexpected share or language-runtime symbol import in $symbols" >&2
        exit 1
    fi

    strings -a "$binary" > "$strings_file"
    for marker in UIPasteboard generalPasteboard hasStrings strings firstObject dataUsingEncoding: setString: setItems: hasImages image setImage:; do
        if ! grep -Fq "$marker" "$strings_file"; then
            echo "missing clipboard selector marker $marker in $strings_file" >&2
            exit 1
        fi
    done
    if grep -Eiq 'UIActivityViewController|UIActivity|swift_|Py[A-Z_]' "$strings_file"; then
        echo "unexpected share or language-runtime string in $strings_file" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit } $1 == "version" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

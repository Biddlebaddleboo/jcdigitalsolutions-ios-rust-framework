#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

probe_dir=$(mktemp -d "${TMPDIR:-/tmp}/ios-image-io-import-probe.XXXXXX")
trap 'rm -rf "$probe_dir"' EXIT HUP INT TERM
sysroot=$(rustc --print sysroot)
host=$(rustc -vV | sed -n 's/^host: //p')
llvm_nm="$sysroot/lib/rustlib/$host/bin/llvm-nm"
if [ ! -x "$llvm_nm" ]; then
    echo "Rust LLVM symbol tool is required for this audit: $llvm_nm" >&2
    exit 1
fi
mkdir -p "$probe_dir/src"
cat > "$probe_dir/Cargo.toml" <<EOF
[package]
name = "ios-image-io-import-probe"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["staticlib"]

[workspace]

[dependencies]
framework-image = { path = "$root/crates/framework-image" }
ios-image-io = { path = "$root/platform/ios/ios-image-io" }
EOF
cat > "$probe_dir/src/lib.rs" <<'EOF'
use framework_image::ImageMetadataReader;
use ios_image_io::IosImageMetadataBackend;

#[unsafe(no_mangle)]
pub extern "C" fn ios_image_io_import_probe() -> u32 {
    let mut reader = ImageMetadataReader::new(IosImageMetadataBackend::new());
    reader
        .read_metadata(&[])
        .map(|metadata| metadata.image_count())
        .unwrap_or(0)
}
EOF

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo build --release --manifest-path "$probe_dir/Cargo.toml" --target "$target"
    archive="$probe_dir/target/$target/release/libios_image_io_import_probe.a"
    imports="$probe_dir/imports-$target.txt"
    "$llvm_nm" -u "$archive" > "$imports"

    for symbol in \
        CGImageSourceCreateWithData \
        CGImageSourceGetCount \
        CGImageSourceCopyPropertiesAtIndex \
        CFDataCreate \
        CFDictionaryGetValue \
        CFNumberGetValue \
        CFNumberGetTypeID \
        CFGetTypeID; do
        if ! rg -q "$symbol" "$imports"; then
            echo "expected native import $symbol is absent for $target" >&2
            exit 1
        fi
    done

    if rg -qi 'CGImage(Create|SourceCreateImageAtIndex|SourceCreateThumbnailAtIndex)|CoreImage|CoreGraphics|UIKit|Photos|AVFoundation|swift|objc_msgSend|objc_retain|objc_release' "$imports"; then
        echo "unexpected image, UI, Swift, or Objective-C runtime import for $target" >&2
        exit 1
    fi

    cargo tree --locked -p ios-image-io --target "$target" -e features > "$probe_dir/features-$target.txt"
    if rg -q 'objc2-core-graphics|block2|objc2-image-io feature "(CGImageDestination|CGImageMetadata|CGImageAnimation|objc2|std)"|objc2-core-foundation feature "std"' "$probe_dir/features-$target.txt"; then
        echo "unexpected ImageIO or Objective-C binding feature for $target" >&2
        exit 1
    fi
done

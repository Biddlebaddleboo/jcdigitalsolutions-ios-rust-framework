#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang diff find grep nm otool sort vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS UI text-metrics link/import check" >&2
        exit 1
    fi
done

cat > target/ios-ui-text-metrics-imports-expected.txt <<'IMPORTS'
CoreFoundation
CoreText
libSystem.B.dylib
IMPORTS

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            sdk=iphoneos
            deployment_target=12.0
            clang_target="arm64-apple-ios${deployment_target}"
            ;;
        aarch64-apple-ios-sim)
            sdk=iphonesimulator
            deployment_target=14.0
            clang_target="arm64-apple-ios${deployment_target}-simulator"
            ;;
    esac

    target_dir="target/ios-ui-text-metrics-link-$target"
    mkdir -p "$target_dir"
    xcrun --sdk "$sdk" clang -target "$clang_target" -std=c11 -Wall -Wextra -Werror \
        -c platform/ios/ios-ui/probes/text-metrics-layout.c \
        -o "$target_dir/text-metrics-layout.o"

    RUSTFLAGS='-C link-arg=-Wl,-dead_strip_dylibs' \
        IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_text_metrics_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_text_metrics_link_import_probe"
    imports="$target_dir/imports.txt"
    libraries="$target_dir/libraries.txt"
    symbols="$target_dir/undefined-symbols.txt"
    build_info="$target_dir/build-info.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | sort -u > "$libraries"
    diff -u target/ios-ui-text-metrics-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in _CTFontCreateUIFontForLanguage _CTFontGetAscent _CTFontGetDescent _CTFontGetLeading _CFRelease; do
        if ! grep -Fq "$symbol" "$symbols"; then
            echo "$target probe is missing $symbol" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]' "$symbols" || grep -Eqi 'swift|python' "$libraries"; then
        echo "unexpected Swift/Python runtime import in $target probe" >&2
        exit 1
    fi

    vtool -show-build "$binary" > "$build_info"
    actual_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$build_info")
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-ui -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-ui" >&2
    exit 1
fi

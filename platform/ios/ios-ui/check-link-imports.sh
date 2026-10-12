#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"
mkdir -p target

for tool in awk cargo clang diff find grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS UI geometry link/import check" >&2
        exit 1
    fi
done

cat > target/ios-ui-geometry-imports-expected.txt <<'IMPORTS'
CoreGraphics
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

    target_dir="target/ios-ui-geometry-link-$target"
    mkdir -p "$target_dir"
    xcrun --sdk "$sdk" clang -target "$clang_target" -std=c11 -Wall -Wextra -Werror \
        -c platform/ios/ios-ui/probes/geometry-layout.c \
        -o "$target_dir/geometry-layout.o"

    RUSTFLAGS='-C link-arg=-Wl,-dead_strip_dylibs' \
        IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --release -p ios-ui \
        --example ios_ui_geometry_link_import_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_geometry_link_import_probe"
    imports="$target_dir/imports.txt"
    libraries="$target_dir/libraries.txt"
    symbols="$target_dir/undefined-symbols.txt"
    build_info="$target_dir/build-info.txt"

    otool -L "$binary" > "$imports"
    awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' "$imports" | LC_ALL=C sort -u > "$libraries"
    diff -u target/ios-ui-geometry-imports-expected.txt "$libraries"

    nm -u "$binary" > "$symbols"
    for symbol in _CGRectIntersection _CGRectIsNull _CGRectIsEmpty; do
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

    stack_target_dir="target/ios-ui-stack-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$stack_target_dir" \
        cargo build --locked --release -p ios-ui \
        --example ios_ui_stack_link_probe --target "$target"
    stack_binary="$stack_target_dir/$target/release/examples/ios_ui_stack_link_probe"
    stack_imports="$stack_target_dir/imports.txt"
    stack_symbols="$stack_target_dir/undefined-symbols.txt"
    stack_build_info="$stack_target_dir/build-info.txt"

    otool -L "$stack_binary" > "$stack_imports"
    if ! grep -Fq '/UIKit.framework/UIKit' "$stack_imports"; then
        echo "$target stack probe does not link UIKit" >&2
        exit 1
    fi

    nm -u "$stack_binary" > "$stack_symbols"
    for symbol in _objc_getClass _objc_msgSend; do
        if ! grep -Fq "$symbol" "$stack_symbols"; then
            echo "$target stack probe is missing $symbol" >&2
            exit 1
        fi
    done
    for symbol in UIStackView 'addArrangedSubview:' 'setAxis:' 'setSpacing:'; do
        if ! strings "$stack_binary" | grep -Fq "$symbol"; then
            echo "$target stack probe is missing $symbol" >&2
            exit 1
        fi
    done
    for symbol in 'removeArrangedSubview:' 'setAlignment:' 'setDistribution:'; do
        if ! strings "$stack_binary" | grep -Fq "$symbol"; then
            echo "$target stack probe is missing $symbol" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]' "$stack_symbols" || grep -Eqi 'swift|python' "$stack_imports"; then
        echo "unexpected Swift/Python runtime import in $target stack probe" >&2
        exit 1
    fi

    vtool -show-build "$stack_binary" > "$stack_build_info"
    actual_stack_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$stack_build_info")
    if [ "$actual_stack_deployment_target" != "$deployment_target" ]; then
        echo "$target stack probe has deployment target ${actual_stack_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi

    scroll_target_dir="target/ios-ui-scroll-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$scroll_target_dir" \
        cargo build --locked --release -p ios-ui \
        --example ios_ui_scroll_view_link_probe --target "$target"
    scroll_binary="$scroll_target_dir/$target/release/examples/ios_ui_scroll_view_link_probe"
    scroll_imports="$scroll_target_dir/imports.txt"
    scroll_symbols="$scroll_target_dir/undefined-symbols.txt"
    scroll_build_info="$scroll_target_dir/build-info.txt"

    otool -L "$scroll_binary" > "$scroll_imports"
    if ! grep -Fq '/UIKit.framework/UIKit' "$scroll_imports"; then
        echo "$target scroll probe does not link UIKit" >&2
        exit 1
    fi

    nm -u "$scroll_binary" > "$scroll_symbols"
    for symbol in _objc_getClass _objc_msgSend; do
        if ! grep -Fq "$symbol" "$scroll_symbols"; then
            echo "$target scroll probe is missing $symbol" >&2
            exit 1
        fi
    done
    for symbol in UIScrollView 'setContentSize:' 'contentLayoutGuide' 'frameLayoutGuide' 'topAnchor' 'leadingAnchor' 'bottomAnchor' 'trailingAnchor' 'widthAnchor' 'constraintEqualToAnchor:' 'setActive:' 'setTranslatesAutoresizingMaskIntoConstraints:'; do
        if ! strings "$scroll_binary" | grep -Fq "$symbol"; then
            echo "$target scroll probe is missing $symbol" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]' "$scroll_symbols" || grep -Eqi 'swift|python' "$scroll_imports"; then
        echo "unexpected Swift/Python runtime import in $target scroll probe" >&2
        exit 1
    fi

    vtool -show-build "$scroll_binary" > "$scroll_build_info"
    actual_scroll_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$scroll_build_info")
    if [ "$actual_scroll_deployment_target" != "$deployment_target" ]; then
        echo "$target scroll probe has deployment target ${actual_scroll_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi

    toggle_target_dir="target/ios-ui-toggle-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$toggle_target_dir" \
        cargo build --locked --release -p ios-ui \
        --example ios_ui_toggle_link_probe --target "$target"
    toggle_binary="$toggle_target_dir/$target/release/examples/ios_ui_toggle_link_probe"
    toggle_imports="$toggle_target_dir/imports.txt"
    toggle_symbols="$toggle_target_dir/undefined-symbols.txt"
    toggle_build_info="$toggle_target_dir/build-info.txt"

    otool -L "$toggle_binary" > "$toggle_imports"
    if ! grep -Fq '/UIKit.framework/UIKit' "$toggle_imports"; then
        echo "$target toggle probe does not link UIKit" >&2
        exit 1
    fi

    nm -u "$toggle_binary" > "$toggle_symbols"
    for symbol in _objc_getClass _objc_msgSend; do
        if ! grep -Fq "$symbol" "$toggle_symbols"; then
            echo "$target toggle probe is missing $symbol" >&2
            exit 1
        fi
    done
    for symbol in UISwitch isOn 'setOn:animated:' 'toggleChanged:' 'removeTarget:action:forControlEvents:'; do
        if ! strings "$toggle_binary" | grep -Fq "$symbol"; then
            echo "$target toggle probe is missing $symbol" >&2
            exit 1
        fi
    done
    if grep -Eqi 'swift_|Py[A-Z_]' "$toggle_symbols" || grep -Eqi 'swift|python' "$toggle_imports"; then
        echo "unexpected Swift/Python runtime import in $target toggle probe" >&2
        exit 1
    fi

    vtool -show-build "$toggle_binary" > "$toggle_build_info"
    actual_toggle_deployment_target=$(awk '$1 == "minos" { print $2; exit }' "$toggle_build_info")
    if [ "$actual_toggle_deployment_target" != "$deployment_target" ]; then
        echo "$target toggle probe has deployment target ${actual_toggle_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
done

if find platform/ios/ios-ui -type f -name '*.swift' -print -quit | grep -q .; then
    echo "Swift source found under platform/ios/ios-ui" >&2
    exit 1
fi

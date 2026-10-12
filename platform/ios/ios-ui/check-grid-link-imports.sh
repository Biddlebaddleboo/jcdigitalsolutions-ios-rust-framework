#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo grep nm otool strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the iOS UI grid link check" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac

    target_dir="target/ios-ui-grid-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_grid_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_grid_link_probe"
    imports=$(otool -L "$binary")
    printf '%s\n' "$imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target grid probe does not link UIKit" >&2
        exit 1
    }
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target grid probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for symbol in UIStackView 'addArrangedSubview:' 'insertArrangedSubview:atIndex:' \
        'removeArrangedSubview:' 'setDistribution:'; do
        printf '%s\n' "$binary_strings" | grep -F "$symbol" >/dev/null || {
            echo "$target grid probe is missing UIKit symbol or selector $symbol" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eqi 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target grid probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target grid probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s UIKit grid selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"

    list_target_dir="target/ios-ui-list-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$list_target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_list_link_probe --target "$target"
    list_binary="$list_target_dir/$target/release/examples/ios_ui_list_link_probe"
    list_imports=$(otool -L "$list_binary")
    printf '%s\n' "$list_imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target list probe does not link UIKit" >&2
        exit 1
    }
    list_symbols=$(nm -u "$list_binary")
    list_strings=$(strings "$list_binary")
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$list_symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target list probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for symbol in UITableView UITableViewCell 'setDataSource:' 'setDelegate:' \
        'tableView:numberOfRowsInSection:' 'tableView:cellForRowAtIndexPath:' \
        'tableView:didSelectRowAtIndexPath:' 'dequeueReusableCellWithIdentifier:' \
        'initWithStyle:reuseIdentifier:' textLabel detailTextLabel reloadData \
        'setAccessibilityLabel:' 'setAccessibilityHint:' 'setAccessibilityIdentifier:'; do
        printf '%s\n' "$list_strings" | grep -F "$symbol" >/dev/null || {
            echo "$target list/accessibility probe is missing UIKit symbol or selector $symbol" >&2
            exit 1
        }
    done
    if printf '%s\n' "$list_symbols" "$list_imports" | grep -Eqi 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target list probe" >&2
        exit 1
    fi
    list_deployment_target=$(vtool -show-build "$list_binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$list_deployment_target" != "$deployment_target" ]; then
        echo "$target list probe has deployment target ${list_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s UITableView reuse, selection, and button accessibility selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"

    context_menu_target_dir="target/ios-ui-context-menu-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$context_menu_target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_context_menu_link_probe --target "$target"
    context_menu_binary="$context_menu_target_dir/$target/release/examples/ios_ui_context_menu_link_probe"
    context_menu_imports=$(otool -L "$context_menu_binary")
    printf '%s\n' "$context_menu_imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target context-menu probe does not link UIKit" >&2
        exit 1
    }
    context_menu_symbols=$(nm -u "$context_menu_binary")
    context_menu_strings=$(strings "$context_menu_binary")
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$context_menu_symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target context-menu probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for symbol in UIContextMenuInteraction UIContextMenuInteractionDelegate UIAction UIMenu \
        'initWithDelegate:' 'contextMenuInteraction:configurationForMenuAtLocation:' \
        'configurationWithIdentifier:previewProvider:actionProvider:' \
        'actionWithTitle:image:identifier:handler:' 'addInteraction:' 'removeInteraction:'; do
        printf '%s\n' "$context_menu_strings" | grep -F "$symbol" >/dev/null || {
            echo "$target context-menu probe is missing UIKit class or selector $symbol" >&2
            exit 1
        }
    done
    if printf '%s\n' "$context_menu_symbols" "$context_menu_imports" | grep -Eqi 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target context-menu probe" >&2
        exit 1
    fi
    context_menu_deployment_target=$(vtool -show-build "$context_menu_binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$context_menu_deployment_target" != "$deployment_target" ]; then
        echo "$target context-menu probe has deployment target ${context_menu_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    grep -F 'objc2::available!(ios = 13.0, ..)' platform/ios/ios-ui/src/platform/context_menu.rs >/dev/null || {
        echo "$target context-menu probe is missing its runtime iOS 13 availability guard" >&2
        exit 1
    }
    printf '%s UIContextMenuInteraction selectors and iOS 13 runtime guard verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"

    selectable_text_target_dir="target/ios-ui-selectable-text-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$selectable_text_target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_selectable_text_link_probe --target "$target"
    selectable_text_binary="$selectable_text_target_dir/$target/release/examples/ios_ui_selectable_text_link_probe"
    selectable_text_imports=$(otool -L "$selectable_text_binary")
    printf '%s\n' "$selectable_text_imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target selectable-text probe does not link UIKit" >&2
        exit 1
    }
    selectable_text_symbols=$(nm -u "$selectable_text_binary")
    selectable_text_strings=$(strings "$selectable_text_binary")
    for symbol in objc_getClass objc_msgSend; do
        printf '%s\n' "$selectable_text_symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target selectable-text probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for symbol in UITextView UITextViewDelegate 'setEditable:' 'setSelectable:' \
        'setScrollEnabled:' 'setText:' 'setDelegate:' 'textViewDidChange:' \
        'becomeFirstResponder' 'resignFirstResponder' text; do
        printf '%s\n' "$selectable_text_strings" | grep -F "$symbol" >/dev/null || {
            echo "$target selectable-text probe is missing UIKit class or selector $symbol" >&2
            exit 1
        }
    done
    if printf '%s\n' "$selectable_text_symbols" "$selectable_text_imports" | grep -Eqi 'swift_|Py[A-Z_]|swift|python'; then
        echo "unexpected Swift/Python runtime import in $target selectable-text probe" >&2
        exit 1
    fi
    selectable_text_deployment_target=$(vtool -show-build "$selectable_text_binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$selectable_text_deployment_target" != "$deployment_target" ]; then
        echo "$target selectable-text probe has deployment target ${selectable_text_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    grep -F 'native.setEditable(false);' platform/ios/ios-ui/src/platform/selectable_text.rs >/dev/null || {
        echo "$target selectable-text view is missing its read-only policy" >&2
        exit 1
    }
    grep -F 'native.setSelectable(true);' platform/ios/ios-ui/src/platform/selectable_text.rs >/dev/null || {
        echo "$target selectable-text view is missing its selection policy" >&2
        exit 1
    }
    grep -F 'native.setEditable(true);' platform/ios/ios-ui/src/platform/selectable_text.rs >/dev/null || {
        echo "$target text editor is missing its edit policy" >&2
        exit 1
    }
    grep -F 'textViewDidChange:' platform/ios/ios-ui/src/platform/selectable_text.rs >/dev/null || {
        echo "$target text editor is missing its edit callback" >&2
        exit 1
    }
    printf '%s UITextView read-only/editable policies and delegate selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"
done

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for tool in awk cargo grep nm otool strings vtool; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "$tool is required for the collection-grid link check" >&2
        exit 1
    fi
done

grep -F 'objc2::available!(ios = 13.0, ..)' platform/ios/ios-ui/src/platform/collection_grid.rs >/dev/null || {
    echo "collection grid is missing its iOS 13 runtime availability guard" >&2
    exit 1
}

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios) deployment_target=12.0 ;;
        aarch64-apple-ios-sim) deployment_target=14.0 ;;
    esac
    target_dir="target/ios-ui-collection-grid-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$deployment_target" CARGO_TARGET_DIR="$target_dir" \
        cargo build --locked --offline --release -p ios-ui \
        --example ios_ui_collection_grid_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_ui_collection_grid_link_probe"
    imports=$(otool -L "$binary")
    printf '%s\n' "$imports" | grep -F '/UIKit.framework/UIKit' >/dev/null || {
        echo "$target collection-grid probe does not link UIKit" >&2
        exit 1
    }
    symbols=$(nm -u "$binary")
    binary_strings=$(strings "$binary")
    for symbol in objc_getClass objc_msgSend UICollectionElementKindSectionHeader; do
        printf '%s\n' "$symbols" | grep -F "$symbol" >/dev/null || {
            echo "$target collection-grid probe is missing Objective-C symbol $symbol" >&2
            exit 1
        }
    done
    for symbol in UICollectionView UICollectionViewCell UICollectionReusableView \
        UICollectionViewCompositionalLayout NSCollectionLayoutBoundarySupplementaryItem \
        UICollectionViewDiffableDataSource NSDiffableDataSourceSnapshot \
        NSCollectionLayoutSection NSCollectionLayoutGroup NSCollectionLayoutItem \
        NSCollectionLayoutSize NSCollectionLayoutDimension \
        UIButton UISwitch UILabel \
        'initWithSectionProvider:' 'appendSectionsWithIdentifiers:' \
        'appendItemsWithIdentifiers:intoSectionWithIdentifier:' 'reloadItemsWithIdentifiers:' \
        'setInterGroupSpacing:' 'setOrthogonalScrollingBehavior:' \
        numberWithUnsignedLongLong unsignedLongLongValue \
        'dequeueReusableCellWithReuseIdentifier:forIndexPath:' \
        'applySnapshot:animatingDifferences:completion:' \
        'setSupplementaryViewProvider:' 'reloadSectionsWithIdentifiers:' \
        'registerClass:forSupplementaryViewOfKind:withReuseIdentifier:' \
        'dequeueReusableSupplementaryViewOfKind:withReuseIdentifier:forIndexPath:' \
        'initWithFrame:' \
        'itemIdentifierForIndexPath:' 'collectionView:shouldSelectItemAtIndexPath:' \
        'collectionView:didSelectItemAtIndexPath:' 'gridActionTapped:' 'gridToggleChanged:' \
        'setOn:animated:' 'setEnabled:'; do
        printf '%s\n' "$binary_strings" | grep -F "$symbol" >/dev/null || {
            echo "$target collection-grid probe is missing UIKit symbol or selector $symbol" >&2
            exit 1
        }
    done
    if printf '%s\n' "$symbols" "$imports" | grep -Eq 'swift_|Swift\.framework|libswift|(^|[^[:alnum:]_])Py[A-Z_]|Python\.framework|libpython'; then
        echo "unexpected Swift/Python runtime import in $target collection-grid probe" >&2
        exit 1
    fi
    actual_deployment_target=$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')
    if [ "$actual_deployment_target" != "$deployment_target" ]; then
        echo "$target collection-grid probe has deployment target ${actual_deployment_target:-unknown}; expected $deployment_target" >&2
        exit 1
    fi
    printf '%s collection-grid UIKit selectors verified at iOS %s; probe not executed\n' \
        "$target" "$deployment_target"
done

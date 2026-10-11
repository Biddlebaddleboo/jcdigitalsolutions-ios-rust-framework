#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
cd "$repo_root"

for tool in awk cargo diff find grep nm otool sort strings vtool xcrun; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '%s is required for the iOS Contacts fetch link/import check\n' "$tool" >&2
        exit 1
    fi
done

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    case "$target" in
        aarch64-apple-ios)
            sdk_name=iphoneos
            minimum=10.0
            clang_target=arm64-apple-ios10.0
            min_flag=-miphoneos-version-min=10.0
            ;;
        aarch64-apple-ios-sim)
            sdk_name=iphonesimulator
            minimum=14.0
            clang_target=arm64-apple-ios14.0-simulator
            min_flag=-mios-simulator-version-min=14.0
            ;;
    esac

    sdk=$(xcrun --sdk "$sdk_name" --show-sdk-path)
    xcrun --sdk "$sdk_name" clang -target "$clang_target" -isysroot "$sdk" \
        -fsyntax-only platform/ios/ios-contacts/examples/contacts_fetch_api_floor.m

    target_dir="target/ios-contacts-link-$target"
    IPHONEOS_DEPLOYMENT_TARGET="$minimum" RUSTFLAGS="-C link-arg=$min_flag" \
        CARGO_TARGET_DIR="$target_dir" cargo +1.94.1 build --locked --release \
        -p ios-contacts --example ios_contacts_fetch_link_probe --target "$target"
    binary="$target_dir/$target/release/examples/ios_contacts_fetch_link_probe"
    imports=$(otool -L "$binary")
    actual_libraries=$(printf '%s\n' "$imports" | awk 'NR > 1 { path = $1; sub(/^.*\//, "", path); print path }' | LC_ALL=C sort)
    expected_libraries=$(printf '%s\n' Contacts Foundation libSystem.B.dylib libobjc.A.dylib | LC_ALL=C sort)
    if [ "$actual_libraries" != "$expected_libraries" ]; then
        printf '%s probe imports unexpected libraries:\n%s\n' "$target" "$actual_libraries" >&2
        exit 1
    fi

    symbols=$(nm -u "$binary")
    printf '%s\n' "$symbols" | grep -F objc_msgSend >/dev/null || {
        printf '%s probe does not import objc_msgSend\n' "$target" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F objc_getClass >/dev/null || {
        printf '%s probe does not import objc_getClass for public Contacts class lookup\n' "$target" >&2
        exit 1
    }
    printf '%s\n' "$symbols" | grep -F CNContactThumbnailImageDataKey >/dev/null || {
        printf '%s probe does not import the public thumbnail key descriptor\n' "$target" >&2
        exit 1
    }
    binary_strings=$(strings "$binary")
    for class in CNMutableContact CNSaveRequest CNContactVCardSerialization; do
        printf '%s\n' "$binary_strings" | grep -Fx "$class" >/dev/null || {
            printf '%s probe does not reference the public Contacts runtime class name %s\n' "$target" "$class" >&2
            exit 1
        }
    done
    printf '%s\n' "$binary_strings" | grep -F 'unifiedContactWithIdentifier:keysToFetch:error:' >/dev/null || {
        printf '%s probe does not link the public single-contact fetch selector\n' "$target" >&2
        exit 1
    }
    for selector in \
        'predicateForContactsMatchingName:' \
        'enumerateContactsWithFetchRequest:error:usingBlock:' \
        'addContact:toContainerWithIdentifier:' \
        'mutableCopy' \
        'updateContact:' \
        'executeSaveRequest:error:' \
        'phoneNumberWithStringValue:' \
        'labeledValueWithLabel:value:' \
        'setGivenName:' \
        'setFamilyName:' \
        'setOrganizationName:' \
        'setPhoneNumbers:' \
        'setEmailAddresses:' \
        'deleteContact:' \
        'descriptorForRequiredKeys' \
        'dataWithContacts:error:' \
        'thumbnailImageData'; do
        printf '%s\n' "$binary_strings" | grep -F "$selector" >/dev/null || {
            printf '%s probe does not link the public Contacts selector %s\n' "$target" "$selector" >&2
            exit 1
        }
    done
    if printf '%s\n%s\n' "$imports" "$symbols" | grep -E 'libswift|SwiftUI|PhotosUI|_swift_|_\$s[0-9]' >/dev/null; then
        printf '%s probe unexpectedly imports Swift or UI runtime dependencies\n' "$target" >&2
        exit 1
    fi

    build=$(vtool -show-build "$binary")
    printf '%s\n' "$build" | grep -E "(minos|version)[[:space:]]+$minimum([.]0)?([[:space:]]|$)" >/dev/null || {
        printf '%s probe does not declare the iOS %s deployment minimum\n' "$target" "$minimum" >&2
        exit 1
    }
    printf '%s public Contacts/Foundation imports and iOS %s deployment minimum verified; probe not executed\n' \
        "$target" "$minimum"
done

if find platform/ios/ios-contacts -type f -name '*.swift' -print -quit | grep -q .; then
    echo 'Swift source found under platform/ios/ios-contacts' >&2
    exit 1
fi

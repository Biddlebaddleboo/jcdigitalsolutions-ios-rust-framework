#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
cd "$root"

source_files='platform/ios/ios-contact-picker/Cargo.toml platform/ios/ios-contact-picker/src/lib.rs platform/ios/ios-contact-picker/src/platform.rs platform/ios/ios-contact-picker/src/session_policy.rs'
forbidden='CNContactStore|requestAccessForEntityType_completionHandler|requestAuthorizationForEntityType|authorizationStatusForEntityType|NSContactsUsageDescription|enumerateContactsWithFetchRequest_error_usingBlock|unifiedContactsMatchingPredicate|predicateForContactsMatchingName|CNContactIdentifierKey|contactPicker_didSelectContacts|contactPicker_didSelectContactProperty|contactPicker_didSelectContactProperties|didSelectContacts:|didSelectContactProperty:|didSelectContactProperties:'

check_rejected() {
    if grep -En "$forbidden" "$1" >/dev/null 2>&1; then
        return 1
    fi
    return 0
}

for file in $source_files; do
    if ! check_rejected "$file"; then
        echo "forbidden Contacts store, authorization, or multi/property-selection API found in $file" >&2
        exit 1
    fi
done

grep -Fq '#[unsafe(method(contactPicker:didSelectContact:))]' platform/ios/ios-contact-picker/src/platform.rs || {
    echo 'single-contact delegate selector is missing' >&2
    exit 1
}
grep -Fq '#[unsafe(method(contactPickerDidCancel:))]' platform/ios/ios-contact-picker/src/platform.rs || {
    echo 'picker cancellation delegate selector is missing' >&2
    exit 1
}
for key in \
    CNContactGivenNameKey \
    CNContactFamilyNameKey \
    CNContactOrganizationNameKey \
    CNContactPhoneNumbersKey \
    CNContactEmailAddressesKey; do
    grep -Fq "$key" platform/ios/ios-contact-picker/src/platform.rs || {
        echo "selected-contact detail card key is missing: $key" >&2
        exit 1
    }
done
grep -Fq 'IosContactPickerSession::present' platform/ios/ios-contact-picker/examples/ios_contact_picker_link_probe.rs || {
    echo 'link probe must use the public Rust session API' >&2
    exit 1
}

tmp=$(mktemp -d "${TMPDIR:-/tmp}/ios-contact-picker-surface.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
for fixture in \
    'CNContactStore' \
    'requestAccessForEntityType_completionHandler' \
    'requestAuthorizationForEntityType' \
    'authorizationStatusForEntityType' \
    'NSContactsUsageDescription' \
    'enumerateContactsWithFetchRequest_error_usingBlock' \
    'unifiedContactsMatchingPredicate' \
    'predicateForContactsMatchingName' \
    'CNContactIdentifierKey' \
    'contactPicker_didSelectContacts' \
    'contactPicker_didSelectContactProperty' \
    'contactPicker_didSelectContactProperties' \
    'didSelectContacts:' \
    'didSelectContactProperty:' \
    'didSelectContactProperties:'; do
    printf '%s\n' "$fixture" > "$tmp/fixture.rs"
    if check_rejected "$tmp/fixture.rs"; then
        echo "negative surface guard failed to reject: $fixture" >&2
        exit 1
    fi
done

echo 'ContactsUI source surface and fifteen forbidden-API negative fixtures passed'

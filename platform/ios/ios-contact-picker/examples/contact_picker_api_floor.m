#import <ContactsUI/CNContactPickerViewController.h>

@interface ContactPickerApiFloorProbe : NSObject <CNContactPickerDelegate>
@end

@implementation ContactPickerApiFloorProbe
- (void)contactPickerDidCancel:(CNContactPickerViewController *)picker {
    (void)picker;
}
- (void)contactPicker:(CNContactPickerViewController *)picker didSelectContact:(CNContact *)contact {
    (void)picker;
    (void)contact;
}
@end

void contact_picker_api_floor_probe(CNContactPickerViewController *picker) {
    picker.displayedPropertyKeys = @[
        CNContactGivenNameKey,
        CNContactFamilyNameKey,
        CNContactOrganizationNameKey,
        CNContactPhoneNumbersKey,
        CNContactEmailAddressesKey,
    ];
    picker.delegate = nil;
}

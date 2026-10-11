#import <Contacts/Contacts.h>

void contacts_fetch_api_floor(CNContactStore *store, NSString *identifier) {
    NSError *error = nil;
    CNContact *contact = [store unifiedContactWithIdentifier:identifier
                                                keysToFetch:@[
                                                    CNContactIdentifierKey,
                                                    CNContactGivenNameKey,
                                                    CNContactFamilyNameKey,
                                                    CNContactOrganizationNameKey,
                                                    CNContactPhoneNumbersKey,
                                                    CNContactEmailAddressesKey,
                                                ]
                                                       error:&error];
    (void)contact;
    (void)error;
}

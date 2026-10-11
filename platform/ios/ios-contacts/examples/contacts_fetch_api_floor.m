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

void contacts_name_search_api_floor(CNContactStore *store, NSString *name) {
    NSError *error = nil;
    NSArray<id<CNKeyDescriptor>> *keys = @[
        CNContactIdentifierKey,
        CNContactGivenNameKey,
        CNContactFamilyNameKey,
        CNContactOrganizationNameKey,
        CNContactPhoneNumbersKey,
        CNContactEmailAddressesKey,
    ];
    CNContactFetchRequest *request = [[CNContactFetchRequest alloc] initWithKeysToFetch:keys];
    request.predicate = [CNContact predicateForContactsMatchingName:name];
    BOOL succeeded = [store enumerateContactsWithFetchRequest:request
                                                        error:&error
                                                   usingBlock:^(CNContact *contact, BOOL *stop) {
        (void)contact;
        *stop = YES;
    }];
    (void)succeeded;
    (void)error;
}

void contacts_create_api_floor(CNContactStore *store) {
    CNMutableContact *contact = [CNMutableContact new];
    contact.givenName = @"Ada";
    contact.phoneNumbers = @[
        [CNLabeledValue labeledValueWithLabel:nil
                                        value:[CNPhoneNumber phoneNumberWithStringValue:@"+1 555 0100"]],
    ];
    contact.emailAddresses = @[
        [CNLabeledValue labeledValueWithLabel:@"work" value:@"ada@example.test"],
    ];
    CNSaveRequest *request = [CNSaveRequest new];
    [request addContact:contact toContainerWithIdentifier:nil];
    NSError *error = nil;
    BOOL succeeded = [store executeSaveRequest:request error:&error];
    (void)succeeded;
    (void)error;
}

void contacts_update_api_floor(CNContactStore *store, NSString *identifier) {
    NSError *error = nil;
    CNContact *contact = [store unifiedContactWithIdentifier:identifier
                                                keysToFetch:@[
                                                    CNContactIdentifierKey,
                                                    CNContactGivenNameKey,
                                                ]
                                                       error:&error];
    CNMutableContact *mutableContact = [contact mutableCopy];
    mutableContact.givenName = @"";
    CNSaveRequest *request = [CNSaveRequest new];
    [request updateContact:mutableContact];
    BOOL succeeded = [store executeSaveRequest:request error:&error];
    (void)succeeded;
    (void)error;
}

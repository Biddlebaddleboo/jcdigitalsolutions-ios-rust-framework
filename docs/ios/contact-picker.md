# iOS user-mediated contact picker

`ios-contact-picker` presents Apple's public `CNContactPickerViewController` and returns one
user-selected contact as an owned `framework_contacts::Contact` snapshot. The picker is an
independent UIKit/ContactsUI crate; it does not add ContactsUI to the `ios-contacts` store backend.

```rust,ignore
use ios_contact_picker::{ContactPickerEvent, IosContactPickerSession};

let picker = IosContactPickerSession::present(main_thread, presenter, |event| match event {
    ContactPickerEvent::Selected(contact) => use_selected_contact(contact),
    ContactPickerEvent::Cancelled => {},
});
```

The caller must keep the session alive while the picker should remain available. A selected contact
or the user's Cancel action produces one terminal callback on the main thread. `cancel()` requests
dismissal and reports `Cancelled` once; dropping an active session detaches the callback and
requests dismissal without a callback. UIKit dismissal is asynchronous and this API does not claim
that a view appeared or was dismissed at runtime.

The picker requires neither Contacts authorization nor `NSContactsUsageDescription`. The app
receives only the contact the user selects, not full-store access; the picker does not call
`CNContactStore`, request permission, enumerate records, or modify contacts. It exposes the selected
contact's common identifier, name, organization, phone, and email fields as Rust-owned values.
Unselected contacts are not returned. See Apple's [ContactsUI picker](https://developer.apple.com/documentation/contactsui/cncontactpickerviewcontroller)
and [delegate](https://developer.apple.com/documentation/contactsui/cncontactpickerdelegate)
documentation.

`CNContactPickerViewController` is public from iOS 9.0. The registered Rust target floors are iOS
10.0 for device and iOS 14.0 for Simulator. Host policy tests and compiler/link checks do not prove
picker presentation, a user's selection, live cancellation, Simulator interaction, or physical-
device behavior.

# iOS Contacts authorization

`ios-contacts` implements the portable [`framework-contacts`](../capabilities/contacts.md)
authorization contract through public Contacts APIs. It can query authorization status and
explicitly request access; it does not fetch or enumerate contacts, expose identifiers, edit data,
present ContactsUI, or provide a picker or button.

```rust,ignore
use framework_contacts::{Contacts, ContactsAuthorization};
use ios_contacts::IosContactsBackend;

async fn request_contacts_access() -> Result<ContactsAuthorization, framework_contacts::ContactsError> {
    let mut contacts = Contacts::new(IosContactsBackend::new());
    let before = contacts.authorization_status();
    if matches!(before, ContactsAuthorization::NotDetermined) {
        contacts.request_authorization().await
    } else {
        Ok(before)
    }
}
```

The application supplies its own executor or polls the future itself. `IosContactsBackend::new()`
creates caller-owned backend state without querying authorization or prompting. The status query
does not prompt. A prompt-capable native call begins only when `request_authorization()`'s future
is first polled. The app must include the exact `NSContactsUsageDescription` key in its
`Info.plist` with user-understandable purpose text before making that request. This crate does not
modify the host app's property list.

## Status, permission, and OS availability

The `CNContactStore` authorization status API, Contacts entity, and `requestAccessForEntityType`
operation have an iOS 9.0 API floor according to the installed Xcode 26.6 / iPhoneOS 26.5 SDK
headers. `CNAuthorizationStatusLimited` is separately marked as introduced in iOS 18.0 by those
headers. This reports the local SDK declarations; it does not assert an Xcode 27 baseline.

The backend maps native Not Determined, Restricted, Denied, Authorized, and Limited states to
distinct portable variants. Limited is not promoted to full Authorized. It permits only the
contact subset selected by the user. If a native status value is not recognized, the portable
result is Unknown. Authorization status can change outside the process; query again when the app
needs current state. This API does not observe Settings changes or fetch contact data.

## Request lifecycle and errors

The request invokes `CNContactStore::requestAccessForEntityType_completionHandler` for
`CNEntityType::Contacts`. Apple's callback can run on an arbitrary queue, so the adapter contains
the callback result in thread-safe owned state and does not assume a main-thread callback. It
re-queries `CNContactStore::authorizationStatusForEntityType(CNEntityType::Contacts)` inside the
callback, because the callback's `granted` boolean does not distinguish full from Limited access.
The re-queried status is returned for a recognized decision, including denial or Limited access.
An unexpected native error with no classifiable decision maps to `ContactsError::Backend` with
`ErrorKind::Platform` and a representable nonzero `NSError.code()` when available.

The future is lazy: dropping it before first poll performs no native request. Dropping it after
start detaches its waker and result interest but cannot promise to dismiss an OS permission prompt.
Native callback state remains alive until completion, accepts one result, and discards late results
after detachment. Panics are contained at the block boundary and map to an internal operation
error. Permission denial is a status result, not a transport failure.

Creating this backend or observing Authorized/Limited status does not enable contact access through
this crate. No fetch method exists in this slice. Automated checks compile and lint the portable
and native adapters; they do not show the permission UI, exercise consent or Limited selection,
verify Settings changes, or prove contact-data access.

## Dependency and linkage

The iOS-only adapter uses `objc2-contacts` 0.3.2 with default features disabled and only its
`CNContactStore` and `block2` features enabled, plus `objc2`, Foundation `NSError`, and `block2`.
No contact-record, ContactsUI, or unrelated Contacts binding features are enabled. The portable
D24 crate exposes no binding types. The adapter requires the public Contacts and Foundation
frameworks at final link; no Swift source, UI framework, or private API is added. The package
checks do not inspect a final host-app binary for imports.

References: [Apple CNContactStore](https://developer.apple.com/documentation/contacts/cncontactstore),
[Apple Limited authorization](https://developer.apple.com/documentation/contacts/cnauthorizationstatus/limited?language=objc),
[Apple accessing the contact store](https://developer.apple.com/documentation/contacts/accessing-the-contact-store),
and [objc2 Contacts 0.3.2 bindings](https://docs.rs/objc2-contacts/0.3.2/objc2_contacts/).

# iOS Contacts authorization

`ios-contacts` implements the portable [`framework-contacts`](../capabilities/contacts.md)
authorization and read contract through public Contacts APIs. It can query status, explicitly
request access, fetch a caller-selected record by local identifier, and search names with a
caller-supplied result cap. It does not fetch the whole store, edit contacts, present ContactsUI,
or provide a picker or button.

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

After the application has obtained access and another UI/API has supplied a local identifier, it
can fetch one owned record without a prompt:

```rust,ignore
use framework_contacts::Contacts;
use ios_contacts::IosContactsBackend;

fn read_selected_contact(identifier: &str) -> Result<framework_contacts::Contact, framework_contacts::ContactsError> {
    Contacts::new(IosContactsBackend::new()).fetch_contact(identifier)
}
```

The application supplies its own executor or polls the future itself. The host app must include
the exact `NSContactsUsageDescription` key with user-understandable purpose text before requesting
or accessing contact data. `IosContactsBackend::new()`
creates caller-owned backend state without querying authorization or prompting. The status query
does not prompt. A prompt-capable native call begins only when `request_authorization()`'s future
is first polled. This crate does not modify the host app's property list.

## Status, permission, and OS availability

The `CNContactStore` authorization status API, Contacts entity, and `requestAccessForEntityType`
operation have an iOS 9.0 API floor according to the installed Xcode 26.6 / iPhoneOS 26.5 SDK
headers. `CNAuthorizationStatusLimited` is separately marked as introduced in iOS 18.0 by those
headers. This reports the local SDK declarations; it does not assert an Xcode 27 baseline.

The backend maps native Not Determined, Restricted, Denied, Authorized, and Limited states to
distinct portable variants. Limited is not promoted to full Authorized. It permits only the
contact subset selected by the user. If a native status value is not recognized, the portable
result is Unknown. Authorization status can change outside the process; query again when the app
needs current state. The record fetch path re-queries status and proceeds only for Authorized or
Limited. Limited access does not imply the selected identifier is visible.

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

Fetch rejects empty/NUL-containing identifiers as `InvalidIdentifier`, reports insufficient
authorization as `AuthorizationRequired(status)`, maps `CNErrorCodeRecordDoesNotExist` in
`CNErrorDomain` to `NotFound`, and preserves other representable native codes as platform errors.
It calls `unifiedContactWithIdentifier:keysToFetch:error:` for only identifier, given name, family
name, organization, phone numbers, and email addresses; it copies all results into Rust-owned
strings and vectors before the autorelease pool ends. Phone/email labels and order reflect the
native result and are not durable sort contracts. If authorization is revoked between the status
query and fetch, the adapter re-queries status on native failure.

Limited access includes only the user-selected subset. A non-selected identifier may resolve as
not found, so the result does not prove that the contact is absent from the full store. Contacts
identifiers are local handles, not durable identity; deletion or unification can make a saved
identifier stale. The fetched unified record's returned identifier may differ from the supplied
identifier. Re-fetch when needed and handle `NotFound`.

## Bounded name search

After access is already available, search with the iOS backend's public name predicate:

```rust,ignore
use framework_contacts::Contacts;
use ios_contacts::IosContactsBackend;

fn find_contacts(query: &str) -> Result<Vec<framework_contacts::Contact>, framework_contacts::ContactsError> {
    Contacts::new(IosContactsBackend::new()).search_contacts_by_name(query, 20)
}
```

The method rejects an empty or NUL-containing query as `InvalidSearchQuery`, rechecks current
authorization for every valid call, accepts only `Authorized` or `Limited`, and never prompts. A
zero result limit returns an empty vector after the authorization check and skips native
enumeration. For nonzero limits, the adapter uses
`CNContact::predicateForContactsMatchingName` and
`CNContactStore::enumerateContactsWithFetchRequest_error_usingBlock`; it copies only the existing
six common fields and sets the native stop flag as soon as the caller cap is reached. It does not
materialize all matches. Apple documents that the name may contain any number of words, but does
not specify substring, case, token-order, or result-order guarantees; the adapter passes the query
unchanged and exposes native callback order without a sort promise. Under Limited, only the
user-selected subset is searchable. No matches is an empty success; an enumeration error is
`Backend` unless a status recheck finds authorization no longer permits access.

Contacts fetches perform I/O, so keep this synchronous call off latency-sensitive UI paths when
appropriate. The Contacts name predicate, fetch request and enumeration API are available from iOS
9.0 in the installed Xcode 26.6 / iPhoneOS 26.5 SDK. The registered Rust deployment floors remain
iOS 10.0 for device and iOS 14.0 for Simulator. Apple API references:
[`predicateForContacts(matchingName:)`](https://developer.apple.com/documentation/contacts/cncontact/predicateforcontacts%28matchingname%3A%29) and
[`enumerateContacts(with:usingBlock:)`](https://developer.apple.com/documentation/contacts/cncontactstore/enumeratecontacts%28with%3Ausingblock%3A%29?language=objc).

`fetch_contact` is synchronous, and a Contacts store read can block. Avoid calling it from a
latency-sensitive UI context when the read may take time. The crate does not add a hidden executor;
the application decides where to perform the read.

Creating this backend or observing Authorized/Limited status does not prove a contact can be read.
Automated checks compile and link the adapter and test host status/error/value mappings; they do not
show permission UI, exercise consent or Limited selection, verify Settings changes, or prove a
live contact-data fetch.

## Dependency and linkage

The iOS-only adapter uses `objc2-contacts` 0.3.2 with default features disabled and only its
`CNContact`, `CNContactStore`, `CNContact_Predicates`, `CNContactFetchRequest`, `CNFetchRequest`,
`CNLabeledValue`, `CNPhoneNumber`, `CNError`, and `block2` features, plus `objc2`, Foundation
`NSArray`, `NSError`, and `NSString`, and `block2`. No ContactsUI, mutation, or unrelated binding
features are enabled. The portable crate exposes no binding types. The single-record and name
search APIs are available from iOS 9.0; Limited authorization itself was introduced in iOS 18.0.
Rust 1.94.1 sets iOS 10.0 as its minimum supported device deployment target, so the registered
device and Simulator link checks use floors iOS 10.0 and iOS 14.0, respectively. They verify public
Contacts/Foundation imports and deployment metadata, but do not execute the probe. No Swift
source, UI framework, or private API is added.

References: [Apple CNContactStore](https://developer.apple.com/documentation/contacts/cncontactstore),
[Apple Limited authorization](https://developer.apple.com/documentation/contacts/cnauthorizationstatus/limited?language=objc),
[Apple accessing the contact store](https://developer.apple.com/documentation/contacts/accessing-the-contact-store),
and [objc2 Contacts 0.3.2 bindings](https://docs.rs/objc2-contacts/0.3.2/objc2_contacts/).

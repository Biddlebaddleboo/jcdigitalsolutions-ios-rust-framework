# Portable Contacts authorization and data operations

`framework-contacts` defines a portable authorization contract and separate optional extensions
for one-record fetch and thumbnail read, bounded name search, add-only creation, selected-record
update and delete, and vCard export. It exposes no ContactsUI type or picker and does not enumerate
the full store. Existing `ContactsBackend` implementors need not implement these operations; a
backend opts in through the operation-specific extension trait.

## Authorization states

`ContactsAuthorization::Authorized` means the backend reports full access. `Limited` is separate:
it means the platform permits access to a user-selected subset, not all contacts. Both values make
`allows_contact_access()` true; only `Authorized` makes `is_full_access()` true. `Denied` and
`Restricted` do not permit access. `Unknown` is used for an unrecognized or unclassifiable state.

## Backend and request semantics

`Contacts<B>` owns the backend value supplied by its caller. The facade performs no global lookup,
hidden initialization, or executor selection; read methods allocate only the owned values in their
result contract. `authorization_status()` is a non-prompting query. `request_authorization()` is an
explicit operation; a native backend must defer its
prompt-capable call until the returned future is first polled. Dropping an unpolled future has no
prompt side effect. Dropping a started future abandons the result but cannot be assumed to dismiss
a system prompt. Callback state must not borrow future-owned storage and must remain valid until one
terminal native outcome. The first terminal completion becomes one future result; duplicate native
completions must not produce another result. A live future yields its result at most once.

## Read one record

`Contacts<B>::fetch_contact(identifier)` delegates to a `ContactsFetchBackend` and returns one
owned Rust snapshot. `Contact` contains the local identifier, given/family names, organization,
labeled phone numbers, and labeled email addresses. Names and organization are empty when absent;
labels are optional native label tokens, and the phone/email order is not a stable sorting promise.
The method is synchronous, and a platform store read can block. Avoid calling it from a
latency-sensitive UI context when the read may take time; the crate does not add a hidden executor.
The portable crate uses `no_std + alloc`; it does not expose Apple types.

```rust,ignore
use framework_contacts::Contacts;
use ios_contacts::IosContactsBackend;

fn read_selected_contact(identifier: &str) -> Result<framework_contacts::Contact, framework_contacts::ContactsError> {
    let contacts = Contacts::new(IosContactsBackend::new());
    contacts.fetch_contact(identifier)
}
```

Fetch never prompts or requests permission. It re-queries authorization for each call and proceeds
only for `Authorized` or `Limited`. The host app must include `NSContactsUsageDescription` for
Contacts access; this crate does not edit the host property list. Empty or NUL-containing
identifiers are invalid. A Contacts
`RecordDoesNotExist` result maps to `NotFound`; under `Limited`, an identifier outside the
user-selected subset can have the same result as a missing or stale identifier. Local identifiers
are not durable identities and may stop resolving after deletion or record unification. Contacts
may return a unified record whose identifier differs from the supplied identifier.

The fetch path reads only the six declared keys and returns owned values detached from the native
contact object. It does not observe later store changes. The iOS backend requires the host app's
`NSContactsUsageDescription` for Contacts access; fetch itself does not request access. Static
checks do not prove permission state, selected-contact visibility,
or a live Contacts fetch on simulator or device.

## Read one contact thumbnail

`Contacts<B>::fetch_contact_thumbnail(identifier)` delegates to the separate
`ContactsImageBackend` extension and returns `Ok(None)` when the selected contact has no thumbnail,
or owned `Vec<u8>` bytes copied from native thumbnail `NSData`. The iOS backend rejects empty or
NUL-containing identifiers, rechecks access without prompting, accepts only `Authorized` or
`Limited`, and fetches exactly the supplied identifier with only `CNContactThumbnailImageDataKey`.
It reads `thumbnailImageData()` only after that fetch and returns its bytes unchanged; it does not
fetch the full image, decode or transcode data, or alter the common `Contact` snapshot.

The operation is synchronous and may block on Contacts I/O. Limited access applies only to contacts
currently visible to the app. Registered host tests and static target compile/link checks do not
prove live thumbnail availability, Limited visibility, or physical-device behavior.

## Bounded name search

`Contacts<B>::search_contacts_by_name(query, max_results)` delegates to the separate
`ContactsSearchBackend` extension and returns at most the caller's `u32` limit of the same owned
`Contact` snapshots. Empty or NUL-containing queries return `InvalidSearchQuery`; valid queries
are passed unchanged to the backend. On iOS, search uses Apple's public
`CNContact::predicateForContactsMatchingName` with synchronous `CNContactFetchRequest`
enumeration. Apple documents that the name may contain any number of words, but this API does not
promise substring, case, token-order, or result-sort behavior. Result order follows native callback
delivery and is not a stable contract.

Every valid call rechecks authorization and proceeds only for `Authorized` or `Limited`; search
never requests access. A zero limit returns an empty vector after that check and without native
enumeration. For a nonzero cap, the iOS callback stops as soon as it reaches the limit and keeps no
more than that many snapshots. Limited access searches only the user-selected visible subset. A
native enumeration failure rechecks authorization and otherwise returns `Backend`; no matches are
a successful empty result, not `NotFound`.

Search is synchronous and Contacts fetches perform I/O, so avoid it on latency-sensitive UI paths.
The host app still supplies `NSContactsUsageDescription`; the crate adds no executor or prompt.
Host tests and iOS device/Simulator compile-link checks do not prove live matching, Limited
visibility, native callback stop delivery, or device runtime behavior. See Apple's
[`predicateForContacts(matchingName:)`](https://developer.apple.com/documentation/contacts/cncontact/predicateforcontacts%28matchingname%3A%29).

## Add one new record

`Contacts<B>::create_contact(&ContactDraft)` delegates to the separate add-only
`ContactsCreateBackend`. The Rust-owned draft contains given/family names, organization, and
optional-label phone/email values. At least one data value must be non-empty; empty phone/email
values are invalid. The backend receives label options and string values unchanged, without
trimming or locale-specific format validation.

The operation is synchronous and may block on Contacts I/O. It rechecks authorization for every
valid draft, never prompts, and proceeds only for `Authorized` or `Limited`. Apple documents that
under Limited access an app can create contacts and automatically access the contacts it creates;
Limited still does not grant access to other contacts outside the user-selected set. The iOS
backend uses one public `CNMutableContact` and one `CNSaveRequest.addContact` call with a nil
container identifier, which Apple documents as the default container. It does not fetch a
container, update/delete any existing record, or return an identifier: the native add/save methods
do not document a dependable post-save identifier result. A native failure is `Backend` with a
representable platform error code; if a fresh status query finds permission no longer allows the
operation, it is `AuthorizationRequired(status)` instead.

```rust,ignore
use framework_contacts::{ContactDraft, ContactLabeledValue, Contacts};
use ios_contacts::IosContactsBackend;

fn add_contact() -> Result<(), framework_contacts::ContactsError> {
    let draft = ContactDraft {
        given_name: String::from("Ada"),
        email_addresses: vec![ContactLabeledValue {
            label: Some(String::from("work")),
            value: String::from("ada@example.test"),
        }],
        ..ContactDraft::default()
    };
    Contacts::new(IosContactsBackend::new()).create_contact(&draft)
}
```

Contacts writes are synchronous; avoid latency-sensitive UI contexts. The host app owns its
`Info.plist` and must include `NSContactsUsageDescription`; the framework adds no prompt, executor,
UI, entitlement, or app metadata. Static host tests and device/Simulator compile/link checks do not
prove an actual save, Limited-access behavior, default-container selection, or a returned local
identifier.

## Update one existing record

`Contacts<B>::update_contact(identifier, &ContactUpdate)` delegates to the separate
`ContactsUpdateBackend` extension. Its owned optional fields select given name, family name,
organization, phone numbers, and email addresses. `None` preserves a property. `Some(String::new())`
clears a selected string; `Some(Vec::new())` clears the full phone or email list. A nonempty list
replaces only that list in caller order rather than appending. Empty phone/email values and a patch
with every field `None` return `InvalidContactUpdate`; empty names and an explicitly empty list are
valid clears.

The synchronous iOS operation validates before native work, rechecks current status, accepts only
`Authorized` or `Limited`, and never prompts. It fetches one contact by the caller's local
identifier, makes a public `CNMutableContact` copy, sets only `Some` fields, and executes one
`CNSaveRequest.updateContact:`. It returns unit and does not refetch a snapshot. Under Limited,
Apple permits edits only to contacts currently granted to the app (and automatically grants access
to contacts the app created); an identifier outside that scope must fail without prompting. A
stale-record Contacts error maps to `NotFound`; other native error codes are preserved unless a
fresh status query shows that access no longer permits the operation.

Apple may return a unified contact whose identifier differs from the supplied identifier. The
operation updates that one logical record returned for the supplied identifier, not a caller-chosen
account-level constituent. Contacts reads and saves are synchronous and may block. Host tests and
target compile/link checks do not prove a live edit, Limited visibility, concurrent change
behavior, Simulator execution, or physical-device behavior. See Apple's [Contacts access guide](https://developer.apple.com/documentation/contacts/accessing-the-contact-store),
[CNMutableContact](https://developer.apple.com/documentation/contacts/cnmutablecontact), and
[CNSaveRequest](https://developer.apple.com/documentation/contacts/cnsaverequest).

## Delete one selected record

`Contacts<B>::delete_contact(identifier)` delegates to the separate source-compatible
`ContactsDeleteBackend` extension. The synchronous operation rejects empty or NUL-containing
identifiers before native work, rechecks authorization for each valid call, permits only
`Authorized` or `Limited`, and never prompts. The iOS backend fetches only the caller-selected
identifier and only `CNContactIdentifierKey`, makes a public `CNMutableContact` copy, and executes
one `CNSaveRequest.deleteContact:`. It does not enumerate or search, delete groups, present UI, or
batch records. The return value is unit; this API provides no confirmation UI or undo.

Limited access permits deletion only when the selected contact is currently accessible to the app;
Apple documents that app-created contacts are automatically accessible. Contacts-domain
missing-record code 200 maps to `NotFound` while access remains allowed. If access changes before a
failed save, the fresh authorization result takes precedence; unrelated native errors remain
`Backend`. A unified lookup targets the single logical contact returned for the supplied identifier,
not a selected account-level constituent. Contacts fetch and save remain synchronous and may block.

The public `CNSaveRequest.deleteContact:` API and `CNContact` mutable-copy contract are available
from iOS 9.0 in the installed Xcode 26.6 / iPhoneOS 26.5 SDK and pinned `objc2-contacts` 0.3.2;
registered device and Simulator floors remain iOS 10.0 and iOS 14.0. Host tests and target
compile/link checks do not prove a live deletion, Limited-access decision, Simulator execution, or
physical-device behavior. See Apple's [Contacts access guide](https://developer.apple.com/documentation/contacts/accessing-the-contact-store),
[`CNSaveRequest`](https://developer.apple.com/documentation/contacts/cnsaverequest?language=objc), and
[`deleteContact:`](https://developer.apple.com/documentation/contacts/cnsaverequest/delete%28_%3A%29-8m1tc?language=objc).
